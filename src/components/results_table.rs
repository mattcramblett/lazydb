use ratatui::prelude::Rect;

use crate::{
    action::Action,
    app_event::{AppEvent, QueryTag},
    components::Component,
    config::Config,
    widgets::data_table::DataTable,
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

impl Component for ResultsTable {
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
            Action::Clear => self.data_table.clear_selection(),
            Action::Yank => self.data_table.yank_selection()?,
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
