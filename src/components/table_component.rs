use crate::{action::Action, app_event::AppEvent, components::{Component, data_table::DataTable}};

pub trait TableComponent: Component {
    /// Accessor to internal table
    fn table(&mut self) -> &mut DataTable;

    fn default_set_focus(&mut self, focused: bool) -> color_eyre::Result<()> {
        self.table().focused = focused;
        Ok(())
    }

    fn default_update(&mut self, action: Action) -> color_eyre::Result<Option<AppEvent>> {
        match action {
            Action::NavDown => self.table().state.select_next(),
            Action::NavUp => self.table().state.select_previous(),
            Action::NavLeft => self.table().state.select_previous_column(),
            Action::NavRight => self.table().state.select_next_column(),
            Action::Clear => self.table().clear_selection(),
            Action::Yank => self.table().yank_selection()?,
            Action::MakeSelection => {
                if let Some(selection) = self.table().cell_selection() {
                    return Ok(Some(AppEvent::CellSelected(selection)));
                }
                if let Some(row_selection) = self.table().row_selection() {
                    return Ok(Some(AppEvent::RowSelected(
                        self.table().columns.clone(),
                        row_selection,
                    )));
                }
            }
            Action::PageLeft => self.table().scroll_left(),
            Action::PageRight => self.table().scroll_right(),
            _ => {}
        }
        Ok(None)
    }
}
