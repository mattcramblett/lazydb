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
        let outer_layout =
            Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)])
                .split(root);
        let layout = Layout::vertical([
            Constraint::Percentage(45),
            Constraint::Percentage(45),
            Constraint::Percentage(10),
        ])
        .split(outer_layout[1]);
        vec![
            (ComponentId::TableList, outer_layout[0]),
            (ComponentId::TextEditor, layout[0]),
            (ComponentId::ResultsTable, layout[1]),
            (ComponentId::Messages, layout[2]),
        ]
    }
}
