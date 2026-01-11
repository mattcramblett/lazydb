use color_eyre::Result;
use color_eyre::eyre::bail;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::app_event::QueryTag;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Query {
    pub tag: QueryTag,
    pub query: String,
    pub binds: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Table {
    pub schema: String,
    pub name: String,
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
                Ok(Query {
                    query,
                    binds: None,
                    tag,
                })
            }
            QueryTag::InitialTable(table) => {
                let re = Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").unwrap();
                if !{
                    re.is_match(&table.name)
                        && !table.name.is_empty()
                        && { table.name.len() <= 128 }
                        && re.is_match(&table.schema)
                        && !table.schema.is_empty()
                        && { table.schema.len() <= 128 }
                } {
                    bail!(format!(
                        "Invalid table name or schema found: {}.{}",
                        table.schema, table.name
                    ))
                }
                let quoted = format!(
                    "\"{}\".\"{}\"",
                    table.schema.replace('"', "\"\""),
                    table.name.replace('"', "\"\"")
                );
                let query = format!("SELECT * FROM {} LIMIT 1000;", quoted);
                Ok(Query {
                    query,
                    binds: None,
                    tag: QueryTag::User,
                }) // NOTE: user initiated
            }
            QueryTag::TableStructure(table) => {
                let query = String::from(
"
SELECT
	col.column_name column_name,
	CASE
		WHEN udt_name IN ('varchar', 'bpchar') THEN concat(udt_name, '(', character_maximum_length, ')')
		WHEN udt_name = 'numeric'
		AND numeric_precision IS NOT NULL THEN concat('numeric(', numeric_precision, ',', numeric_scale, ')')
		ELSE udt_name
	END AS data_type,
	is_nullable,
	CASE
		WHEN column_default IS NULL THEN ''
		ELSE column_default
	END AS column_default,
	CASE
		WHEN rel.column_name IS NOT NULL THEN concat(rel.table_schema, '.', rel.table_name, '(', rel.column_name, ')')
		ELSE ''
	END AS foreign_key
FROM
	information_schema.columns col
	LEFT JOIN (
		SELECT
			kcu.constraint_schema,
			kcu.constraint_name,
			kcu.table_schema,
			kcu.table_name,
			kcu.column_name,
			kcu.ordinal_position,
			kcu.position_in_unique_constraint
		FROM
			information_schema.key_column_usage kcu
			JOIN information_schema.table_constraints tco ON kcu.constraint_schema = tco.constraint_schema
			AND kcu.constraint_name = tco.constraint_name
			AND tco.constraint_type = 'FOREIGN KEY'
	) AS kcu ON col.table_schema = kcu.table_schema
	AND col.table_name = kcu.table_name
	AND col.column_name = kcu.column_name
	LEFT JOIN information_schema.referential_constraints rco ON rco.constraint_name = kcu.constraint_name
	AND rco.constraint_schema = kcu.table_schema
	LEFT JOIN information_schema.key_column_usage rel ON rco.unique_constraint_name = rel.constraint_name
	AND rco.unique_constraint_schema = rel.constraint_schema
	AND rel.ordinal_position = kcu.position_in_unique_constraint
WHERE
	col.table_schema NOT IN ('information_schema', 'pg_catalog')
	AND col.table_schema = $1 AND col.table_name = $2
ORDER BY
	col.ordinal_position;
"
                );
                Ok(Query {
                    query,
                    binds: Some(vec![table.schema, table.name]),
                    tag,
                })
            }
            QueryTag::User => {
                // NOTE: special case, not a system query. Explictly matching this case to force
                // matching against all meaningful variants.
                Ok(Query {
                    query: String::default(),
                    binds: None,
                    tag,
                })
            }
        }
    }
}
