use std::char;
use std::ops::{Deref, DerefMut};
use std::{collections::HashMap, fs::read_to_string, path::Path};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::BorderType;
use tracing::*;

use super::error::Error;
use crate::config::Config;
use crate::event::AppEvent;

use super::utils::{get_border_connections, get_config_dir, get_data_dir};

const SEPERATOR: char = '=';

#[derive(Debug)]
struct RawConfig(pub HashMap<String, Vec<(String, String)>>);

impl Deref for RawConfig {
    type Target = HashMap<String, Vec<(String, String)>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RawConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Config {
    pub fn new() -> Result<Self, Error> {
        let config_path = get_config_dir().join("config");
        if config_path.exists() {
            Config::from_file(config_path)
        } else {
            info!("Found no config file, proceeding with default values");
            Ok(Config::default())
        }
    }

    fn from_file<P: AsRef<Path>>(path: P) -> Result<Config, Error> {
        let config_file = path.as_ref();
        if !config_file.exists() {
            return Err(Error::NoConfigFile(
                config_file.to_string_lossy().into_owned(),
            ));
        }
        let mut config = Config::default();

        config.config_dir = path.as_ref().to_path_buf();
        config.data_dir = get_data_dir();

        let raw_config = raw_from_file(config_file)?;

        info!("{:?}", raw_config);

        if let Some(style) = raw_config.get("settings") {
            for (raw_key, raw_value) in style {
                match raw_key.as_str() {
                    "default_leagues" => {
                        config.default_leagues =
                            raw_value.split(',').map(|s| s.trim().to_string()).collect()
                    }
                    "spoil_results" => config.spoil_results = parse_bool(raw_value)?,
                    "spoil_matches" => config.spoil_matches = parse_bool(raw_value)?,
                    "automatic_reload" => config.automatic_reload = parse_bool(raw_value)?,
                    _ => {
                        return Err(Error::UnknownKey(
                            raw_key.to_string(),
                            "settings".to_string(),
                        ));
                    }
                };
            }
        }

        if let Some(binds) = raw_config.get("keybindings") {
            for (raw_key, raw_command) in binds {
                let key_event = parse_key_event(raw_key)?;
                let command = parse_command(&raw_command)?;
                config.keybindings.insert(key_event, command);
            }
        }

        if let Some(style) = raw_config.get("style") {
            for (raw_key, raw_style) in style {
                match raw_key.as_str() {
                    "default" => config.style.default = parse_style(raw_style)?,
                    "highlight" => config.style.highlight = parse_style(raw_style)?,
                    "selected" => config.style.selected = parse_style(raw_style)?,
                    "winner" => config.style.winner = parse_optional_style(raw_style)?,
                    "loser" => config.style.loser = parse_optional_style(raw_style)?,
                    "border" => {
                        config.style.border = parse_border_type(raw_style)?;
                        config.style.border_set = get_border_connections(config.style.border);
                    }
                    _ => {
                        return Err(Error::UnknownKey(raw_key.to_string(), "style".to_string()));
                    }
                };
            }
        }

        Ok(config)
    }
}

fn raw_from_file<P: AsRef<Path>>(path: P) -> Result<RawConfig, Error> {
    let content = read_to_string(path)?;
    let mut section = String::new();
    let mut sections: RawConfig = RawConfig(HashMap::new());

    for (i, line) in content.lines().enumerate() {
        let line = line.trim();
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            if section.is_empty() {
                return Err(Error::EmptyHeader(i));
            }
            sections.entry(section.clone()).or_default();
            continue;
        } else if line.starts_with('[') || line.ends_with(']') {
            return Err(Error::IncompleteHeader(i));
        }

        if let Some((key, value)) = line.split_once(SEPERATOR) {
            let key = key.trim().to_string();
            if key.is_empty() {
                return Err(Error::EmptyKey(i));
            }
            let value = value.trim().to_string();
            if value.is_empty() {
                return Err(Error::EmptyValue(i));
            }
            if let Some(s) = sections.get_mut(&section) {
                s.push((key, value));
            } else {
                sections
                    .entry("settings".to_string())
                    .or_default()
                    .push((key, value));
            }
        } else {
            return Err(Error::MissingSeperator(i));
        }
    }
    Ok(sections)
}

fn parse_key_event(raw: &str) -> Result<KeyEvent, Error> {
    let raw_lower = raw.to_ascii_lowercase();
    let (remaining, modifiers) = extract_modifiers(&raw_lower);
    parse_key_code_with_modifiers(remaining, modifiers)
}

fn parse_command(raw: &str) -> Result<AppEvent, Error> {
    use AppEvent::*;

    Ok(match raw {
        "Quit" => Quit,
        "Up" => Up,
        "Down" => Down,
        "Left" => Left,
        "Right" => Right,
        "Select" => Select,
        "GotoToday" => GotoToday,
        "ToggleSpoilResults" => ToggleSpoilResults,
        "ToggleSpoilMatches" => ToggleSpoilMatches,
        "ReloadLeagues" => ReloadLeagues,
        "ReloadSchedule" => ReloadSchedule,
        _ => return Err(Error::InvalidCommand(raw.to_string())),
    })
}

fn extract_modifiers(raw: &str) -> (&str, KeyModifiers) {
    let mut modifiers = KeyModifiers::empty();
    let mut current = raw;

    loop {
        match current {
            rest if rest.starts_with("ctrl-") => {
                modifiers.insert(KeyModifiers::CONTROL);
                current = &rest[5..];
            }
            rest if rest.starts_with("alt-") => {
                modifiers.insert(KeyModifiers::ALT);
                current = &rest[4..];
            }
            rest if rest.starts_with("shift-") => {
                modifiers.insert(KeyModifiers::SHIFT);
                current = &rest[6..];
            }
            _ => break, // break out of the loop if no known prefix is detected
        };
    }

    (current, modifiers)
}

fn parse_key_code_with_modifiers(
    raw: &str,
    mut modifiers: KeyModifiers,
) -> Result<KeyEvent, Error> {
    let c = match raw {
        "esc" => KeyCode::Esc,
        "enter" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "backtab" => {
            modifiers.insert(KeyModifiers::SHIFT);
            KeyCode::BackTab
        }
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),
        "space" => KeyCode::Char(' '),
        "hyphen" => KeyCode::Char('-'),
        "minus" => KeyCode::Char('-'),
        "tab" => KeyCode::Tab,
        c if c.len() == 1 => {
            let mut c = c.chars().next().unwrap();
            if modifiers.contains(KeyModifiers::SHIFT) {
                c = c.to_ascii_uppercase();
            }
            KeyCode::Char(c)
        }
        _ => return Err(Error::InvalidKeybind(raw.to_string())),
    };
    Ok(KeyEvent::new(c, modifiers))
}

fn parse_style(line: &str) -> Result<Style, Error> {
    let (foreground, background) =
        line.split_at(line.to_lowercase().find("on ").unwrap_or(line.len()));
    let foreground = process_color_string(foreground);
    let background = process_color_string(&background.replace("on ", ""));

    let mut style = Style::default();
    if !foreground.0.is_empty() {
        let fg = parse_color(&foreground.0)?;
        style = style.fg(fg);
    }
    if !background.0.is_empty() {
        let bg = parse_color(&background.0)?;
        style = style.bg(bg);
    }
    style = style.add_modifier(foreground.1 | background.1);
    Ok(style)
}

fn parse_optional_style(line: &str) -> Result<Option<Style>, Error> {
    if line.to_lowercase() == "none" {
        Ok(None)
    } else {
        Ok(Some(parse_style(line)?))
    }
}

fn process_color_string(color_str: &str) -> (String, Modifier) {
    let color = color_str
        .replace("bold ", "")
        // .replace("underline ", "")
        .replace("inverse ", "")
        .trim()
        .to_string();

    let mut modifiers = Modifier::empty();
    /*
        if color_str.contains("underline") {
            modifiers |= Modifier::UNDERLINED;
        }
    */
    if color_str.contains("bold") {
        modifiers |= Modifier::BOLD;
    }
    if color_str.contains("inverse") {
        modifiers |= Modifier::REVERSED;
    }

    (color, modifiers)
}

fn parse_color(s: &str) -> Result<Color, Error> {
    if let Some(rgb) = parse_rgb(s) {
        return Ok(Color::Rgb(rgb.0, rgb.1, rgb.2));
    }
    match s {
        "black" => Ok(Color::Indexed(0)),
        "red" => Ok(Color::Indexed(1)),
        "green" => Ok(Color::Indexed(2)),
        "yellow" => Ok(Color::Indexed(3)),
        "blue" => Ok(Color::Indexed(4)),
        "magenta" => Ok(Color::Indexed(5)),
        "cyan" => Ok(Color::Indexed(6)),
        "gray" => Ok(Color::Indexed(7)),
        "bright black" => Ok(Color::Indexed(8)),
        "bright red" => Ok(Color::Indexed(9)),
        "bright green" => Ok(Color::Indexed(10)),
        "bright yellow" => Ok(Color::Indexed(11)),
        "bright blue" => Ok(Color::Indexed(12)),
        "bright magenta" => Ok(Color::Indexed(13)),
        "bright cyan" => Ok(Color::Indexed(14)),
        "white" => Ok(Color::Indexed(15)),
        _ => Err(Error::InvalidColor(s.to_string())),
    }
}

fn parse_rgb(s: &str) -> Option<(u8, u8, u8)> {
    if !s.starts_with('#') || s.len() != 7 {
        return None;
    }

    let r = u8::from_str_radix(&s[1..3], 16).ok()?;
    let g = u8::from_str_radix(&s[3..5], 16).ok()?;
    let b = u8::from_str_radix(&s[5..7], 16).ok()?;

    Some((r, g, b))
}

fn parse_border_type(line: &str) -> Result<Option<BorderType>, Error> {
    match line.to_lowercase().as_str() {
        "plain" => Ok(Some(BorderType::Plain)),
        "rounded" => Ok(Some(BorderType::Rounded)),
        "double" => Ok(Some(BorderType::Double)),
        "thick" => Ok(Some(BorderType::Thick)),
        "none" => Ok(None),
        _ => Err(Error::InvalidBorder(line.to_string())),
    }
}

fn parse_bool(line: &str) -> Result<bool, Error> {
    match line.to_lowercase().as_str() {
        "yes" | "true" => Ok(true),
        "no" | "false" => Ok(false),
        _ => return Err(Error::InvalidBool(line.to_string())),
    }
}
