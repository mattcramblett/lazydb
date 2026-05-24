use serde::{Deserialize, Serialize};
use strum::Display;

use crate::{
    app::Mode,
    database::{
        connection::{DbConnection, QueryResult},
        system_query::{self, Table},
    },
};

/// App events are global signals that can be produced and reacted upon by components, independent of
/// current layout and do not require the component to be in focus.
/// Events can be produced as a result of system-level triggers or indirectly through user initiated
/// actions.
#[derive(Clone)]
pub enum AppEvent {
    ModeSwitched(Mode),
    DbConnectionRequested(String),
    DbConnectionEstablished(DbConnection),
    SchemaChangeRequested(String),
    QueryExecutionRequested(system_query::Query),
    QueryResultReturned(QueryResult, QueryTag),
    UserMessage(MessageType, String),
    CellSelected(String),
    RowSelected(Vec<String>, Vec<Option<String>>), // columns, row
}

#[derive(Clone)]
pub enum MessageType {
    Error,
    Info,
    Debug,
}

/// Queries performed can be tagged for specific listeners. Some queries are triggered by the system
/// for app functionality will be tagged for specific use cases.
#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum QueryTag {
    /// User queries are triggered by the user and should be shown in the results table.
    User,
    ListTables,
    InitialTable(Table),
    TableStructure(Table),
}
