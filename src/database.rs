use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{time::SystemTime};
use tokio_postgres::{Error, NoTls, Row, types::Type};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub async fn get_query_result(query: String) -> Result<QueryResult, Error> {
    let dbname = "sportsdb";

    let (client, connection) = tokio_postgres::connect(
        &format!(
            "host=localhost user=postgres password=postgres dbname={}",
            dbname
        ),
        NoTls,
    )
    .await?;

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let rows = client.query(&query, &[]).await?;

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
            .map(|i| value_to_string(&row, i).unwrap())
            .collect();
        results.push(data_row);
    }

    Ok(QueryResult {
        columns,
        rows: results,
    })
}

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
