use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{action::Action, app::Mode, config::Config};

pub struct ConnectionMenu {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    list_state: ListState,
    focused: bool,
}

impl Default for ConnectionMenu {
    fn default() -> Self {
        Self {
            command_tx: Default::default(),
            config: Default::default(),
            list_state: ListState::default().with_selected(Some(0)),
            focused: true, // NOTE: this is the first pane used in app startup, so focus it
        }
    }
}

impl ConnectionMenu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn items(&self) -> Vec<String> {
        self.config.db_connections.0.clone().into_keys().collect()
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
            Action::MakeSelection => {
                if let Some(idx) = self.list_state.selected() && self.focused
                    && let Some(connection_name) = self.items().get(idx)
                {
                    return Ok(Some(Action::OpenDbConnection(connection_name.to_string())));
                }
            }
            Action::NavDown if self.focused => {
                // protect against excess navigation
                if let Some(selected) = self.list_state.selected()
                    && !self.items().is_empty()
                    && selected >= self.items().len() - 1
                {
                    return Ok(None);
                }
                self.list_state.select_next()
            }
            Action::NavUp if self.focused => self.list_state.select_previous(),
            Action::ChangeMode(Mode::ConnectionMenu) => self.focused = true,
            Action::ChangeMode(_) => self.focused = false,
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        let items: Vec<String> = self.items();
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
1. Create a config.yaml file to define connections
2. Set the absolute path to the file in the env var LAZYDB_CONFIG
3. Restart LazyDB",
            )
            .block(block)
            .centered();
            frame.render_widget(paragraph, area);
        }
        Ok(())
    }
}
