use crate::{
    action::Action,
    app_event::{AppEvent, MessageType, QueryTag},
    components::Component,
    config::Config,
};
use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, BorderType, List, ListState},
};
use tokio::sync::mpsc::UnboundedSender;

/// Displays errors
#[derive(Default)]
pub struct Messages {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    message_history: Vec<(MessageType, String)>,
}

impl Component for Messages {
    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::Result<()> {
        let block = Block::bordered()
            .title("messages")
            .style(Style::new().fg(Color::Blue))
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Plain);

        let items = self.message_history.iter().map(|(msg_type, msg)| {
            let style = match msg_type {
                MessageType::Info => Color::Cyan,
                MessageType::Error => Color::Red,
                MessageType::Debug => Color::DarkGray,
            };
            Text::styled(msg, style)
        });

        let list = List::new(items)
            .highlight_style(Modifier::REVERSED)
            .block(block);
        let mut state = ListState::default();
        state.scroll_down_by(self.message_history.len().try_into().unwrap_or(0));

        frame.render_stateful_widget(list, area, &mut state);
        Ok(())
    }

    fn handle_app_events(
        &mut self,
        event: crate::app_event::AppEvent,
    ) -> color_eyre::Result<Option<AppEvent>> {
        match event {
            AppEvent::QueryExecutionRequested(query) if query.tag == QueryTag::User => {
                self.add_message(MessageType::Info, "Running...".to_string())
            }
            AppEvent::UserMessage(msg_type, msg) => self.add_message(msg_type, msg),
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

impl Messages {
    pub fn add_message(&mut self, message_type: MessageType, message: String) {
        if self.message_history.len() > 100 {
            self.message_history.remove(0);
        }
        self.message_history.push((message_type, message));
    }
}
