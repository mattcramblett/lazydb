use crate::database::connection::{DbConnection, QueryResult};

/// App events are events triggered in the system that are not direct user actions (e.g. they will
/// not have key maps tied directly to them.
#[derive(Clone)]
pub enum AppEvent {
    DbConnectionEstablished(DbConnection),
    QueryResult(QueryResult),
    UserMessage(MessageType, String),
}

#[derive(Clone)]
pub enum MessageType {
    Error,
    Info,
}
