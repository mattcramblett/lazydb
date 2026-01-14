use std::collections::{HashSet};

use arboard::Clipboard;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::Text,
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
};

pub struct SchemaList<'a> {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    list_state: ListState,
    focused: Option<FocusTarget>,
    /// schema, table
    items: Vec<String>,
    search_input: TextArea<'a>,
}

enum FocusTarget {
    List,
    Search,
}

impl<'a> Default for SchemaList<'a> {
    fn default() -> Self {
        let mut search_input = TextArea::default();
        search_input.set_placeholder_text("Search schemas");
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

impl<'a> Component for SchemaList<'a> {
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
            Action::ChangeMode(Mode::ExploreSchemas) => self.focused = Some(FocusTarget::List),
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
                        && self.display_items().count() > 0
                        && selected >= self.display_items().count() - 1
                    {
                        return Ok(None);
                    }
                    self.list_state.select_next();
                }
                Action::NavUp => self.list_state.select_previous(),
                Action::MakeSelection => {
                    if let Some(selection) = self.selection() {
                        return Ok(Some(Action::ChangeSchema(selection)))
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
                    text_area.set_placeholder_text("Search schemas");
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
            text_area.set_placeholder_text("Search schemas");
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
            // Listen for when the ListTables query is returned to populate schemas
            AppEvent::QueryResult(result, QueryTag::ListTables) => {
                self.items = result
                    .rows
                    .iter()
                    .map(|r| r.first().cloned().unwrap_or_else(|| "unknown".into()))
                    .collect::<HashSet<String>>()
                    .into_iter()
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
            .title("Schemas [alt+0]")
            .style(Style::new().fg(if has_focus { Color::Cyan } else { Color::Blue }))
            .title_alignment(Alignment::Center)
            .border_type(if has_focus {
                BorderType::Thick
            } else {
                BorderType::Plain
            });

        let list = List::new(
            self.display_items()
                .map(|schema| Text::styled(schema, Color::Cyan)),
        )
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

impl<'a> SchemaList<'a> {
    fn display_items(&self) -> impl Iterator<Item = &String> {
        self.items
            .iter()
            .filter(move |name| !self.has_search() || name.contains(&self.search_content()))
    }

    fn search_content(&self) -> String {
        self.search_input.lines().join("")
    }

    fn has_search(&self) -> bool {
        !self.search_content().is_empty()
    }

    fn selection(&self) -> Option<String> {
        if let Some(index) = self.list_state.selected()
            && let Some(schema) = self.display_items().nth(index)
        {
            return Some(schema.clone());
        }
        None
    }
}
