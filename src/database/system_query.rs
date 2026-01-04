use color_eyre::eyre::bail;
use color_eyre::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::app_event::QueryTag;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Query {
    pub tag: QueryTag,
    pub query: String,
    pub binds: Option<Vec<String>>,
}

pub struct SystemQuery {}

impl SystemQuery {
    pub fn query_for(tag: QueryTag) -> Result<Query> {
        match tag.clone() {
            QueryTag::ListTables => {
                let query = String::from(
                "
SELECT
	table_schema,
	table_name
FROM
	information_schema.tables
WHERE
	table_schema NOT IN ('pg_catalog', 'information_schema')
ORDER BY
	table_schema, table_name ASC;
",
            );
            Ok(Query { query, binds: None, tag })
            }
            QueryTag::InitialTable(table_name) => {
                if !{
                    let name: &str = &table_name;
                    let re = Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").unwrap();
                    re.is_match(name) && !name.is_empty() && name.len() <= 128
                } {
                    bail!(format!("Invalid table name found: {}", table_name))
                }
                let quoted = format!("\"{}\"", table_name.replace('"', "\"\""));
                let query = format!("SELECT * FROM {} LIMIT 1000;", quoted);
                Ok(Query { query, binds: None, tag: QueryTag::User }) // NOTE: user initiated
            }
            QueryTag::TableStructure(table_name) => {
                let query = String::from(
"
SELECT
	column_name,
	CASE
		WHEN udt_name IN ('varchar', 'bpchar') THEN concat(udt_name, '(', character_maximum_length, ')')
		WHEN udt_name = 'numeric'
		AND numeric_precision IS NOT NULL THEN concat('numeric(', numeric_precision, ',', numeric_scale, ')')
		ELSE udt_name
	END AS data_type,
	column_default,
	is_nullable
FROM
	information_schema.columns
WHERE
	table_name = $1
ORDER BY
	ordinal_position ASC;
"
                );
                Ok(Query { query, binds: Some(vec![table_name]), tag })
            }
            QueryTag::User => {
                // NOTE: special case, not a system query. Explictly matching this case to force
                // matching against all meaningful variants.
                Ok(Query { query: String::default(), binds: None, tag })
            },
        }
    }
}

