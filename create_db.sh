#!/bin/sh

set -eu

# A shell script to create a database of the latest version.
# sqlx macros require an existing database for compile-time
# checking of SQL queries, and this script can be used to
# create one easily.

repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
db_file="$repo_dir/test.db"
schema_file="$repo_dir/docs/database-schema.sql"

rm -f "$db_file"

for migration_file in "$repo_dir"/src/migrations/*.sql; do
  sqlite3 "$db_file" <"$migration_file"
done
