use crate::app_event::QueryTag;

pub struct SystemQuery {}

impl SystemQuery {
    pub fn query_for(tag: QueryTag) -> String {
        match tag {
            QueryTag::TableSchema => String::from(
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
        }
    }
}
