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
    ExecuteQuery(String),
    QueryResult(crate::database::QueryResult),
    DisplaySqlError(String),
    NavDown,
    NavUp,
    NavLeft,
    NavRight,
}
