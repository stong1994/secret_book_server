# Secret Book Server

The bak-server for [secret book](https://github.com/stong1994/secret_book) to sync secrets.

## 1. Init db

```
cd db && sh setup_db.sh && cd -
```

## 2. Run

```
cargo run
```

## clean db

```
cd db && sh clean_db.sh && cd -
```

## Install

1. set db path

```bash
export SECRET_SERVER_DB_URL="/xxxxx/xxx.db"
```

2. run

```bash
./secret_book_server
```
