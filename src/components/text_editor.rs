use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType},
};
use tokio::sync::mpsc::UnboundedSender;
use tui_textarea::TextArea;

use crate::{action::Action, app::Mode, components::Component, config::Config};

/// Text editor for SQL statements.
pub struct TextEditor<'a> {
    internal: TextArea<'a>,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    focused: bool,
}

impl<'a> Default for TextEditor<'a> {
    fn default() -> Self {
        let mut internal = TextArea::default();

        let style = Style::default().bg(Color::DarkGray).fg(Color::LightBlue);
        internal.set_line_number_style(style);

        Self {
            internal,
            command_tx: Default::default(),
            config: Default::default(),
            focused: true,
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
            Action::ChangeMode(Mode::EditQuery) => self.focused = true,
            Action::ChangeMode(_) => self.focused = false,
            _ => {}
        }
        Ok(None)
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Action>> {
        match key.code {
            // ctrl+r runs the query in the editor
            KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
                let query = self.internal.clone().into_lines().join("\n");
                Ok(Some(Action::ExecuteQuery(query)))
            }
            // any other key we accept as editor input, only if focused
            _ => {
                if self.focused {
                    self.internal.input(key);
                }
                Ok(None)
            }
        }
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
        let block = Block::bordered()
            .title("lazydb")
            .style(Style::new().fg(if self.focused {
                Color::Cyan
            } else {
                Color::Blue
            }))
            .title_alignment(Alignment::Center)
            .border_type(if self.focused {
                BorderType::Thick
            } else {
                BorderType::Plain
            });
        self.internal.set_block(block);

        let [top, _] =
            Layout::vertical([Constraint::Percentage(50), Constraint::Min(0)]).areas(area);
        frame.render_widget(&self.internal, top);
        Ok(())
    }
}
