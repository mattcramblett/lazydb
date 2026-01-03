use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use color_eyre::eyre::bail;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row};
use sqlx::postgres::{PgColumn, PgRow};
use sqlx::{
    Column, PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ConnectionConfig {
    host: Option<String>,
    port: Option<u16>,
    user: Option<String>,
    password: Option<String>,
    database_name: Option<String>,
}

#[derive(Clone)]
pub struct DbConnection {
    pool: PgPool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl DbConnection {
    /// Creates a database connection with the given config
    pub async fn create(config: ConnectionConfig) -> color_eyre::Result<Self> {
        let options = Self::make_connection_opts(&config);
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect_with(options)
            .await?;

        Ok(Self { pool })
    }

    pub async fn get_query_result(&self, query: String, binds: Option<Vec<String>>) -> color_eyre::Result<QueryResult> {
        let mut sql = sqlx::query::<Postgres>(&query);
        if let Some(params) = binds {
            for value in params {
                sql = sql.bind(value)
            }
        }
        let rows = sql.fetch_all(&self.pool).await?;
        let mut iter = rows.iter().peekable();

        let first_row = iter.peek();
        let mut columns: Vec<String> = vec![];
        let mut col_info: Vec<PgColumn> = vec![];

        // Assume the first row has the same columns as the rest of the rows
        if let Some(row) = first_row {
            col_info.append(&mut row.columns().to_vec());
            let mut cols: Vec<String> =
                row.columns().iter().map(|c| c.name().to_string()).collect();
            columns.append(&mut cols);
        } else {
            return Ok(QueryResult {
                rows: Default::default(),
                columns: Default::default(),
            });
        }

        let mut results: Vec<Vec<String>> = vec![];
        for row in iter {
            let mut r: Vec<String> = vec![];
            for col in row.columns() {
                r.push(Self::display_value(row, col)?);
            }
            results.push(r);
        }

        Ok(QueryResult {
            rows: results,
            columns,
        })
    }

    fn make_connection_opts(config: &ConnectionConfig) -> PgConnectOptions {
        let mut options = PgConnectOptions::default();

        if let Some(host) = &config.host {
            options = options.host(host);
        }

        if let Some(port) = &config.port {
            options = options.port(*port);
        }

        if let Some(user) = &config.user {
            options = options.username(user);
        }

        if let Some(password) = &config.password {
            options = options.password(password);
        }

        if let Some(db_name) = &config.database_name {
            options = options.database(db_name);
        }

        options
    }

    /// sqlx adds compile-time type safety for database types by connecting to a database at
    /// compile-time. Since we don't know what the type will be until runtime, we have to check
    /// possibilities.
    fn display_value(row: &PgRow, col: &PgColumn) -> Result<String> {
        let index = col.ordinal();
        let null = "NULL".to_string();

        if let Ok(val) = row.try_get::<Option<i16>, _>(index) {
            Ok(val.map_or(null, |v| v.to_string()))
        } else if let Ok(val) = row.try_get::<Option<i8>, _>(index) {
            Ok(val.map_or(null, |v| v.to_string()))
        } else if let Ok(val) = row.try_get::<Option<i32>, _>(index) {
            Ok(val.map_or(null, |v| v.to_string()))
        } else if let Ok(val) = row.try_get::<Option<i64>, _>(index) {
            Ok(val.map_or(null, |v| v.to_string()))
        } else if let Ok(val) = row.try_get::<Option<f32>, _>(index) {
            Ok(val.map_or(null, |v| v.to_string()))
        } else if let Ok(val) = row.try_get::<Option<f64>, _>(index) {
            Ok(val.map_or(null, |v| v.to_string()))
        } else if let Ok(val) = row.try_get::<Option<bigdecimal::BigDecimal>, _>(index) {
            Ok(val.map_or(null, |v| v.to_string()))
        } else if let Ok(val) = row.try_get::<Option<&[u8]>, _>(index) {
            // Postgres bytea type: try utf-8 first for readability but fallback to hex
            match val {
                None => Ok(null),
                Some(bytes) => {
                    if let Ok(s) = str::from_utf8(bytes) {
                        Ok(s.to_string())
                    } else {
                        Ok(format!("\\x{}", hex::encode(bytes)))
                    }
                }
            }
        } else if let Ok(val) = row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>(index) {
            Ok(val.map_or(null, |v| v.to_rfc3339()))
        } else if let Ok(val) = row.try_get::<Option<chrono::DateTime<chrono::Local>>, _>(index) {
            Ok(val.map_or(null, |v| v.to_rfc3339()))
        } else if let Ok(val) = row.try_get::<Option<NaiveDateTime>, _>(index) {
            Ok(val.map_or(null, |v| v.to_string()))
        } else if let Ok(val) = row.try_get::<Option<NaiveDate>, _>(index) {
            Ok(val.map_or(null, |v| v.format("%Y-%m-%d").to_string()))
        } else if let Ok(val) = row.try_get::<Option<NaiveTime>, _>(index) {
            Ok(val.map_or(null, |v| v.format("%H:%M:%S%.6f").to_string()))
        } else if let Ok(val) = row.try_get::<Option<serde_json::Value>, _>(index) {
            Ok(val.map_or(null, |v| v.to_string()))
        } else if let Ok(val) = row.try_get::<Option<bool>, _>(index) {
            Ok(val.map_or(null, |v| v.to_string()))
        } else if let Ok(val) = row.try_get::<Option<Vec<String>>, _>(index) {
            Ok(val.map_or(null, |vals| vals.join(",")))
        } else if let Ok(val) = row.try_get::<Option<String>, _>(index) {
            Ok(val.unwrap_or(null))
        } else {
            bail!(format!("Unsupported type: {}", col.type_info()))
        }
    }
}
