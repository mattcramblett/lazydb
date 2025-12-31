use crate::{
    action::Action,
    app_event::{AppEvent, MessageType},
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
    message: Option<(MessageType, String)>,
}

impl Component for Messages {
    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::Result<()> {
        if let Some((msg_type, message)) = self.message.clone() {
            let (title, color) = match msg_type {
                MessageType::Error => ("error", Color::Red),
                MessageType::Info => ("messages", Color::Cyan),
            };
            let block = Block::bordered()
                .title(title)
                .style(Style::new().fg(color))
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Thick);
            let paragraph = Paragraph::new(message).block(block);
            frame.render_widget(paragraph, area);
        } else {
            // empty state
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
            Action::ExecuteQuery(_, None) | Action::OpenDbConnection(_) => self.message = None,
            _ => {}
        }
        Ok(None)
    }

    fn handle_app_events(
        &mut self,
        event: crate::app_event::AppEvent,
    ) -> color_eyre::Result<Option<Action>> {
        if let AppEvent::UserMessage(msg_type, msg) = event {
            self.message = Some((msg_type, msg));
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
