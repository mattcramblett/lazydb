use serde::{Deserialize, Serialize};
use strum::Display;

use crate::{app::Mode, database::system_query};

/// Actions are user-driven events, which differ from AppEvents
#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,
    ChangeMode(Mode),
    MakeSelection,
    OpenDbConnection(String),
    ViewStructure,
    ChangeSchema(String),
    ExecuteQuery(system_query::Query),
    NavDown,
    NavUp,
    NavLeft,
    NavRight,
    Yank,
    Search,
    Clear,
    SelectCell(String),
}
