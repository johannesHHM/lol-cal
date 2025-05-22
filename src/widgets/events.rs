use chrono::{DateTime, Local, NaiveDate};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Stylize},
    symbols::line,
    text::{Line, Text},
    widgets::{Block, Borders, Clear, StatefulWidgetRef, Widget, WidgetRef},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, rc::Rc};
use tracing::{debug, info};

use crate::config::Config;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StratType {
    BestOf(String),
    PlayAll(String),
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Strategy {
    pub strat_type: StratType,
    pub count: u16,
}

impl StratType {
    pub fn get_string(&self) -> &str {
        match self {
            StratType::BestOf(s) => s,
            StratType::PlayAll(s) => s,
            StratType::Unknown(s) => s,
        }
    }
}

impl From<String> for StratType {
    fn from(name: String) -> Self {
        match name.as_str() {
            "bestOf" => StratType::BestOf("Best of".to_string()),
            "playAll" => StratType::PlayAll("Play all".to_string()),
            _ => StratType::Unknown(name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchState {
    Completed(String),
    InProgress(String),
    Unstarted(String),
    Unknown(String),
}

impl MatchState {
    pub fn get_string(&self) -> &str {
        match self {
            MatchState::Completed(s) => s,
            MatchState::InProgress(s) => s,
            MatchState::Unstarted(s) => s,
            MatchState::Unknown(s) => s,
        }
    }
}

impl From<String> for MatchState {
    fn from(name: String) -> Self {
        match name.as_str() {
            "completed" => MatchState::Completed("Completed".to_string()),
            "inProgress" => MatchState::InProgress("In progress".to_string()),
            "unstarted" => MatchState::Unstarted("Unstarted".to_string()),
            _ => MatchState::Unknown(name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Team {
    pub name: String,
    pub short: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchResult {
    pub game_wins: (u16, u16),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub start_time: DateTime<Local>,
    pub league_name: String,
    pub block_name: String,
    pub strategy: Strategy,
    pub state: MatchState,
    pub result: Option<MatchResult>,
    pub teams: Vec<Team>,
}

#[derive(Debug, Default)]
pub struct ScheduleState {
    pub focused: bool,
    pub spoil_results: bool,
    pub spoil_matches: bool,
    pub offset: usize,
    pub selected: Option<usize>,
}

#[derive(Debug)]
pub struct Events {
    active: Vec<String>,
    events: HashMap<String, Vec<Event>>,
    config: Rc<Config>,
}

impl ScheduleState {
    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
        if index.is_none() {
            self.offset = 0;
        }
    }

    pub fn select_today(&mut self, events: &Events) {
        let today = Local::now();

        debug!("active: {:?}", events.active);
        let mut events: Vec<&Event> = events
            .events
            .iter()
            .filter(|(key, _)| events.active.contains(key))
            .flat_map(|(_, event_list)| event_list.iter())
            .collect();

        events.sort_by_key(|event| event.start_time);

        if events.is_empty() {
            return;
        }

        let sel = events
            .iter()
            .position(|e| e.start_time >= today || matches!(e.state, MatchState::InProgress(_)));

        self.selected = Some(sel.unwrap_or(events.len() - 1));
        self.offset = self.selected.unwrap_or_default();
    }

    pub fn scroll_up_by(&mut self, amount: u16) {
        match self.selected {
            Some(sel) => self.selected = Some(sel.saturating_sub(amount as usize)),
            None => self.selected = Some(self.offset),
        }
    }

    pub fn scroll_down_by(&mut self, amount: u16) {
        match self.selected {
            Some(sel) => self.selected = Some(sel.saturating_add(amount as usize)),
            None => self.selected = Some(self.offset),
        }
    }
}

impl Events {
    pub fn new(config: Rc<Config>) -> Self {
        Self {
            active: Vec::new(),
            events: HashMap::new(),
            config: config,
        }
    }

    pub fn add_events(&mut self, slug: String, events: Vec<Event>) {
        self.events.insert(slug, events);
        debug!("Inserted new events: {:?}", self.events);
    }

    pub fn set_active(&mut self, slug: String) {
        info!("Inserting new active: '{}'", slug);
        if !self.active.contains(&slug) {
            self.active.push(slug);
        }
    }

    pub fn unset_active(&mut self, slug: &str) {
        info!("Removing active: '{}'", slug);
        if let Some(pos) = self.active.iter().position(|x| x == slug) {
            self.active.remove(pos);
        }
    }

    fn get_events_bounds(
        &self,
        events: &Vec<&Event>,
        selected: Option<usize>,
        offset: usize,
        max_height: usize,
    ) -> (usize, usize) {
        let offset = offset.min(events.len().saturating_sub(1));

        let mut first_visible_index = offset;
        let mut last_visible_index = offset;

        let mut height_from_offset = 0;

        let mut last_date: Option<NaiveDate> = None;

        for event in events.iter().skip(offset) {
            if height_from_offset + EVENT_HEIGHT > max_height {
                break;
            }

            let current_date: NaiveDate = event.start_time.date_naive();

            if Some(current_date) != last_date {
                if height_from_offset + DATE_HEIGHT + EVENT_HEIGHT > max_height {
                    break;
                }
                height_from_offset += DATE_HEIGHT;
                last_date = Some(current_date);
            }
            height_from_offset += EVENT_HEIGHT;
            last_visible_index += 1;
        }

        let index_to_display = selected.unwrap_or(first_visible_index);

        while index_to_display >= last_visible_index {
            let date: NaiveDate = events[last_visible_index].start_time.date_naive();

            if Some(date) != last_date {
                height_from_offset = height_from_offset.saturating_add(DATE_HEIGHT);
                last_date = Some(date);
            }

            height_from_offset = height_from_offset.saturating_add(EVENT_HEIGHT);
            last_visible_index += 1;

            while height_from_offset > max_height {
                let first_date = events[first_visible_index].start_time.date_naive();

                let second_last_date = if first_visible_index + 1 <= last_visible_index {
                    Some(events[first_visible_index + 1].start_time.date_naive())
                } else {
                    None
                };

                if Some(first_date) != second_last_date {
                    height_from_offset = height_from_offset.saturating_sub(DATE_HEIGHT);
                }

                height_from_offset = height_from_offset.saturating_sub(EVENT_HEIGHT);
                first_visible_index += 1;
            }
        }

        while index_to_display < first_visible_index {
            let first_date = events[first_visible_index - 1].start_time.date_naive();

            if first_date != events[first_visible_index].start_time.date_naive() {
                height_from_offset = height_from_offset.saturating_add(DATE_HEIGHT);
            }

            height_from_offset = height_from_offset.saturating_add(EVENT_HEIGHT);
            first_visible_index -= 1;

            while height_from_offset > max_height {
                last_visible_index -= 1;
                let last_date = events[last_visible_index].start_time.date_naive();
                if last_date != events[last_visible_index - 1].start_time.date_naive() {
                    height_from_offset = height_from_offset.saturating_sub(DATE_HEIGHT);
                }
                height_from_offset = height_from_offset.saturating_sub(EVENT_HEIGHT);
            }
        }

        (first_visible_index, last_visible_index)
    }
}

const DATE_HEIGHT: usize = 2;
const EVENT_HEIGHT: usize = 2;

impl StatefulWidgetRef for &Events {
    type State = ScheduleState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if area.is_empty() {
            return;
        }

        let styles = &self.config.style;

        let mut events: Vec<&Event> = self
            .events
            .iter()
            .filter(|(slug, _)| self.active.contains(slug))
            .flat_map(|(_, events)| events.iter())
            .collect();

        let inner_area = {
            if let (Some(block), Some(set)) = (styles.border, styles.border_set) {
                let border_style = if state.focused {
                    styles.highlight
                } else {
                    styles.default
                };
                let block = Block::new()
                    .borders(Borders::all())
                    .border_type(block)
                    .border_style(border_style);
                block.render_ref(area, buf);
                let mut inner = block.inner(area);
                if inner.height >= 2 {
                    let hor = set.horizontal;
                    let left = set.vertical_right;
                    let right = set.vertical_left;

                    let title_area: Rect = Rect {
                        x: area.left(),
                        y: area.top() + 2,
                        width: area.width,
                        height: 1 as u16,
                    };

                    let title_line = Line::from(format!(
                        "{}{}{}",
                        left,
                        hor.repeat(area.width.saturating_sub(2) as usize),
                        right
                    ))
                    .style(border_style);
                    title_line.render_ref(title_area, buf);

                    let total_events: Vec<&Event> = self
                        .events
                        .iter()
                        .flat_map(|(_, events)| events.iter())
                        .collect();

                    let content = format!("({}/{})", events.len(), total_events.len());

                    if area.width as usize >= content.len() + 4 {
                        let showing_area: Rect = Rect {
                            x: title_area.x + area.width.saturating_sub(content.len() as u16 + 2),
                            y: title_area.y,
                            width: title_area.width.saturating_sub(4).min(content.len() as u16),
                            height: 1 as u16,
                        };

                        let showing_header =
                            Line::from(content).right_aligned().style(styles.highlight);
                        showing_header.render_ref(showing_area, buf);
                    }

                    let title_area: Rect = Rect {
                        x: area.left() + 1,
                        y: area.top() + 1,
                        width: area.width.saturating_sub(2),
                        height: 1 as u16,
                    };

                    let title = Line::from("Schedule").centered().style(styles.highlight);
                    title.render_ref(title_area, buf);

                    inner.y += 2;
                    inner.height = inner.height.saturating_sub(2);
                } else {
                    inner.y += 1;
                    inner.height = inner.height.saturating_sub(1);
                }
                inner
            } else {
                area
            }
        };
        if inner_area.is_empty() {
            return;
        }

        Clear.render(inner_area, buf);

        events.sort_by_key(|event| event.start_time);

        if events.is_empty() {
            state.selected = None;
            return;
        }

        // If the selected index is out of bounds, set it to the last item
        if state.selected.is_some_and(|s| s >= events.len()) {
            state.select(Some(events.len().saturating_sub(1)));
        }

        let max_height = inner_area.height as usize;

        let (first_visible_index, _) =
            self.get_events_bounds(&events, state.selected, state.offset, max_height);

        state.offset = first_visible_index;

        let mut current_height: u16 = 0;
        let mut last_date: Option<NaiveDate> = None;

        let hor_layout = Layout::horizontal([
            Constraint::Length(3),  // - or *
            Constraint::Length(5),  // time
            Constraint::Min(4),     // team0
            Constraint::Length(4),  // vs
            Constraint::Min(4),     // team1
            Constraint::Length(11), // empty
        ])
        .split(inner_area);

        let hor = if let Some(set) = styles.border_set {
            set.horizontal
        } else {
            line::HORIZONTAL
        };

        let date_header =
            Line::from(format!("{}", hor.repeat(inner_area.width as usize))).style(styles.default);

        for (i, event) in events.iter().enumerate().skip(state.offset) {
            let date: NaiveDate = event.start_time.date_naive();

            // If new date, render date header
            if Some(date) != last_date {
                if last_date != None {
                    if current_height as usize + 1 > max_height {
                        break;
                    }
                    let date_area: Rect = Rect {
                        x: inner_area.left(),
                        y: inner_area.top() + current_height,
                        width: inner_area.width,
                        height: 1 as u16,
                    };

                    date_header.render_ref(date_area, buf);
                    current_height += 1;
                }

                if current_height as usize + 1 > max_height {
                    break;
                }

                let style = if state
                    .selected
                    .is_some_and(|s| events[s].start_time.date_naive() == date)
                {
                    styles.selected
                } else {
                    styles.highlight
                };

                let date_line = Line::from(event.start_time.format("%A - %d %B ").to_string())
                    .right_aligned()
                    .style(style);

                let date_area: Rect = Rect {
                    x: inner_area.left(),
                    y: inner_area.top() + current_height,
                    width: inner_area.width,
                    height: 1 as u16,
                };
                current_height += 1;
                date_line.render(date_area, buf);
                last_date = Some(date);
            }

            if current_height as usize + 1 > max_height {
                break;
            }

            let event_top_layout: Rc<[Rect]> = hor_layout
                .iter()
                .map(|r| Rect {
                    x: r.x,
                    y: inner_area.top() + current_height,
                    width: r.width,
                    height: 1,
                })
                .collect();

            let mut style = if state.selected.is_some_and(|s| s == i) && state.focused {
                styles.highlight
            } else {
                styles.default
            };

            let (mut team0, mut team1) =
                if event_top_layout[2].width > 30 && event_top_layout[4].width > 30 {
                    (event.teams[0].name.clone(), event.teams[1].name.clone())
                } else {
                    (event.teams[0].short.clone(), event.teams[1].short.clone())
                };

            let mut style0 = style;
            let mut style1 = style;

            if state.spoil_results && !matches!(event.state, MatchState::Unstarted(_)) {
                (team0, team1) = match &event.result {
                    Some(res) => {
                        if matches!(event.state, MatchState::Completed(_)) {
                            if res.game_wins.0 > res.game_wins.1 {
                                if let Some(style_winner) = styles.winner {
                                    style0 = style_winner;
                                }
                                if let Some(style_loser) = styles.loser {
                                    style1 = style_loser;
                                }
                            } else if res.game_wins.1 > res.game_wins.0 {
                                if let Some(style_winner) = styles.winner {
                                    style1 = style_winner;
                                }
                                if let Some(style_loser) = styles.loser {
                                    style0 = style_loser;
                                }
                            }
                        }
                        (
                            format!("{} - {}", res.game_wins.0, team0),
                            format!("{} - {}", team1, res.game_wins.1),
                        )
                    }
                    None => (team0, team1),
                };
            }

            if !state.spoil_matches && matches!(event.state, MatchState::Unstarted(_)) {
                if event.teams[0].name != "TBD" {
                    team0 = "???".to_string();
                }
                if event.teams[1].name != "TBD" {
                    team1 = "???".to_string();
                }
            }

            if state.selected.is_some_and(|s| s == i) && state.focused {
                style.bg = styles.selected.bg;
                style0.bg = styles.selected.bg;
                style1.bg = styles.selected.bg;
            }

            Text::from(if state.selected.is_some_and(|s| s == i) {
                " * "
            } else {
                " - "
            })
            .style(style)
            .render(event_top_layout[0], buf);
            Text::from(event.start_time.format("%H:%M").to_string())
                .style(style)
                .add_modifier(Modifier::BOLD)
                .left_aligned()
                .render(event_top_layout[1], buf);
            Text::from(team0)
                .style(style0)
                .right_aligned()
                .render(event_top_layout[2], buf);
            Text::from(" vs ")
                .style(style)
                .centered()
                .render(event_top_layout[3], buf);
            Text::from(team1)
                .style(style1)
                .left_aligned()
                .render(event_top_layout[4], buf);
            Text::from(event.state.get_string())
                .style(style)
                .right_aligned()
                .render(event_top_layout[5], buf);
            current_height += 1;

            if current_height as usize + 1 > max_height {
                break;
            }

            let event_low_area = Rect {
                x: inner_area.left(),
                y: inner_area.top() + current_height,
                width: inner_area.width,
                height: 1,
            };

            Text::from(format!(
                "   {} {}",
                event.strategy.strat_type.get_string(),
                event.strategy.count
            ))
            .left_aligned()
            .style(style)
            .render(event_low_area, buf);

            Text::from(format!(
                "{} - {}",
                event.block_name.to_owned(),
                event.league_name,
            ))
            .right_aligned()
            .style(style)
            .render(event_low_area, buf);

            current_height += 1;
        }
    }
}
