use std::rc::Rc;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::KeyEvent,
    layout::{Constraint, Layout, Rect},
};
use strum::Display;
use tracing::*;

use crate::{
    config::{self, Config},
    event::{AppEvent, Event, EventHandler},
    resources::ResourceManager,
    widgets::{
        events::{Events, ScheduleState},
        leagues::{Leagues, LeaguesState},
    },
};

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub events: EventHandler,
    pub config: Rc<Config>,
    pub resources: ResourceManager,

    pub mode: Mode,
    pub leagues: Leagues,
    pub leagues_state: LeaguesState,
    pub schedule: Events,
    pub schedule_state: ScheduleState,
}

#[derive(Debug, Default, Display)]
pub enum Mode {
    #[default]
    None,
    Leagues,
    Events,
}

impl App {
    pub fn new() -> Result<Self, config::Error> {
        let config = Rc::new(Config::new()?);
        let resources = ResourceManager::new(config.data_dir.clone());
        let schedule = Events::new(config.clone());
        let leagues = Leagues::new(config.clone());
        let events = EventHandler::new();

        Ok(App {
            running: true,
            events: events,
            config: config,
            resources: resources,
            mode: Mode::default(),
            leagues: leagues,
            leagues_state: LeaguesState::default(),
            schedule: schedule,
            schedule_state: ScheduleState::default(),
        })
    }

    pub fn init(&mut self) {
        self.events.send(AppEvent::ReloadLeagues);
        self.schedule_state.spoil_results = self.config.spoil_results;
        self.schedule_state.spoil_matches = self.config.spoil_matches;
    }

    fn handle_up(&mut self) {
        match self.mode {
            Mode::None => {}
            Mode::Leagues => self.leagues_state.list_state.scroll_up_by(1),
            Mode::Events => self.schedule_state.scroll_up_by(1),
        }
    }

    fn handle_down(&mut self) {
        match self.mode {
            Mode::None => {}
            Mode::Leagues => self.leagues_state.list_state.scroll_down_by(1),
            Mode::Events => self.schedule_state.scroll_down_by(1),
        }
    }

    fn handle_left(&mut self) {
        match self.mode {
            Mode::Leagues => {}
            Mode::Events | Mode::None => {
                self.mode = Mode::Leagues;
                self.schedule_state.focused = false;
                self.leagues_state.focused = true;
            }
        }
    }

    fn handle_right(&mut self) {
        match self.mode {
            Mode::Leagues | Mode::None => {
                self.mode = Mode::Events;
                self.schedule_state.focused = true;
                self.leagues_state.focused = false;
            }
            Mode::Events => {}
        }
    }

    fn handle_select(&mut self) {
        match self.mode {
            Mode::None => {}
            Mode::Leagues => {
                let id = self.leagues.select(&self.leagues_state.list_state);
                if let Some((selected, id)) = id {
                    match selected {
                        true => self.set_active(id),
                        false => self.schedule.unset_active(&id),
                    }
                    self.schedule_state.select_today(&self.schedule);
                }
            }
            Mode::Events => {}
        }
    }

    fn reload_leagues(&mut self) {
        let sender = self.events.get_sender_clone();
        let resources = self.resources.clone();
        tokio::spawn(async move {
            match resources.get_leagues().await {
                Some(leagues) => sender
                    .send(Event::App(AppEvent::RecieveLeagues(leagues)))
                    .unwrap(),
                None => {}
            };
        });
    }

    fn reload_schedule(&mut self) {
        let slugs = self.leagues.get_selected_ids();
        if slugs.is_empty() {
            return;
        }

        let sender = self.events.get_sender_clone();
        let resources = self.resources.clone();

        tokio::spawn(async move {
            for slug in slugs {
                match resources.get_schedule(&slug).await {
                    Some(events) => sender
                        .send(Event::App(AppEvent::RecieveSchedule((slug, events))))
                        .unwrap(),
                    None => {}
                };
            }
        });
    }

    fn set_active(&mut self, slug: String) {
        self.schedule.set_active(slug);
        if self.config.automatic_reload {
            self.reload_schedule();
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| self.draw(frame, frame.area()))?;
            match self.events.next().await? {
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event) => self.handle_key_events(key_event)?,
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::Quit => self.quit(),
                    AppEvent::Up => self.handle_up(),
                    AppEvent::Down => self.handle_down(),
                    AppEvent::Left => self.handle_left(),
                    AppEvent::Right => self.handle_right(),
                    AppEvent::Select => self.handle_select(),

                    AppEvent::GotoToday => self.schedule_state.select_today(&self.schedule),
                    AppEvent::ToggleSpoilResults => {
                        self.schedule_state.spoil_results = !self.schedule_state.spoil_results
                    }
                    AppEvent::ToggleSpoilMatches => {
                        self.schedule_state.spoil_matches = !self.schedule_state.spoil_matches
                    }

                    AppEvent::ReloadLeagues => self.reload_leagues(),
                    AppEvent::RecieveLeagues(l) => {
                        self.leagues.set_leagues(l);
                        if !self.leagues.leagues.is_empty() {
                            self.leagues_state.list_state.select_first();
                            let default_leagues = self.config.default_leagues.clone();
                            for name in &default_leagues {
                                match self.leagues.select_name(name) {
                                    Some(id) => self.set_active(id),
                                    None => warn!("Could not find default league '{}'", name),
                                }
                            }
                        }
                    }
                    AppEvent::ReloadSchedule => self.reload_schedule(),
                    AppEvent::RecieveSchedule((slug, events)) => {
                        self.schedule.add_events(slug, events);
                        self.schedule_state.select_today(&self.schedule);
                    }
                },
            }
        }
        Ok(())
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match self.config.keybindings.get(&key_event) {
            Some(app_event) => self.events.send(app_event.clone()),
            None => {}
        };
        Ok(())
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        // let vert_areas = Layout::vertical([Constraint::Max(1), Constraint::Min(0)]).split(area);
        let mut league_padding = 3;
        if self.config.style.border.is_some() {
            league_padding += 2;
        }
        let hor_areas = Layout::horizontal([
            Constraint::Length(self.leagues.longest + league_padding),
            Constraint::Min(50),
        ])
        .split(area);

        frame.render_stateful_widget_ref(&self.leagues, hor_areas[0], &mut self.leagues_state);
        frame.render_stateful_widget_ref(&self.schedule, hor_areas[1], &mut self.schedule_state);

        /*
        let top_line = Text::from(format!(
            "mode: {} | schedule_state: {} {:?}",
            self.mode, self.schedule_state.offset, self.schedule_state.selected
        ));

        frame.render_widget(top_line, vert_areas[0]);
        */
    }
}
