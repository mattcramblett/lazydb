use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::{Client, NoTls, Row, types::Type};

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ConnectionConfig {
    host: Option<String>,
    port: Option<u16>,
    user: Option<String>,
    password: Option<String>,
    database_name: Option<String>,
}

pub struct DbConnection {
    db_client: Client,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl DbConnection {
    /// Creates a database connection with the given config
    pub async fn create(config: ConnectionConfig) -> color_eyre::Result<Self> {
        let (client, connection) =
            tokio_postgres::connect(&Self::make_connection_string(&config)?, NoTls).await?;
        connection.await?;

        Ok(Self { db_client: client })
    }

    /// Returns a result from the given query
    pub async fn get_query_result(
        &self,
        query: String,
    ) -> Result<QueryResult, tokio_postgres::Error> {
        let rows = self.db_client.query(&query, &[]).await?;

        // Assume every row in the result has the same columns. Pre-compute how many columns to
        // display.
        let Some(first_row) = rows.first() else {
            return Ok(QueryResult::default());
        };
        let col_count = first_row.columns().len();
        let columns = first_row
            .columns()
            .iter()
            .map(|c| c.name().to_string())
            .collect();
        let mut results: Vec<Vec<String>> = vec![];

        for row in rows {
            let data_row = (0..col_count)
                .map(|i| Self::value_to_string(&row, i).unwrap())
                .collect();
            results.push(data_row);
        }

        Ok(QueryResult {
            columns,
            rows: results,
        })
    }

    /// Converts the postgres type to a formatted string for visual display
    /// TODO: revisit and cover more types
    fn value_to_string(row: &Row, i: usize) -> Result<String, tokio_postgres::Error> {
        let col = &row.columns()[i];
        let ty = col.type_();

        let s = match *ty {
            Type::BOOL => row
                .try_get::<usize, Option<bool>>(i)
                .map(|opt| opt.map_or("NULL".into(), |v| v.to_string()))?,
            Type::INT2 => row
                .try_get::<usize, Option<i16>>(i)
                .map(|opt| opt.map_or("NULL".into(), |v| v.to_string()))?,
            Type::INT4 => row
                .try_get::<usize, Option<i32>>(i)
                .map(|opt| opt.map_or("NULL".into(), |v| v.to_string()))?,
            Type::INT8 => row
                .try_get::<usize, Option<i64>>(i)
                .map(|opt| opt.map_or("NULL".into(), |v| v.to_string()))?,
            Type::FLOAT4 => row
                .try_get::<usize, Option<f32>>(i)
                .map(|opt| opt.map_or("NULL".into(), |v| v.to_string()))?,
            Type::FLOAT8 => row
                .try_get::<usize, Option<f64>>(i)
                .map(|opt| opt.map_or("NULL".into(), |v| v.to_string()))?,
            // text types
            Type::TEXT | Type::VARCHAR | Type::BPCHAR | Type::NAME => row
                .try_get::<usize, Option<String>>(i)
                .map(|opt| opt.unwrap_or_else(|| "NULL".into()))?,
            Type::BYTEA => row
                .try_get::<usize, Option<Vec<u8>>>(i)
                .map(|opt| opt.map_or("NULL".into(), |b| format!("<BYTEA {} bytes>", b.len())))?,
            Type::TIMESTAMP | Type::TIMESTAMPTZ => {
                // TODO: there might be a cleaner way with chrono that doesn't require `into`
                row.try_get::<usize, Option<SystemTime>>(i).map(|opt| {
                    opt.map_or("NULL".into(), |t| {
                        let dt: DateTime<Utc> = t.into();
                        dt.to_rfc3339()
                    })
                })?
            }
            Type::DATE => row
                .try_get::<usize, Option<chrono::NaiveDate>>(i)
                .map(|opt| opt.map_or("NULL".into(), |d| d.to_string()))?,
            // Fallback: try String, then show type name
            _ => match row.try_get::<usize, Option<String>>(i) {
                Ok(opt) => opt.unwrap_or_else(|| "NULL".into()),
                Err(_) => format!("<unsupported type: {}>", ty.name()),
            },
        };

        Ok(s)
    }

    fn make_connection_string(config: &ConnectionConfig) -> color_eyre::Result<String> {
        let mut result: Vec<String> = vec![];

        if let Some(host) = &config.host {
            result.push(format!("host={}", host).to_string());
        }

        if let Some(port) = &config.port {
            result.push(format!("port={}", port).to_string());
        }

        if let Some(user) = &config.user {
            result.push(format!("user={}", user).to_string());
        }

        if let Some(password) = &config.password {
            result.push(format!("password={}", password).to_string());
        }

        if let Some(db_name) = &config.database_name {
            result.push(format!("dbname={}", db_name).to_string());
        }

        Ok(result.join(" "))
    }
}
