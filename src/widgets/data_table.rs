use arboard::Clipboard;
use ratatui::{
    layout::{Alignment, Constraint},
    prelude::Rect,
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Cell, Row, Table, TableState},
};
use unicode_width::UnicodeWidthStr;

#[derive(Debug)]
pub struct DataTable {
    /// Table state determining selections, etc.
    pub state: TableState,
    /// Column names
    pub columns: Vec<String>,
    /// Result rows
    pub rows: Vec<Vec<Option<String>>>,
    /// Display title
    pub title: String,
    /// Whether this table is in focus
    pub focused: bool,
    /// Widths to render for each column
    widths: Vec<u16>,
    /// Column paging offset (scrolling columns left to right)
    column_offset: usize,
}

impl Default for DataTable {
    fn default() -> Self {
        let mut def = Self {
            columns: Default::default(),
            rows: Default::default(),
            widths: Default::default(),
            state: Default::default(),
            title: Default::default(),
            focused: false,
            column_offset: 0,
        };
        def.set_data(vec![], vec![]);
        def
    }
}

impl DataTable {
    pub fn draw(&mut self, frame: &mut ratatui::Frame, area: Rect) -> color_eyre::Result<()> {
        // Clip the number of displayed columns based on calculated widths and visible space
        let mut visible_cols = 0;
        let mut visible_width = 0;
        let column_space = 2;
        for width in self.widths[self.column_offset..].iter() {
            // stop counting when columns will overflow, with additional buffer
            if visible_width >= area.width {
                break;
            }
            visible_width += width + column_space * 2;
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
                if let Some(row_val) = val {
                    if row_val.is_empty() {
                        Cell::from("EMPTY").fg(Color::Rgb(44, 44, 44))
                    } else {
                        Cell::from(row_val.as_str())
                    }
                } else {
                    Cell::from("NULL").fg(Color::Rgb(38, 38, 38))
                }
            }))
            .style(Style::default().bg(color))
        });

        let widths: Vec<Constraint> = if visible_cols == self.columns.len() {
            self.columns.iter().map(|_| Constraint::default()).collect()
        } else {
            self.widths[col_range.clone()].iter().map(|len| Constraint::Length(*len)).collect()
        };

        let table = Table::default()
            .rows(table_rows)
            .widths(widths)
            .header(header)
            .block(self.make_block())
            .column_spacing(column_space)
            .style(Color::Blue)
            .row_highlight_style(Style::new().on_dark_gray().bold())
            .cell_highlight_style(Style::new().reversed().yellow())
            .highlight_symbol("▷");

        frame.render_stateful_widget(table, area, &mut self.state);
        Ok(())
    }

    pub fn set_data(&mut self, new_cols: Vec<String>, new_rows: Vec<Vec<Option<String>>>) {
        self.columns = new_cols;
        self.rows = new_rows;
        self.state = TableState::default();
        self.widths = self.calc_widths();
        self.column_offset = 0;
    }

    pub fn yank_selection(&mut self) -> color_eyre::Result<()> {
        if let Ok(clipboard) = Clipboard::new() {
            let mut clip = clipboard;
            if let Some((idx, col)) = self.state.selected_cell()
                && let Some(row) = self.rows.get(idx)
                && let Some(val) = row.get(col)
            {
                clip.set_text(val.clone().unwrap_or(String::from("NULL")))?; // copy cell value
            } else if let Some(idx) = self.state.selected()
                && let Some(row) = self.rows.get(idx)
            {
                let row_str: String = row
                    .iter()
                    .map(|v| v.clone().unwrap_or(String::from("NULL")))
                    .collect::<Vec<String>>()
                    .join(" ");
                clip.set_text(row_str)?;
            }
        }
        Ok(())
    }

    pub fn clear_selection(&mut self) {
        if let Some(selection) = self.state.selected_cell() {
            // Clear the cell selection, but retain the row selection
            self.state.select_cell(None);
            self.state.select(Some(selection.0));
        } else if self.state.selected().is_some() {
            self.state.select(None);
        }
    }

    pub fn cell_selection(&self) -> Option<String> {
        if let Some((row_idx, col_idx)) = self.state.selected_cell()
            && let Some(row) = self.rows.get(row_idx)
            && let Some(cell) = row.get(col_idx)
        {
            return cell.clone();
        }
        None
    }

    pub fn row_selection(&self) -> Option<Vec<Option<String>>> {
        if let Some(index) = self.state.selected() {
            return self.rows.get(index).cloned();
        }
        None
    }

    pub fn scroll_left(&mut self) {
        if self.column_offset > 0 {
            self.column_offset -= 1;
        }
    }

    pub fn scroll_right(&mut self) {
        if self.column_offset < self.columns.len() - 1 {
            self.column_offset += 1;
        }
    }

    fn calc_widths(&self) -> Vec<u16> {
        let max = 120u16;
        self.columns
            .iter()
            .map(|c| std::cmp::min(max, (c.width()) as u16))
            .collect()
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
            .title(self.title.clone())
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
}
