use ratatui::widgets::{Block, Paragraph};

use crate::components::Component;

#[derive(Default)]
pub struct Title {}

impl Component for Title {
    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::Result<()> {
        let block = Block::new();
        let paragraph = Paragraph::new(String::from(
            "

██╗      █████╗ ███████╗██╗   ██╗██████╗ ██████╗ 
██║     ██╔══██╗╚══███╔╝╚██╗ ██╔╝██╔══██╗██╔══██╗
██║     ███████║  ███╔╝  ╚████╔╝ ██║  ██║██████╔╝
██║     ██╔══██║ ███╔╝    ╚██╔╝  ██║  ██║██╔══██╗
███████╗██║  ██║███████╗   ██║   ██████╔╝██████╔╝
╚══════╝╚═╝  ╚═╝╚══════╝   ╚═╝   ╚═════╝ ╚═════╝ 
                                                 
",
        ))
        .block(block).centered();
        frame.render_widget(paragraph, area);
        Ok(())
    }
}
