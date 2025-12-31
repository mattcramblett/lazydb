use serde::{Deserialize, Serialize};
use strum::Display;

use crate::database::connection::{DbConnection, QueryResult};

/// App events are events triggered in the system that are not direct user actions (e.g. they will
/// not have key maps tied directly to them.
#[derive(Clone)]
pub enum AppEvent {
    DbConnectionEstablished(DbConnection),
    QueryResult(QueryResult, QueryTag),
    UserMessage(MessageType, String),
}

#[derive(Clone)]
pub enum MessageType {
    Error,
    Info,
}

/// Queries performed can be tagged for specific listeners. Some queries are triggered by the system
/// for app functionality will be tagged for specific use cases.
#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum QueryTag {
    /// User queries are triggered by the user and should be shown in the results table.
    User,
    ListTables,
    InitialTable(String),
}
