use crate::app::App;
use tracing::*;

pub mod app;
pub mod config;
pub mod event;
pub mod logging;
pub mod net;
pub mod resources;
pub mod ui;
pub mod widgets;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    tui_main().await
}

async fn tui_main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    crate::logging::init()?;
    let mut app = App::new()?;
    app.init();

    info!("{:?}", app.config);

    let mut terminal = ratatui::init();
    terminal.clear()?; // needed for first clear in tty
    let result = app.run(terminal).await;
    ratatui::restore();
    result
}
