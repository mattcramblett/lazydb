use ratatui::layout::{Constraint, Layout, Rect};

use crate::app::{ComponentId, Mode};

#[derive(Clone, Default)]
pub struct AppRenderPlan {}

impl AppRenderPlan {
    pub fn compute_layouts(&self, mode: Mode, root: Rect) -> Vec<(ComponentId, Rect)> {
        if mode == Mode::ConnectionMenu {
            let layout = Layout::vertical([
                Constraint::Min(10),
                Constraint::Fill(40),
                Constraint::Min(5),
            ])
            .split(root);
            return vec![
                (ComponentId::Title, layout[0]),
                (ComponentId::ConnectionMenu, layout[1]),
                (ComponentId::Messages, layout[2]),
            ];
        }
        let layout = Layout::vertical([
            Constraint::Percentage(10),
            Constraint::Percentage(40),
            Constraint::Percentage(40),
            Constraint::Percentage(10),
        ])
        .split(root);
        vec![
            (ComponentId::TableList, layout[0]),
            (ComponentId::TextEditor, layout[1]),
            (ComponentId::ResultsTable, layout[2]),
            (ComponentId::Messages, layout[3]),
        ]
    }
}
