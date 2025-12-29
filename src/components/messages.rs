use crate::{
    action::Action,
    app_event::{AppEvent, UserMessage},
    components::Component,
    config::Config,
};
use ratatui::{
    layout::Alignment,
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
            Action::ExecuteQuery(_) | Action::OpenDbConnection(_) => self.text = None,
            _ => {}
        }
        Ok(None)
    }

    fn handle_app_events(
        &mut self,
        event: crate::app_event::AppEvent,
    ) -> color_eyre::Result<Option<Action>> {
        if let AppEvent::UserMessage(msg) = event {
            match msg {
                UserMessage::Error(text) | UserMessage::Info(text) => self.text = Some(text),
            }
        };
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
