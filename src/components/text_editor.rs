use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, BorderType},
};
use tokio::sync::mpsc::UnboundedSender;
use tui_textarea::TextArea;

use crate::{
    action::Action, app::Mode, app_event::QueryTag, components::Component, config::Config,
    database::system_query::Query,
};

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
        internal.set_selection_style(Style::default().bg(Color::DarkGray));

        Self {
            internal,
            command_tx: Default::default(),
            config: Default::default(),
            focused: false,
        }
    }
}

impl<'a> TextEditor<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    fn query(&self) -> String {
        if let Some(selection_range) = self.internal.selection_range() {
            let ((first_line, first_line_char), (last_line, last_line_char)) = selection_range;
            let mut lines: Vec<String> = vec![];
            let iter = self.internal.clone().into_lines().into_iter().enumerate();

            for (idx, line) in iter {
                if idx == first_line && idx == last_line {
                    lines.push(line[first_line_char..last_line_char].to_string());
                } else if idx == first_line {
                    lines.push(line[first_line_char..line.len()].to_string());
                } else if idx == last_line {
                    lines.push(line[0..last_line_char].to_string());
                } else if (first_line..=last_line).contains(&idx) {
                    lines.push(line);
                }
            }
            lines.join("\n")
        } else {
            self.internal.clone().into_lines().join("\n")
        }
    }
}

impl Component for TextEditor<'_> {
    fn update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
        match action {
            Action::ChangeMode(Mode::EditQuery) => self.focused = true,
            Action::ChangeMode(_) => self.focused = false,
            Action::ExecuteQuery(Query {
                tag: QueryTag::InitialTable(_),
                query,
                ..
            }) => {
                self.internal.insert_newline();
                self.internal.insert_str(query);
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Action>> {
        // Only handle arbitrary key events if the editor is in focus
        if !self.focused {
            return Ok(None);
        }

        match key.code {
            // ctrl+r runs the query in the editor
            // TODO: make this keymap configurable
            KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
                Ok(Some(Action::ExecuteQuery(Query {
                    query: self.query(),
                    tag: QueryTag::User,
                    binds: None,
                })))
            }
            // any other key we accept as editor input
            _ => {
                self.internal.input(key);
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
            .title("lazydb [alt+2]")
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

        frame.render_widget(&self.internal, area);
        Ok(())
    }
}
