use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{action::Action, config::Config};

pub struct ConnectionMenu {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    list_state: ListState,
}

impl Default for ConnectionMenu {
    fn default() -> Self {
        Self {
            command_tx: Default::default(),
            config: Default::default(),
            list_state: ListState::default().with_selected(Some(0)),
        }
    }
}

impl ConnectionMenu {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for ConnectionMenu {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> color_eyre::Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> color_eyre::Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
        match action {
            Action::NavDown => self.list_state.select_next(),
            Action::NavUp => self.list_state.select_previous(),
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        // println!("config? {:?}", self.config);
        let items: Vec<String> = self.config.db_connections.0.clone().into_keys().collect();
        if !items.is_empty() {
            let block = Block::bordered()
                .title("choose a connection")
                .style(Style::new().fg(Color::Cyan))
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Thick);
            let list = List::new(items)
                .style(Color::Cyan)
                .highlight_style(Modifier::REVERSED)
                .highlight_symbol("▹ ")
                .block(block);

            frame.render_stateful_widget(list, area, &mut self.list_state.clone());
        } else {
            let block = Block::bordered()
                .title("lazydb")
                .style(Style::new().fg(Color::Cyan))
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Thick);
            let paragraph = Paragraph::new(
                "
No connections are configured.
1. Create a config.toml file to define connections
2. Set the path to the file in the env var LAZYDB_CONFIG
3. Restart LazyDB
                ",
            )
            .block(block)
            .centered();
            frame.render_widget(paragraph, area);
        }
        Ok(())
    }
}
