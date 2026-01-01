use arboard::Clipboard;
use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, List, ListState, Paragraph},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    app::Mode,
    app_event::{AppEvent, QueryTag},
    components::Component,
    config::Config,
    database::system_query::SystemQuery,
};

pub struct TableList {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    list_state: ListState,
    focused: bool,
    items: Vec<String>,
}

impl Default for TableList {
    fn default() -> Self {
        Self {
            command_tx: Default::default(),
            config: Default::default(),
            list_state: ListState::default().with_selected(Some(0)),
            focused: true,
            items: Default::default(),
        }
    }
}

impl Component for TableList {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> color_eyre::Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> color_eyre::Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
        match action {
            Action::NavDown if self.focused => {
                // protect against excess navigation
                if let Some(selected) = self.list_state.selected()
                    && !self.items.is_empty()
                    && selected >= self.items.len() - 1
                {
                    return Ok(None);
                }
                self.list_state.select_next()
            }
            Action::NavUp if self.focused => self.list_state.select_previous(),
            Action::MakeSelection if self.focused => {
                if let Some(selection) = self.selection() {
                    let table_name = selection.to_string();
                    return Ok(Some(Action::ExecuteQuery(
                        SystemQuery::query_for(QueryTag::InitialTable(table_name.clone())),
                        QueryTag::User, // tag as a User query, since they initiated the action
                    )));
                }
                return Ok(None);
            }
            Action::ChangeMode(Mode::ExploreTables) => self.focused = true,
            Action::ChangeMode(_) => self.focused = false,
            Action::Yank if self.focused => {
                if let Ok(clipboard) = Clipboard::new()
                    && let Some(selection) = self.selection()
                {
                    let mut clip = clipboard;
                    clip.set_text(selection)?
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
            // When a database connection is established, trigger a system query for the tables
            AppEvent::DbConnectionEstablished(_) => Ok(Some(Action::ExecuteQuery(
                SystemQuery::query_for(QueryTag::ListTables),
                QueryTag::ListTables,
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
        let block = Block::bordered()
            .title("tables [alt+1]")
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
            });

        if self.items.is_empty() {
            let paragraph = Paragraph::new("No tables found.").block(block);
            frame.render_widget(paragraph, area);
            return Ok(());
        }

        let list = List::new(self.items.clone())
            .style(Color::Cyan)
            .highlight_style(Modifier::REVERSED)
            .highlight_symbol("▹ ")
            .block(block);

        frame.render_stateful_widget(list, area, &mut self.list_state.clone());
        Ok(())
    }
}

impl TableList {
    fn selection(&self) -> Option<String> {
        if let Some(index) = self.list_state.selected()
            && let Some(selection) = self.items.get(index)
        {
            return Some(String::from(selection));
        }
        None
    }
}
