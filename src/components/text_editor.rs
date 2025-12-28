use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, BorderType},
};
use tokio::sync::mpsc::UnboundedSender;
use tui_textarea::TextArea;

use crate::{action::Action, components::Component, config::Config};

/// Text editor for SQL statements.
pub struct TextEditor<'a> {
    internal: TextArea<'a>,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
}

impl<'a> Default for TextEditor<'a> {
    fn default() -> Self {
        let mut internal = TextArea::default();

        let style = Style::default().bg(Color::DarkGray).fg(Color::LightBlue);
        internal.set_line_number_style(style);

        let block = Block::bordered()
            .title("lazydb")
            .style(Style::new().fg(Color::LightBlue))
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Thick);
        internal.set_block(block);

        Self {
            internal,
            command_tx: Default::default(),
            config: Default::default(),
        }
    }
}

impl<'a> TextEditor<'a> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for TextEditor<'_> {
    fn update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
        match action {
            Action::KeyPress(key_event) => {
                self.internal.input(key_event);
            },
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

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::Result<()> {
        frame.render_widget(&self.internal, area);
        Ok(())
    }
}
