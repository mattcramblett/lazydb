use ratatui::{
    layout::{Alignment, Constraint},
    prelude::Rect,
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Cell, Row, StatefulWidget, Table, TableState},
};

use crate::{
    action::Action, app::Mode, app_event::AppEvent, components::Component, config::Config,
};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct ResultsTable {
    /// Column names
    columns: Vec<String>,
    /// Result rows
    rows: Vec<Vec<String>>,
    /// Widths to render for each column
    widths: Vec<Constraint>,
    /// Internal table component being wrapped
    internal: Table<'static>,
    /// Table state determining selections, etc.
    state: TableState,
    /// Whether this table is in focus
    focused: bool,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
}

impl Default for ResultsTable {
    fn default() -> Self {
        let mut def = Self {
            columns: Default::default(),
            rows: Default::default(),
            widths: Default::default(), // TODO: calculate column widths
            state: Default::default(),
            internal: Default::default(),
            focused: false,
            command_tx: Default::default(),
            config: Default::default(),
        };
        def.set_data(vec![], vec![]);
        def
    }
}

impl Component for ResultsTable {
    fn update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
        match action {
            Action::NavDown => self.state.select_next(),
            Action::NavUp => self.state.select_previous(),
            Action::NavLeft => self.state.select_previous_column(),
            Action::NavRight => self.state.select_next_column(),
            Action::ChangeMode(Mode::ExploreResults) => self.update_focused(true),
            Action::ChangeMode(_) => self.update_focused(false),
            _ => {}
        }
        Ok(None)
    }

    fn handle_app_events(
        &mut self,
        event: crate::app_event::AppEvent,
    ) -> color_eyre::Result<Option<Action>> {
        match event {
            AppEvent::QueryResult(result) => {
                self.set_data(result.columns, result.rows);
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
        let table = &self.internal;
        table.render(area, frame.buffer_mut(), &mut self.state.clone());
        Ok(())
    }
}

impl ResultsTable {
    /// Update the Block style when changing focus. This normally would be done inside `draw`, but
    /// that would require cloning the internal Table. For performance reasons, avoid cloning
    /// unless it's really necessary. This is a bit of a smell and may need revisited.
    fn update_focused(&mut self, focused: bool) {
        self.focused = focused;
        let block = self.make_block();
        self.internal = self.internal.clone().block(block);
    }

    /// Update the internal result data for display, and rebuild the table.
    fn set_data(&mut self, new_cols: Vec<String>, new_rows: Vec<Vec<String>>) {
        self.columns = new_cols;
        self.rows = new_rows;
        self.state = TableState::default();
        self.build_internal();
    }

    /// Internally rebuild the underlying table component. Do this only when data changes for
    /// performance.
    fn build_internal(&mut self) {
        let header = Row::new(self.columns.iter().map(|c| Cell::from(c.clone())))
            .style(Style::new().bold())
            .bottom_margin(1);

        // Build a Table<'static> from owned Strings by cloning the strings into the cells.
        // This makes the Table own its text, so it doesn't borrow from temporary values.
        let table_rows = self
            .rows
            .iter()
            .map(|r| Row::new(r.iter().map(|val| Cell::from(val.clone()))));

        let table = Table::new(table_rows, self.widths.clone())
            .header(header)
            .block(self.make_block())
            .column_spacing(1)
            .style(Color::Blue)
            .row_highlight_style(Style::new().on_black().bold())
            .column_highlight_style(Color::Gray)
            .cell_highlight_style(Style::new().reversed().yellow())
            .highlight_symbol("▷ ");

        self.state = TableState::default();
        self.internal = table;
    }

    fn make_block<'a>(&self) -> Block<'a> {
        Block::bordered()
            .title("results [f2]")
            .style(Style::new().fg(if self.focused {
                Color::Cyan
            } else {
                Color::Blue
            }))
            .title_alignment(Alignment::Center)
            .border_type(if self.focused {
                BorderType::Thick
            } else {
                BorderType::Plain
            })
    }
}
