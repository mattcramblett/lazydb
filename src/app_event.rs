
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::database::connection::{DbConnection, QueryResult};

/// App events are events triggered in the system that are not direct user actions (e.g. they will
/// not have key maps tied directly to them.
#[derive(Clone)]
pub enum AppEvent {
    DbConnectionEstablished(DbConnection),
    QueryResult(QueryResult, Option<QueryTag>),
    UserMessage(MessageType, String),
}

#[derive(Clone)]
pub enum MessageType {
    Error,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum QueryTag {
    /// Queries performed by the system for app functionality
    TableSchema
}

