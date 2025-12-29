use crate::{action::Action, components::Component, config::Config};
use ratatui::{
    layout::{Alignment},
    style::{Color, Style},
    widgets::{Block, BorderType, Paragraph},
};
use tokio::sync::mpsc::UnboundedSender;

/// Displays errors
#[derive(Default)]
pub struct Messages {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    text: Option<String>,
}

impl Component for Messages {
    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::Result<()> {
        if let Some(message) = self.text.clone() {
            let block = Block::bordered()
                .title("error")
                .style(Style::new().fg(Color::Red))
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Thick);
            let paragraph = Paragraph::new(message).block(block);
            frame.render_widget(paragraph, area);
        } else {
            let block = Block::bordered()
                .title("messages")
                .style(Style::new().fg(Color::Blue))
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Thick);
            let paragraph = Paragraph::new(String::default()).block(block);
            frame.render_widget(paragraph, area);
        }
        Ok(())
    }

    fn update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
        match action {
            Action::ExecuteQuery(_) => self.text = None,
            Action::DisplaySqlError(error_msg) => self.text = Some(error_msg),
            _ => {}
        }
        Ok(None)
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> color_eyre::Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> color_eyre::Result<()> {
        self.config = config;
        Ok(())
    }
}
