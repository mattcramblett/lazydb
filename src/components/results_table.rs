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
    fn update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
        match action {
            Action::NavDown if self.data_table.focused => self.data_table.state.select_next(),
            Action::NavUp if self.data_table.focused => self.data_table.state.select_previous(),
            Action::NavLeft if self.data_table.focused => {
                self.data_table.state.select_previous_column()
            }
            Action::NavRight if self.data_table.focused => {
                self.data_table.state.select_next_column()
            }
            Action::ChangeMode(Mode::ExploreResults) => {
                self.data_table.focused = true;
                self.data_table.focused = true;
            }
            Action::ChangeMode(_) => {
                self.data_table.focused = false;
                self.data_table.focused = false;
            }
            Action::Clear if self.data_table.focused => self.data_table.clear_selection(),
            Action::Yank => self.data_table.yank_selection()?,
            Action::MakeSelection if self.data_table.focused => {
                if let Some(selection) = self.data_table.cell_selection() {
                    return Ok(Some(Action::SelectCell(selection)));
                }
                if let Some(row_selection) = self.data_table.row_selection() {
                    return Ok(Some(Action::SelectRow(
                        self.data_table.columns.clone(),
                        row_selection,
                    )));
                }
            }
            Action::PageLeft if self.data_table.focused => self.data_table.scroll_left(),
            Action::PageRight if self.data_table.focused => self.data_table.scroll_right(),
            _ => {}
        }
        Ok(None)
    }

    fn handle_app_events(
        &mut self,
        event: crate::app_event::AppEvent,
    ) -> color_eyre::Result<Option<Action>> {
        match event {
            AppEvent::QueryResult(result, QueryTag::User)
            | AppEvent::QueryResult(result, QueryTag::InitialTable(_)) => {
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
