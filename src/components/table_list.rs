use arboard::Clipboard;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, List, ListState},
};
use tokio::sync::mpsc::UnboundedSender;
use tui_textarea::TextArea;

use crate::{
    action::Action,
    app::Mode,
    app_event::{AppEvent, QueryTag},
    components::Component,
    config::Config,
    database::system_query::SystemQuery,
};

pub struct TableList<'a> {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    list_state: ListState,
    focused: Option<FocusTarget>,
    items: Vec<String>,
    search_input: TextArea<'a>,
}

enum FocusTarget {
    List,
    Search,
}

impl<'a> Default for TableList<'a> {
    fn default() -> Self {
        let mut search_input = TextArea::default();
        search_input.set_placeholder_text("Search tables");
        Self {
            command_tx: Default::default(),
            config: Default::default(),
            list_state: ListState::default().with_selected(Some(0)),
            focused: None,
            items: Default::default(),
            search_input,
        }
    }
}

impl<'a> Component for TableList<'a> {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> color_eyre::Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> color_eyre::Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
        // Does not require focus:
        match action {
            Action::ChangeMode(Mode::ExploreTables) => self.focused = Some(FocusTarget::List),
            Action::ChangeMode(_) => self.focused = None,
            _ => {}
        }

        // Actions for search input focused:
        if let Some(FocusTarget::Search) = self.focused
            && action == Action::MakeSelection
        {
            // Key inputs for typing search criteria are in `handle_key_event`
            self.focused = Some(FocusTarget::List);
            return Ok(None);
        }

        // Actions for list focused:
        if let Some(FocusTarget::List) = self.focused {
            match action {
                Action::NavDown => {
                    // protect against excess navigation
                    if let Some(selected) = self.list_state.selected()
                        && !self.display_items().is_empty()
                        && selected >= self.display_items().len() - 1
                    {
                        return Ok(None);
                    }
                    self.list_state.select_next();
                }
                Action::NavUp => self.list_state.select_previous(),
                Action::MakeSelection => {
                    if let Some(selection) = self.selection() {
                        let table_name = selection.to_string();
                        return Ok(Some(Action::ExecuteQuery(
                            SystemQuery::query_for(QueryTag::InitialTable(table_name.clone()))?,
                        )));
                    }
                    return Ok(None);
                }
                Action::ViewStructure => {
                    if let Some(selection) = self.selection() {
                        let table_name = selection.to_string();
                        return Ok(Some(Action::ExecuteQuery(
                            SystemQuery::query_for(QueryTag::TableStructure(table_name.clone()))?,
                        )));
                    }
                    return Ok(None);
                }
                Action::Yank => {
                    if let Ok(clipboard) = Clipboard::new()
                        && let Some(selection) = self.selection()
                    {
                        let mut clip = clipboard;
                        clip.set_text(selection)?
                    }
                }
                Action::Search => {
                    let mut text_area = TextArea::default();
                    text_area.set_placeholder_text("Search tables");
                    self.focused = Some(FocusTarget::Search);
                }
                _ => {}
            }
        }

        // Applicable for any focus target
        if let Action::Clear = action
            && self.focused.is_some()
        {
            let mut text_area = TextArea::default();
            text_area.set_placeholder_text("Search tables");
            self.search_input = text_area;
            self.focused = Some(FocusTarget::List);
        }

        Ok(None)
    }

    fn handle_key_event(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> color_eyre::Result<Option<Action>> {
        if let Some(FocusTarget::Search) = self.focused {
            match key.code {
                KeyCode::Enter => {} // No new line, instead handle it as an app event
                _ => {
                    self.search_input.input(key);
                    self.list_state.select_first();
                }
            }
        }
        Ok(None)
    }

    fn handle_app_events(
        &mut self,
        event: crate::app_event::AppEvent,
    ) -> color_eyre::Result<Option<Action>> {
        match event {
            // When a database connection is established, trigger a system query for the tables
            AppEvent::DbConnectionEstablished(_) => Ok(Some(Action::ExecuteQuery(
                SystemQuery::query_for(QueryTag::ListTables)?,
            ))),
            // Listen for when the query is returned
            AppEvent::QueryResult(result, QueryTag::ListTables) => {
                self.items = result
                    .rows
                    .iter()
                    .map(|r| r.get(1).cloned().unwrap_or_else(|| "---".into()))
                    .collect();
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::Result<()> {
        let has_focus = self.focused.is_some();

        let block = Block::bordered()
            .title("tables [alt+1]")
            .style(Style::new().fg(if has_focus { Color::Cyan } else { Color::Blue }))
            .title_alignment(Alignment::Center)
            .border_type(if has_focus {
                BorderType::Thick
            } else {
                BorderType::Plain
            });

        let list = List::new(self.display_items().clone())
            .style(Color::Cyan)
            .highlight_style(Modifier::REVERSED)
            .highlight_symbol("▹ ")
            .block(block);

        let show_search = matches!(self.focused, Some(FocusTarget::Search)) || self.has_search();

        if show_search {
            let layout = Layout::vertical([Constraint::Min(1), Constraint::Fill(100)]).split(area);

            frame.render_widget(&self.search_input, layout[0]);
            frame.render_stateful_widget(list, layout[1], &mut self.list_state.clone());
        } else {
            frame.render_stateful_widget(list, area, &mut self.list_state.clone());
        }

        Ok(())
    }
}

impl<'a> TableList<'a> {
    fn display_items(&self) -> Vec<String> {
        if self.has_search() {
            return self
                .items
                .iter()
                .filter(|it| it.contains(&self.search_content()))
                .cloned()
                .collect::<Vec<String>>();
        }
        self.items.clone()
    }

    fn search_content(&self) -> String {
        self.search_input.lines().join("")
    }

    fn has_search(&self) -> bool {
        !self.search_content().is_empty()
    }

    fn selection(&self) -> Option<String> {
        if let Some(index) = self.list_state.selected()
            && let Some(selection) = self.display_items().get(index)
        {
            return Some(String::from(selection));
        }
        None
    }
}
