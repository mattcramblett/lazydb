# lazydb

[![CI](https://github.com//lazydb/workflows/CI/badge.svg)](https://github.com/mattcramblett/lazydb/actions)

Terminal UI for database management. Currently in development.

## Sample data
For development, I'm using a local postgres container with some seeded data in order to test the UI.
```bash
docker run -d -p 5432:5432 --name pg-ds-latest aa8y/postgres-dataset:latest
```

## Example config

```yaml
db_connections:
  sportsdb:
    host: "localhost"
    port: 5432
    user: "postgres"
    password: postgres
    database_name: "sportsdb"
  world:
    host: "localhost"
    port: 5432
    user: "postgres"
    password: postgres
    database_name: "world"
```

Add env var to file:
```bash
export LAZYDB_CONFIG="/Users/myusername/.config/lazydb/"
```
