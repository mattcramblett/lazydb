use ratatui::layout::{Constraint, Layout, Rect};

use crate::app::{ComponentId, Mode};

#[derive(Clone)]
pub struct RenderPlan {
    visible_table: ComponentId,
}

impl Default for RenderPlan {
    fn default() -> Self {
        Self {
            visible_table: ComponentId::ResultsTable,
        }
    }
}

impl RenderPlan {
    pub fn compute_layouts(
        &mut self,
        mode: Mode,
        zoom: bool,
        root: Rect,
    ) -> Vec<(ComponentId, Rect)> {
        self.visible_table = match mode {
            Mode::ExploreResults => ComponentId::ResultsTable,
            Mode::ExploreStructure => ComponentId::StructureTable,
            Mode::ExploreIndexes => ComponentId::IndexesTable,
            _ => self.visible_table,
        };

        match mode {
            // ConnectionMenu has a distinct layout, and no zooming.
            Mode::ConnectionMenu => {
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
            // Match against other modes when zoomed:
            Mode::ExploreTables if zoom => {
                let layout = Layout::horizontal(vec![Constraint::Fill(100)]).split(root);
                return vec![(ComponentId::TableList, layout[0])];
            }
            Mode::EditQuery if zoom => {
                let layout = Layout::horizontal(vec![Constraint::Fill(100)]).split(root);
                return vec![(ComponentId::TextEditor, layout[0])];
            }
            Mode::ExploreResults if zoom => {
                let layout = Layout::horizontal(vec![Constraint::Fill(100)]).split(root);
                return vec![(ComponentId::ResultsTable, layout[0])];
            }
            Mode::ExploreStructure if zoom => {
                let layout = Layout::horizontal(vec![Constraint::Fill(100)]).split(root);
                return vec![(ComponentId::StructureTable, layout[0])];
            }
            Mode::ExploreIndexes if zoom => {
                let layout = Layout::horizontal(vec![Constraint::Fill(100)]).split(root);
                return vec![(ComponentId::IndexesTable, layout[0])];
            }
            _ => {}
        }

        // Default, non-zoomed layout:
        let outer_layout =
            Layout::horizontal([Constraint::Min(20), Constraint::Percentage(80)]).split(root);
        let inner_layout = Layout::vertical([
            Constraint::Percentage(45),
            Constraint::Percentage(45),
            Constraint::Percentage(10),
        ])
        .split(outer_layout[1]);

        let sidebar_comp = if matches!(mode, Mode::ExploreSchemas) {
            ComponentId::SchemaList
        } else {
            ComponentId::TableList
        };

        vec![
            (sidebar_comp, outer_layout[0]),
            (ComponentId::TextEditor, inner_layout[0]),
            (self.visible_table, inner_layout[1]),
            (ComponentId::Messages, inner_layout[2]),
        ]
    }
}
