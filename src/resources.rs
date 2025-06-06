use std::path::PathBuf;

use chrono::{DateTime, Duration, Local, Utc};
use reqwest::Client;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{error, info};

use crate::net;
use crate::widgets::events::{Event, MatchResult, Strategy, Team};
use crate::widgets::leagues::League;

#[derive(Debug, Clone)]
pub struct ResourceManager {
    cache_dir: PathBuf,
}

impl ResourceManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            cache_dir: data_dir.join("cache"),
        }
    }

    async fn cache_data<T: Serialize>(&self, name: &str, data: &T) -> std::io::Result<()> {
        let cache_path = self.cache_dir.join(name);

        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let serialized = serde_json::to_vec(data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let mut file = fs::File::create(&cache_path).await?;
        file.write_all(&serialized).await?;

        Ok(())
    }

    async fn load_from_cache<T: DeserializeOwned>(
        &self,
        name: &str,
    ) -> std::io::Result<(T, DateTime<Local>)> {
        let cache_path = self.cache_dir.join(name);

        let mut file = fs::File::open(&cache_path).await?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;

        let metadata = fs::metadata(&cache_path).await?;
        let modified_time = metadata.modified()?;
        let modified_datetime: DateTime<Local> = modified_time.into();

        let data = serde_json::from_slice(&contents)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        Ok((data, modified_datetime))
    }

    pub async fn get_leagues(&self) -> Option<Vec<League>> {
        match self.load_from_cache("leagues.json").await {
            Ok((leagues, cached_time)) => 'fetch: {
                info!("Successfully loaded cached leagues");
                let now = Local::now();

                if cached_time < now - Duration::days(7) {
                    info!("Cached leagues is older then 7 days, fetching new list");
                    break 'fetch;
                }
                return Some(leagues);
            }
            Err(e) => info!("Failed to load cached leagues: {:?}", e),
        }

        let client = Client::new();

        match net::leagues::fetch_leagues(&client).await {
            Ok(leagues) => {
                info!(
                    "Successfully fetched leagues from API, total leagues: {}",
                    leagues.len()
                );
                let leagues = leagues.into_iter().map(League::from).collect();
                match self.cache_data("leagues.json", &leagues).await {
                    Ok(_) => info!("Successfully cached leagues"),
                    Err(e) => error!("Failed to cache leagues: {:?}", e),
                }
                return Some(leagues);
            }
            Err(e) => error!("Failed to fetch leagues: {:?}", e),
        }
        return None;
    }

    pub async fn get_schedule(&self, slug: &str) -> Option<Vec<Event>> {
        // TODO: Currently paging is ignored, would probably make sense to handle
        // this outside of get_schedule, so that we don't have to wait for all
        // pages to be gotten.

        let cache_path = format!("{}.json", slug);

        match self.load_from_cache(&cache_path).await {
            Ok((events, cached_time)) => 'fetch: {
                info!("Successfully loaded cached schedule '{}'", slug);
                let now = Local::now();

                if cached_time < now - Duration::days(3) {
                    info!("Cached schedule is older then 3 days, need to fetch newer");
                    break 'fetch;
                }

                if cached_time > now - Duration::minutes(5) {
                    info!("Cached schedule is younger then 5 minutes, accepting cached data");
                    return Some(events);
                }

                let events: Vec<Event> = events;
                let has_invalid_event = events
                    .iter()
                    .any(|e| e.state.get_string() == "Unstarted" && e.start_time < now);

                if has_invalid_event {
                    info!("Cached schedule is outdated due to unstarted past events");
                    break 'fetch;
                }
                return Some(events);
            }
            Err(e) => info!("Failed to load cached schedule '{}': {:?}", slug, e),
        }

        let client = Client::new();

        match net::schedule::fetch_schedule(&client, slug, None).await {
            Ok(schedule) => {
                info!(
                    "Successfully fetched schedule from API, slug: {}, pages: (before: {:?} after: {:?}) total events: {}",
                    slug,
                    schedule.pages.older,
                    schedule.pages.newer,
                    schedule.events.len()
                );
                let events = schedule.events.into_iter().map(Event::from).collect();
                match self.cache_data(&cache_path, &events).await {
                    Ok(_) => info!("Successfully cached schedule '{}'", slug),
                    Err(e) => error!("Failed to cache schedule '{}': {:?}", slug, e),
                }
                return Some(events);
            }
            Err(e) => error!("Failed to fetch schedule: {:?}", e),
        }
        return None;
    }
}

impl From<net::leagues::League> for League {
    fn from(net_league: net::leagues::League) -> Self {
        Self {
            id: net_league.id,
            name: net_league.name,
            region: net_league.region,
            selected: false,
        }
    }
}

impl From<net::schedule::Event> for Event {
    fn from(net_event: net::schedule::Event) -> Self {
        Self {
            start_time: net_event
                .start_time
                .parse::<DateTime<Utc>>()
                .unwrap()
                .with_timezone(&Local),
            league_name: net_event.league.name,
            block_name: net_event.block_name,
            strategy: Strategy {
                strat_type: net_event.match_field.strategy.type_field.clone().into(),
                count: net_event.match_field.strategy.count as u16,
            },
            state: net_event.state.into(),
            result: (&net_event.match_field).into(),
            teams: net_event
                .match_field
                .teams
                .into_iter()
                .map(|team| Team {
                    name: team.name,
                    short: team.code,
                })
                .collect(),
        }
    }
}

impl From<&net::schedule::Match> for Option<MatchResult> {
    fn from(net_match: &net::schedule::Match) -> Option<MatchResult> {
        if let (Some(rec0), Some(rec1)) = (&net_match.teams[0].result, &net_match.teams[1].result) {
            Some(MatchResult {
                game_wins: (rec0.game_wins as u16, rec1.game_wins as u16),
            })
        } else {
            None
        }
    }
}
