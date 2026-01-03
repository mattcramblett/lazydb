use color_eyre::eyre::bail;
use color_eyre::Result;
use regex::Regex;

use crate::app_event::QueryTag;

pub struct SystemQuery {}

impl SystemQuery {
    pub fn query_for(tag: QueryTag) -> Result<String> {
        match tag {
            QueryTag::ListTables => Ok(String::from(
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
            )),
            QueryTag::InitialTable(table_name) => {
                if !{
                    let name: &str = &table_name;
                    let re = Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").unwrap();
                    re.is_match(name) && !name.is_empty() && name.len() <= 128
                } {
                    bail!(format!("Invalid table name found: {}", table_name))
                }
                let quoted = format!("\"{}\"", table_name.replace('"', "\"\""));
                Ok(format!("SELECT * FROM {} LIMIT 1000;", quoted))
            }
            QueryTag::User => Ok(String::new()), // NOTE: special case, not a system query
        }
    }
}

