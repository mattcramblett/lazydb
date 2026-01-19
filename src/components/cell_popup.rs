use ratatui::{
    layout::{Constraint, Flex, Layout},
    widgets::{Block, Clear, Paragraph},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, components::Component, config::Config};

#[derive(Default)]
pub struct CellPopup {
    content: Option<String>,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
}

impl Component for CellPopup {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> color_eyre::Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> color_eyre::Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: crate::action::Action) -> color_eyre::Result<Option<crate::action::Action>> {
        match action {
            Action::SelectCell(content) => {
                self.content = Some(content);
            },
            Action::Clear => {
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

            let block = Block::bordered().title("Row value");
            let text = Paragraph::new(content.as_str()).block(block);
            frame.render_widget(Clear, area);
            frame.render_widget(text, area);
        }
        Ok(())
    }
}
