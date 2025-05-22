use std::path::PathBuf;

use directories::ProjectDirs;
use ratatui::{symbols::line, widgets::BorderType};

pub fn get_config_dir() -> PathBuf {
    if let Some(proj_dirs) = project_directory() {
        proj_dirs.config_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".config")
    }
}

pub fn get_data_dir() -> PathBuf {
    if let Some(proj_dirs) = project_directory() {
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".config")
    }
}

fn project_directory() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "johannesHHM", env!("CARGO_PKG_NAME"))
}

pub fn get_border_connections(border: Option<BorderType>) -> Option<line::Set> {
    match border {
        Some(BorderType::Plain) => Some(line::NORMAL),
        Some(BorderType::Rounded) => Some(line::ROUNDED),
        Some(BorderType::Double) => Some(line::DOUBLE),
        Some(BorderType::Thick) => Some(line::THICK),
        _ => None,
    }
}
