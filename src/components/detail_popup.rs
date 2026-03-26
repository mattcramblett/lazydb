use ratatui::{
    layout::{Constraint, Flex, Layout},
    style::{Color, Stylize},
    text::Text,
    widgets::{Block, Clear, List, ListItem, ListState, Paragraph},
};
use tokio::sync::mpsc::UnboundedSender;
use unicode_width::UnicodeWidthStr;

use crate::{action::Action, components::Component, config::Config};

#[derive(Default)]
pub struct DetailPopup {
    content: Option<String>,
    row_content: Option<(Vec<String>, Vec<Option<String>>)>,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    list_state: ListState,
}

impl Component for DetailPopup {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> color_eyre::Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> color_eyre::Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(
        &mut self,
        action: crate::action::Action,
    ) -> color_eyre::Result<Option<crate::action::Action>> {
        match action {
            Action::SelectRow(columns, row) => {
                self.content = None;
                self.row_content = Some((columns, row));
            }
            Action::SelectCell(content) => {
                self.row_content = None;
                self.content = Some(content);
            }
            Action::Clear => {
                self.row_content = None;
                self.content = None;
                self.list_state = ListState::default().with_selected(Some(0));
            }
            Action::NavDown if self.is_focused() => {
                self.list_state.select_next();
            }
            Action::NavUp if self.is_focused() => {
                self.list_state.select_previous();
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::Result<()> {
        if let Some(content) = &self.content {
            let percent_y = 20;
            let width = content.width() as u16 + 4;
            let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
            let horizontal = Layout::horizontal([Constraint::Length(width)]).flex(Flex::Center);
            let [area] = vertical.areas(area);
            let [area] = horizontal.areas(area);

            let block = Block::bordered().title("Value").style(Color::Cyan);
            let text = Paragraph::new(content.as_str()).block(block);
            frame.render_widget(Clear, area);
            frame.render_widget(text, area);
        } else if let Some((columns, row)) = &self.row_content {
            let vertical =
                Layout::vertical([Constraint::Length(columns.len() as u16 + 2)]).flex(Flex::Center);
            let horizontal = Layout::horizontal([Constraint::Percentage(65)]).flex(Flex::Center);
            let [area] = vertical.areas(area);
            let [area] = horizontal.areas(area);

            let block = Block::bordered().title("Row").style(Color::Cyan);
            let items = columns.iter().enumerate().map(|(idx, col)| {
                let error_display = Some(String::from("<ERROR GETTING VALUE>"));
                let val = row.get(idx).unwrap_or(&error_display);
                let display_val = if val.is_some() {
                    val.clone().unwrap().clone().white()
                } else {
                    String::from("NULL").dark_gray()
                };

                let mut text = Text::default();
                text.push_span(col.clone().bold().cyan());
                text.push_span(": ");
                text.push_span(display_val);
                ListItem::new(text)
            });

            let list = List::new(items).block(block);
            frame.render_widget(Clear, area);
            frame.render_stateful_widget(list, area, &mut self.list_state.clone());
        }
        Ok(())
    }
}

impl DetailPopup {
    fn is_focused(&self) -> bool {
        self.content.is_some() || self.row_content.is_some()
    }
}
