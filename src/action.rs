use serde::{Deserialize, Serialize};
use strum::Display;

use crate::app::Mode;

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
    SelectConnection,
    ExecuteQuery(String),
    QueryResult(crate::database::connection::QueryResult),
    DisplaySqlError(String),
    NavDown,
    NavUp,
    NavLeft,
    NavRight,
}
