use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazy_static::lazy_static;
use ratatui::{
    style::{Style, Stylize},
    symbols::line,
    widgets::BorderType,
};
use utils::{get_config_dir, get_data_dir};

use crate::event::AppEvent;

mod error;
pub use error::Error;
pub mod parser;
pub mod utils;

lazy_static! {
    pub static ref PROJECT_NAME: String = env!("CARGO_CRATE_NAME").to_uppercase().to_string();
}

#[derive(Debug)]
pub struct KeyBindings(pub HashMap<KeyEvent, AppEvent>);

impl Default for KeyBindings {
    fn default() -> Self {
        let mut map = HashMap::new();

        macro_rules! bind {
            ($code:expr => $event:expr) => {
                map.insert(KeyEvent::new($code, KeyModifiers::NONE), $event);
            };
            ($code:expr, $mods:expr => $event:expr) => {
                map.insert(KeyEvent::new($code, $mods), $event);
            };
        }

        bind!(KeyCode::Char('q') => AppEvent::Quit);
        bind!(KeyCode::Char('c'), KeyModifiers::CONTROL => AppEvent::Quit);

        bind!(KeyCode::Char('k') => AppEvent::Up);
        bind!(KeyCode::Char('j') => AppEvent::Down);
        bind!(KeyCode::Char('h') => AppEvent::Left);
        bind!(KeyCode::Char('l') => AppEvent::Right);

        bind!(KeyCode::Up => AppEvent::Up);
        bind!(KeyCode::Down => AppEvent::Down);
        bind!(KeyCode::Left => AppEvent::Left);
        bind!(KeyCode::Right => AppEvent::Right);
        bind!(KeyCode::Char(' ') => AppEvent::Select);

        bind!(KeyCode::Char('g'), KeyModifiers::CONTROL => AppEvent::GotoToday);
        bind!(KeyCode::Char('s'), KeyModifiers::CONTROL => AppEvent::ToggleSpoilResults);
        bind!(KeyCode::Char('s'), KeyModifiers::SHIFT => AppEvent::ToggleSpoilMatches);

        bind!(KeyCode::Char('r') => AppEvent::ReloadSchedule);

        KeyBindings(map)
    }
}

impl Deref for KeyBindings {
    type Target = HashMap<KeyEvent, AppEvent>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for KeyBindings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
pub struct Styles {
    pub border: Option<BorderType>,
    pub border_set: Option<line::Set>,
    pub default: Style,
    pub highlight: Style,
    pub selected: Style,
    pub winner: Option<Style>,
    pub loser: Option<Style>,
}

impl Default for Styles {
    fn default() -> Self {
        Styles {
            border: Some(BorderType::Plain),
            border_set: Some(line::NORMAL),
            default: Style::default(),
            highlight: Style::default().blue(),
            selected: Style::default().red().bold(),
            winner: Some(Style::default().green()),
            loser: None,
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub default_leagues: Vec<String>,
    pub spoil_results: bool,
    pub spoil_matches: bool,
    pub automatic_reload: bool,
    pub keybindings: KeyBindings,
    pub style: Styles,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config_dir: get_config_dir(),
            data_dir: get_data_dir(),
            default_leagues: Vec::new(),
            spoil_results: false,
            spoil_matches: true,
            automatic_reload: true,
            keybindings: KeyBindings::default(),
            style: Styles::default(),
        }
    }
}
