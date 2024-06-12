#!/usr/bin/env bash
cd $(dirname "$0")
sqlite3 secret.db <clean_db.sql
