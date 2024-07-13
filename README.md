# Secret Book Server

The bak-server for [secret book](https://github.com/stong1994/secret_book) to sync secrets.

## 1. Init db

```
cd db && sh setup_db.sh && cd -
```

## 2. Configuration

```shell
export SECRET_APPLICATION_PORT=12345
export SECRET_APPLICATION_HOST=localhost
export SECRET_LOG_LEVEL=info
export SECRET_LOG_DIR=xxxxx/logs/
export SECRET_DATABASE_URL=xxxxxxxx/secret.db
```

## 2. Run

```
cargo run
```

## clean db

```
cd db && sh clean_db.sh && cd -
```
