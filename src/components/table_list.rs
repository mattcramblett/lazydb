use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, List, ListState, Paragraph},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, components::Component, config::Config};

pub struct TableList {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    list_state: ListState,
}

impl Default for TableList {
    fn default() -> Self {
        Self {
            command_tx: Default::default(),
            config: Default::default(),
            list_state: ListState::default().with_selected(Some(0)),
        }
    }
}

impl Component for TableList {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> color_eyre::Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> color_eyre::Result<()> {
        self.config = config;
        Ok(())
    }

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::Result<()> {
        let items: Vec<String> = self.config.db_connections.0.clone().into_keys().collect();
        let block = Block::bordered()
            .title("tables [alt+3]")
            .style(Style::new().fg(Color::Cyan))
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Thick);

        if items.is_empty() {
            let paragraph = Paragraph::new("No tables found.").block(block);
            frame.render_widget(paragraph, area);
            return Ok(());
        }

        let list = List::new(items)
            .style(Color::Cyan)
            .highlight_style(Modifier::REVERSED)
            .highlight_symbol("▹ ")
            .block(block);

        frame.render_stateful_widget(list, area, &mut self.list_state.clone());
        Ok(())
    }
}
