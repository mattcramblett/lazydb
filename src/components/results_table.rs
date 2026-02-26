use arboard::Clipboard;
use ratatui::{
    layout::Alignment,
    prelude::Rect,
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Cell, Row, Table, TableState},
};
use unicode_width::UnicodeWidthStr;

use crate::{
    action::Action,
    app::Mode,
    app_event::{AppEvent, QueryTag},
    components::Component,
    config::Config,
};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct ResultsTable {
    /// Column names
    columns: Vec<String>,
    /// Result rows
    rows: Vec<Vec<String>>,
    /// Widths to render for each column
    widths: Vec<u16>,
    /// Table state determining selections, etc.
    state: TableState,
    /// Whether this table is in focus
    focused: bool,
    /// Column paging offset (scrolling columns left to right)
    column_offset: usize,
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
            focused: false,
            column_offset: 0,
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
            Action::NavDown if self.focused => self.state.select_next(),
            Action::NavUp if self.focused => self.state.select_previous(),
            Action::NavLeft if self.focused => self.state.select_previous_column(),
            Action::NavRight if self.focused => self.state.select_next_column(),
            Action::ChangeMode(Mode::ExploreResults) => self.focused = true,
            Action::ChangeMode(_) => self.focused = false,
            Action::Clear if self.focused => {
                if let Some(selection) = self.state.selected_cell() {
                    // Clear the cell selection, but retain the row selection
                    self.state.select_cell(None);
                    self.state.select(Some(selection.0));
                } else if self.state.selected().is_some() {
                    self.state.select(None);
                }
            }
            Action::Yank => {
                if let Ok(clipboard) = Clipboard::new() {
                    let mut clip = clipboard;
                    if let Some((idx, col)) = self.state.selected_cell()
                        && let Some(row) = self.rows.get(idx)
                        && let Some(val) = row.get(col)
                    {
                        clip.set_text(val)? // copy cell value
                    } else if let Some(idx) = self.state.selected()
                        && let Some(row) = self.rows.get(idx)
                    {
                        clip.set_text(row.join(" "))?
                    }
                }
            }
            Action::MakeSelection if self.focused => {
                if let Some(selection) = self.cell_selection() {
                    return Ok(Some(Action::SelectCell(selection)));
                }
                if let Some(row_selection) = self.row_selection() {
                    return Ok(Some(Action::SelectRow(self.columns.clone(), row_selection)));
                }
            }
            Action::PageLeft if self.focused => {
                if self.column_offset > 0 {
                    self.column_offset -= 1;
                }
            }
            Action::PageRight if self.focused => {
                if self.column_offset < self.columns.len() - 1 {
                    self.column_offset += 1;
                }
            }
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
        // Clip the number of displayed columns based on calculated widths and visible space
        let mut visible_cols = 0;
        let mut visible_width = 0;
        for width in self.widths[self.column_offset..].iter() {
            // stop counting when columns will overflow, leaving some buffer for space between cols
            if visible_width >= f32::round(area.width as f32 * 0.7) as u16 {
                break;
            }
            visible_width += width;
            visible_cols += 1;
        }
        let col_range = self.column_offset..(visible_cols + self.column_offset);

        let column_names = self.columns[col_range.clone()]
            .iter()
            .map(|c| Cell::from(c.as_str()));
        let header_bg_color = if self.columns.is_empty() {
            Color::Reset
        } else {
            Color::Rgb(18, 18, 18)
        };
        let header = Row::new(column_names)
            .style(Style::new().bold().bg(header_bg_color))
            .bottom_margin(1);

        let table_rows = self.rows.iter().enumerate().map(|(idx, r)| {
            // alternate row colors
            let color = if idx % 2 == 0 {
                Color::Rgb(30, 30, 30)
            } else {
                Color::Reset
            };
            Row::new(r[col_range.clone()].iter().map(|val| {
                if val.is_empty() {
                    Cell::from("EMPTY").fg(Color::Rgb(44, 44, 44))
                } else {
                    Cell::from(val.as_str())
                }
            }))
            .style(Style::default().bg(color))
        });

        let widths = self.widths[col_range.clone()].iter();

        let table = Table::default()
            .rows(table_rows)
            .widths(widths.cloned())
            .header(header)
            .block(self.make_block())
            .column_spacing(2)
            .style(Color::Blue)
            .row_highlight_style(Style::new().on_dark_gray().bold())
            .column_highlight_style(Color::Gray)
            .cell_highlight_style(Style::new().reversed().yellow())
            .highlight_symbol("▷ ");

        frame.render_stateful_widget(table, area, &mut self.state);
        Ok(())
    }
}

impl ResultsTable {
    fn set_data(&mut self, new_cols: Vec<String>, new_rows: Vec<Vec<String>>) {
        self.columns = new_cols;
        self.rows = new_rows;
        self.state = TableState::default();
        self.widths = self.calc_widths();
        self.column_offset = 0;
    }

    fn make_block<'a>(&self) -> Block<'a> {
        let left_arrow = if self.column_offset > 0 { "<<" } else { "" };
        let right_arrow = if !self.columns.is_empty() && self.column_offset < self.columns.len() - 1
        {
            ">>"
        } else {
            ""
        };

        Block::bordered()
            .title("results [alt+3]")
            .title_bottom(format!("{}   {}", left_arrow, right_arrow))
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

    fn cell_selection(&self) -> Option<String> {
        if let Some((row_idx, col_idx)) = self.state.selected_cell()
            && let Some(row) = self.rows.get(row_idx)
        {
            return row.get(col_idx).cloned();
        }
        None
    }

    fn row_selection(&self) -> Option<Vec<String>> {
        if let Some(index) = self.state.selected() {
            return self.rows.get(index).cloned();
        }
        None
    }

    fn calc_widths(&self) -> Vec<u16> {
        let max = 120u16;
        self.columns
            .iter()
            .map(|c| std::cmp::min(max, (c.width()) as u16))
            .collect()
    }
}
