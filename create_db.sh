#!/bin/sh

# A shell script to create a database of the latest version.
# sqlx macros require an existing database for compile-time
# checking of SQL queries, and this script can be used to
# create one easily.

db_file="test.db"

if [ -f $db_file ]; then
  rm $db_file
fi

for migration_file in src/migrations/*; do
  sqlite3 $db_file <"$migration_file"
done
