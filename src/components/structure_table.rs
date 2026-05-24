use arboard::Clipboard;
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
pub struct StructureTable {
    /// Internal table widget
    data_table: DataTable,
    /// Name of table being displayed, if any
    table_name: Option<String>,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
}

impl Default for StructureTable {
    fn default() -> Self {
        let mut data_table = DataTable::default();
        data_table.title = "Structure [alt+4]".to_string();

        let mut def = Self {
            data_table,
            table_name: Default::default(),
            command_tx: Default::default(),
            config: Default::default(),
        };
        def.set_data(vec![], vec![]);
        def
    }
}

impl Component for StructureTable {
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
            Action::Yank => {
                if let Ok(clipboard) = Clipboard::new() {
                    let mut clip = clipboard;
                    if let Some((idx, col)) = self.data_table.state.selected_cell()
                        && let Some(row) = self.data_table.rows.get(idx)
                        && let Some(val) = row.get(col)
                    {
                        clip.set_text(val.clone().unwrap_or("NULL".to_string()))? // copy cell value
                    } else if let Some(idx) = self.data_table.state.selected()
                        && let Some(row) = self.data_table.rows.get(idx)
                    {
                        let row_str: String = row
                            .iter()
                            .map(|v| v.clone().unwrap_or(String::from("NULL")))
                            .collect::<Vec<String>>()
                            .join(" ");
                        clip.set_text(row_str)?
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_app_events(
        &mut self,
        event: crate::app_event::AppEvent,
    ) -> color_eyre::Result<Option<AppEvent>> {
        if let AppEvent::QueryResultReturned(result, QueryTag::TableStructure(table)) = event {
            self.table_name = Some(table.name);
            self.set_data(result.columns, result.rows);
            return Ok(Some(AppEvent::ModeSwitched(Mode::ExploreStructure)));
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
        self.data_table.title = format!(
            "{} [alt+4]",
            self.table_name
                .clone()
                .unwrap_or("Select a table and press 's'".to_string())
        );
        self.data_table.draw(frame, area)?;
        Ok(())
    }
}

impl StructureTable {
    fn set_data(&mut self, new_cols: Vec<String>, new_rows: Vec<Vec<Option<String>>>) {
        self.data_table.set_data(new_cols, new_rows);
    }
}
