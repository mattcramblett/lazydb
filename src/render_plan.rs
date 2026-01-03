use ratatui::layout::{Constraint, Layout, Rect};

use crate::app::{ComponentId, Mode};

#[derive(Clone, Default)]
pub struct RenderPlan {}

impl RenderPlan {
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

        let visible_table = match mode {
            Mode::ExploreStructure => ComponentId::StructureTable,
            _ => ComponentId::ResultsTable
        };

        // Default:
        let outer_layout =
            Layout::horizontal([Constraint::Min(20), Constraint::Percentage(80)]).split(root);
        let inner_layout = Layout::vertical([
            Constraint::Percentage(45),
            Constraint::Percentage(45),
            Constraint::Percentage(10),
        ])
        .split(outer_layout[1]);
        vec![
            (ComponentId::TableList, outer_layout[0]),
            (ComponentId::TextEditor, inner_layout[0]),
            (visible_table, inner_layout[1]),
            (ComponentId::Messages, inner_layout[2]),
        ]
    }
}
