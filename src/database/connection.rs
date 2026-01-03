use serde::{Deserialize, Serialize};
use sqlx::Row;
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

    pub async fn get_query_result(&self, query: String) -> color_eyre::Result<QueryResult> {
        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;
        let mut iter = rows.iter().peekable();

        let first_row = iter.peek();
        let mut columns: Vec<String> = vec![];

        // Assume the first row has the same columns as the rest of the rows
        if let Some(row) = first_row {
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
            for i in 0..columns.len() {
                let val: String = row.try_get(i)?;
                r.push(val);
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
}
