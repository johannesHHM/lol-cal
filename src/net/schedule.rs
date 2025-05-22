use serde::de;
// TODO: remove this later
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

use reqwest::Client;
use serde_json::Value;
use tracing::info;

use crate::net::*;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Root {
    data: Data,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Data {
    schedule: Schedule,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub pages: Pages,
    #[serde(deserialize_with = "deserialize_events")]
    pub events: Vec<Event>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pages {
    pub older: Option<String>,
    pub newer: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub start_time: String,
    pub block_name: String,
    #[serde(rename = "match")]
    pub match_field: Match,
    pub state: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub league: League,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Match {
    pub teams: Vec<Team>,
    pub id: String,
    pub strategy: Strategy,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub code: String,
    pub image: String,
    pub name: String,
    pub result: Option<Resultt>,
    pub record: Option<Record>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resultt {
    pub game_wins: i64,
    pub outcome: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub losses: i64,
    pub wins: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Strategy {
    pub count: i64,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct League {
    pub name: String,
    pub slug: String,
}

// When an event is not a match the "type" field != match.
// We need to filter these out
fn deserialize_events<'de, D>(deserializer: D) -> Result<Vec<Event>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw_events: Vec<Value> = Deserialize::deserialize(deserializer)?;
    let mut filtered = Vec::new();

    for raw in raw_events {
        // Only process if type == "match"
        if raw
            .get("type")
            .and_then(Value::as_str)
            .map(|s| s == "match")
            .unwrap_or(false)
        {
            let event: Event = serde_json::from_value(raw).map_err(|e| de::Error::custom(e))?;
            filtered.push(event);
        }
    }

    Ok(filtered)
}

const SCHEDULE_URL: &str =
    "https://esports-api.lolesports.com/persisted/gw/getSchedule?hl=en-US&leagueId=";

pub async fn fetch_schedule(
    client: &Client,
    slug: &str,
    page: Option<&str>,
) -> Result<Schedule, Error> {
    let url = match page {
        Some(token) => SCHEDULE_URL.to_owned() + &slug + "pageToken=" + &token,
        None => SCHEDULE_URL.to_owned() + &slug,
    };

    let response = client
        .get(url)
        .header(X_API_KEY_NAME, X_API_KEY_VALUE)
        .send()
        .await?;

    if response.status().is_success() {
        let api_response: Root = response
            .json()
            .await
            .map_err(|e| Error::Deserialize(e.to_string()))?;
        info!("{:?}", api_response.data.schedule);
        return Ok(api_response.data.schedule);
    } else {
        return Err(Error::Request(response.status()));
    }
}
