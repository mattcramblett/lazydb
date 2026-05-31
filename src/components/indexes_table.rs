use ratatui::prelude::Rect;

use crate::{
    action::Action,
    app::Mode,
    app_event::{AppEvent, QueryTag},
    components::Component,
    config::Config,
    widgets::data_table::DataTable,
};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct IndexesTable {
    /// Internal table widget
    data_table: DataTable,
    /// Name of table being displayed, if any
    table_name: Option<String>,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
}

impl Default for IndexesTable {
    fn default() -> Self {
        let mut data_table = DataTable::default();
        data_table.title = "Indexes [alt+5]".to_string();

        Self {
            data_table,
            table_name: Default::default(),
            command_tx: Default::default(),
            config: Default::default(),
        }
    }
}

impl Component for IndexesTable {
    fn set_focus(&mut self, focused: bool) -> color_eyre::Result<()> {
        self.data_table.focused = focused;
        Ok(())
    }

    fn update(&mut self, action: Action) -> color_eyre::Result<Option<AppEvent>> {
        match action {
            Action::NavDown => self.data_table.state.select_next(),
            Action::NavUp => self.data_table.state.select_previous(),
            Action::NavLeft => self.data_table.state.select_previous_column(),
            Action::NavRight => self.data_table.state.select_next_column(),
            Action::Yank => self.data_table.yank_selection()?,
            Action::Clear => self.data_table.clear_selection(),
            Action::MakeSelection => {
                if let Some(selection) = self.data_table.cell_selection() {
                    return Ok(Some(AppEvent::CellSelected(selection)));
                }
                if let Some(row_selection) = self.data_table.row_selection() {
                    return Ok(Some(AppEvent::RowSelected(
                        self.data_table.columns.clone(),
                        row_selection,
                    )));
                }
            }
            Action::PageLeft => self.data_table.scroll_left(),
            Action::PageRight => self.data_table.scroll_right(),
            _ => {}
        }
        Ok(None)
    }

    fn handle_app_events(
        &mut self,
        event: crate::app_event::AppEvent,
    ) -> color_eyre::Result<Option<AppEvent>> {
        if let AppEvent::QueryResultReturned(result, QueryTag::TableIndexes(table)) = event {
            self.table_name = Some(table.name);
            self.data_table.set_data(result.columns, result.rows);
            return Ok(Some(AppEvent::ModeSwitched(Mode::ExploreIndexes)));
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
        // TODO: make placeholder text dynamic based on Config
        self.data_table.title = format!(
            "Indexes - {} [alt+5]",
            self.table_name
                .clone()
                .unwrap_or("Select a table and press 'i'".to_string())
        );
        self.data_table.draw(frame, area)?;
        Ok(())
    }
}
