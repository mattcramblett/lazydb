# lazydb

[![CI](https://github.com//lazydb/workflows/CI/badge.svg)](https://github.com/mattcramblett/lazydb/actions)

Terminal UI for database management. Currently in development.

## Sample data
For development, I'm using a local postgres container with some seeded data in order to test the UI.
```bash
docker run -d -p 5432:5432 --name pg-ds-latest aa8y/postgres-dataset:latest
```
