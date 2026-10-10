use ratatui::prelude::Rect;

use crate::{
    action::Action,
    app_event::{AppEvent, QueryTag},
    components::{Component, data_table::DataTable, table_component::TableComponent},
    config::Config,
};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct ResultsTable {
    data_table: DataTable,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
}

impl Default for ResultsTable {
    fn default() -> Self {
        let mut data_table = DataTable::default();
        data_table.title = "Results [alt+3]".to_string();

        Self {
            data_table,
            command_tx: Default::default(),
            config: Default::default(),
        }
    }
}

impl TableComponent for ResultsTable {
    fn table(&mut self) -> &mut DataTable {
        &mut self.data_table
    }
}

impl Component for ResultsTable {
    fn update(&mut self, action: Action) -> color_eyre::Result<Option<AppEvent>> {
        self.default_update(action)
    }

    fn set_focus(&mut self, focused: bool) -> color_eyre::Result<()> {
        self.default_set_focus(focused)
    }

    fn handle_app_events(
        &mut self,
        event: crate::app_event::AppEvent,
    ) -> color_eyre::Result<Option<AppEvent>> {
        match event {
            AppEvent::QueryResultReturned(result, QueryTag::User)
            | AppEvent::QueryResultReturned(result, QueryTag::InitialTable(_)) => {
                self.data_table.set_data(result.columns, result.rows);
            }
            _ => {}
        }
        Ok(None)
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> color_eyre::Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> color_eyre::Result<()> {
        self.config = config;
        Ok(())
    }

    fn draw(&mut self, frame: &mut ratatui::Frame, area: Rect) -> color_eyre::Result<()> {
        self.data_table.draw(frame, area)?;
        Ok(())
    }
}
