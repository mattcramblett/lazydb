use regex::Regex;

use crate::app_event::QueryTag;

pub struct SystemQuery {}

impl SystemQuery {
    pub fn query_for(tag: QueryTag) -> String {
        match tag {
            QueryTag::ListTables => String::from(
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
            ),
            QueryTag::InitialTable(table_name) => {
                if !{
                    let name: &str = &table_name;
                    let re = Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").unwrap();
                    re.is_match(name) && !name.is_empty() && name.len() <= 128
                } {
                    // NOTE: temporary validation until better parameterized statements are available
                    return String::from("invalid_table_name");
                }
                let quoted = format!("\"{}\"", table_name.replace('"', "\"\""));
                format!("SELECT * FROM {} LIMIT 1000;", quoted)
            }
            QueryTag::User => String::new(), // NOTE: special case, not a system query
        }
    }
}

