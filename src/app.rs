use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout},
    prelude::Rect,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::{
    action::Action,
    app_event::{AppEvent, MessageType},
    components::{
        Component, connection_menu::ConnectionMenu, messages::Messages,
        results_table::ResultsTable, text_editor::TextEditor,
    },
    config::Config,
    database::connection::DbConnection,
    tui::Tui,
};

pub struct App {
    config: Config,
    tick_rate: f64,
    frame_rate: f64,
    components: Vec<Box<dyn Component>>,
    should_quit: bool,
    should_suspend: bool,
    mode: Mode,
    db_connection: Option<DbConnection>,
    last_tick_key_events: Vec<KeyEvent>,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    event_tx: mpsc::UnboundedSender<AppEvent>,
    event_rx: mpsc::UnboundedReceiver<AppEvent>,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    /// Choose a database connection
    #[default]
    ConnectionMenu,
    /// Edit text of SQL query
    EditQuery,
    /// Navigate the result table
    ExploreResults,
}

impl App {
    pub fn new(tick_rate: f64, frame_rate: f64) -> color_eyre::Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        Ok(Self {
            tick_rate,
            frame_rate,
            components: vec![
                Box::new(ConnectionMenu::new()),
                Box::new(TextEditor::new()),
                Box::new(ResultsTable::default()),
                Box::new(Messages::default()),
            ],
            should_quit: false,
            should_suspend: false,
            config: Config::new()?,
            mode: Mode::default(),
            db_connection: None,
            last_tick_key_events: Vec::new(),
            action_tx,
            action_rx,
            event_tx,
            event_rx,
        })
    }

    pub async fn run(&mut self) -> color_eyre::Result<()> {
        let mut tui = Tui::new()?
            // .mouse(true) // uncomment this line to enable mouse support
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);
        tui.enter()?;

        for component in self.components.iter_mut() {
            component.register_action_handler(self.action_tx.clone())?;
        }
        for component in self.components.iter_mut() {
            component.register_config_handler(self.config.clone())?;
        }
        for component in self.components.iter_mut() {
            component.init(tui.size()?)?;
        }

        let action_tx = self.action_tx.clone();
        loop {
            self.handle_events(&mut tui).await?;
            self.handle_app_events()?;
            self.handle_actions(&mut tui)?;
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                action_tx.send(Action::ClearScreen)?;
                // tui.mouse(true);
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }

    async fn handle_events(&mut self, tui: &mut Tui) -> color_eyre::Result<()> {
        let Some(event) = tui.next_event().await else {
            return Ok(());
        };
        let action_tx = self.action_tx.clone();
        match event {
            crate::tui::Event::Quit => action_tx.send(Action::Quit)?,
            crate::tui::Event::Tick => action_tx.send(Action::Tick)?,
            crate::tui::Event::Render => action_tx.send(Action::Render)?,
            crate::tui::Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
            crate::tui::Event::Key(key) => self.handle_key_event(key)?,
            _ => {}
        }
        for component in self.components.iter_mut() {
            if let Some(action) = component.handle_events(Some(event.clone()))? {
                action_tx.send(action)?;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        let action_tx = self.action_tx.clone();
        let Some(keymap) = self.config.keybindings.0.get(&self.mode) else {
            return Ok(());
        };
        match keymap.get(&vec![key]) {
            Some(action) => {
                info!("Got action: {action:?}");
                action_tx.send(action.clone())?;
            }
            _ => {
                // If the key was not handled as a single key action,
                // then consider it for multi-key combinations.
                self.last_tick_key_events.push(key);

                // Check for multi-key combinations
                if let Some(action) = keymap.get(&self.last_tick_key_events) {
                    info!("Got action: {action:?}");
                    action_tx.send(action.clone())?;
                } else {
                    // Key event does not match any keybind. Match on globally available keybinds.
                    match key.code {
                        KeyCode::F(1) => {
                            action_tx.send(Action::ChangeMode(Mode::EditQuery))?;
                        }
                        KeyCode::F(2) => {
                            action_tx.send(Action::ChangeMode(Mode::ExploreResults))?;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> color_eyre::Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action != Action::Tick && action != Action::Render {
                debug!("{action:?}");
            }
            match action.clone() {
                Action::Tick => {
                    self.last_tick_key_events.drain(..);
                }
                Action::Quit => self.should_quit = true,
                Action::Suspend => self.should_suspend = true,
                Action::Resume => self.should_suspend = false,
                Action::ClearScreen => tui.terminal.clear()?,
                Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
                Action::Render => self.render(tui)?,
                Action::ChangeMode(new_mode) => self.mode = new_mode,
                Action::OpenDbConnection(connection_name) => {
                    if let Some(db_config) = self.config.db_connections.0.get(&connection_name) {
                        let config = db_config.clone();
                        let event_tx = self.event_tx.clone();
                        tokio::spawn(async move {
                            event_tx.send(AppEvent::UserMessage(
                                MessageType::Info,
                                String::from("Connecting..."),
                            ))?;
                            let connect_result = DbConnection::create(config).await;
                            match connect_result {
                                Ok(connection) => {
                                    event_tx.send(AppEvent::DbConnectionEstablished(connection))?;
                                    event_tx.send(AppEvent::UserMessage(
                                        MessageType::Info,
                                        String::from("Connected!"),
                                    ))
                                }
                                Err(e) => event_tx.send(AppEvent::UserMessage(
                                    MessageType::Error,
                                    format!("{:?}", e),
                                )),
                            }
                        });
                    } else {
                        error!("Attempted to open an unknown connection");
                    }
                }
                Action::ExecuteQuery(query) => {
                    // When a query is executed, report the result back via an app event.
                    let tx = self.event_tx.clone();
                    if let Some(connection) = self.db_connection.clone() {
                        tokio::spawn(async move {
                            let res = connection.get_query_result(query.clone()).await;
                            match res {
                                Ok(query_result) => tx.send(AppEvent::QueryResult(query_result)),
                                Err(db_error) => tx.send(AppEvent::UserMessage(
                                    MessageType::Error,
                                    db_error
                                        .as_db_error()
                                        .map_or(String::from("Unknown error"), |e| e.to_string()),
                                )),
                            }
                        });
                    } else {
                        self.event_tx.send(AppEvent::UserMessage(
                            MessageType::Error,
                            String::from("No connection established."),
                        ))?;
                    }
                }
                _ => {}
            }
            for component in self.components.iter_mut() {
                if let Some(action) = component.update(action.clone())? {
                    self.action_tx.send(action)?
                };
            }
        }
        Ok(())
    }

    fn handle_app_events(&mut self) -> color_eyre::Result<()> {
        while let Ok(app_event) = self.event_rx.try_recv() {
            match app_event.clone() {
                AppEvent::DbConnectionEstablished(connection) => {
                    self.db_connection = Some(connection)
                }
                AppEvent::QueryResult(result) => {
                    self.event_tx.clone().send(AppEvent::UserMessage(
                        MessageType::Info,
                        format!("{} results", result.rows.len()),
                    ))?
                }
                _ => {}
            }
            for component in self.components.iter_mut() {
                if let Some(action) = component.handle_app_events(app_event.clone())? {
                    self.action_tx.send(action)?
                };
            }
        }
        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> color_eyre::Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> color_eyre::Result<()> {
        tui.draw(|frame| {
            let layout = Layout::vertical([
                Constraint::Percentage(10),
                Constraint::Percentage(40),
                Constraint::Percentage(40),
                Constraint::Percentage(10),
            ])
            .split(frame.area());
            for (i, component) in self.components.iter_mut().enumerate() {
                if let Err(err) = component.draw(frame, layout[i]) {
                    let _ = self
                        .action_tx
                        .send(Action::Error(format!("Failed to draw: {:?}", err)));
                }
            }
        })?;
        Ok(())
    }
}
