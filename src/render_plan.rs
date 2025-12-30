use std::collections::HashMap;

use ratatui::layout::Constraint;

use crate::app::{ComponentId, Mode};

#[derive(Default, Clone)]
pub struct ModeRenderPlan {
    pub constraints: Vec<Constraint>,
    pub component_ids: Vec<ComponentId>,
}

#[derive(Clone)]
pub struct AppRenderPlan {
    /// Mapping of the app Mode to its render plan
    plans: HashMap<Mode, ModeRenderPlan>,
    /// The fallback option if there is not a specified plan for the given mode
    default: ModeRenderPlan,
}

impl Default for AppRenderPlan {
    fn default() -> Self {
        let mut plans = HashMap::new();
        plans.insert(
            Mode::ConnectionMenu, 
            ModeRenderPlan {
                constraints: vec![Constraint::Percentage(90), Constraint::Percentage(10)],
                component_ids: vec![ComponentId::ConnectionMenu, ComponentId::Messages],
            }
        );

        let default = ModeRenderPlan {
            constraints: vec![
                Constraint::Percentage(45),
                Constraint::Percentage(45),
                Constraint::Percentage(10),
            ],
            component_ids: vec![
                ComponentId::TextEditor,
                ComponentId::ResultsTable,
                ComponentId::Messages,
            ]
        };

        Self { plans, default }
    }
}

impl AppRenderPlan {
    /// Given a Mode of the app, returns the definition of what should be rendered.
    pub fn get_plan(&self, mode: Mode) -> ModeRenderPlan {
        if let Some(plan) = self.plans.get(&mode) {
            plan.clone()
        } else {
            self.default.clone()
        }
    }
}
