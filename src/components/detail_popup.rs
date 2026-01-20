use ratatui::{
    layout::{Constraint, Flex, Layout},
    widgets::{Block, Clear, Paragraph},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, components::Component, config::Config};

#[derive(Default)]
pub struct DetailPopup {
    content: Option<String>,
    row_content: Option<(Vec<String>, Vec<String>)>,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
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
                self.content = None
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
            // TODO: dynamically compute size?
            let percent_x = 60;
            let percent_y = 20;
            let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
            let horizontal =
                Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
            let [area] = vertical.areas(area);
            let [area] = horizontal.areas(area);

            let block = Block::bordered().title("Value");
            let text = Paragraph::new(content.as_str()).block(block);
            frame.render_widget(Clear, area);
            frame.render_widget(text, area);
        } else if let Some((columns, row)) = &self.row_content {
            // TODO: use a list instead, with copying and scrolling functionality
            // - better format the column names with the values
            // - account for horizontal overflow
            let vertical = Layout::vertical([Constraint::Percentage(100)]).flex(Flex::Center);
            let horizontal = Layout::horizontal([Constraint::Percentage(60)]).flex(Flex::Center);
            let [area] = vertical.areas(area);
            let [area] = horizontal.areas(area);

            let block = Block::bordered().title("Row");
            let mut display = String::new();
            columns.iter().enumerate().for_each(|(idx, col)| {
                display += format!(
                    "{}: {}\n",
                    col,
                    row.get(idx)
                        .unwrap_or(&String::from("<ERROR GETTING VALUE>"))
                )
                .as_str();
            });
            let text = Paragraph::new(display.as_str()).block(block);
            frame.render_widget(Clear, area);
            frame.render_widget(text, area);
        }
        Ok(())
    }
}
