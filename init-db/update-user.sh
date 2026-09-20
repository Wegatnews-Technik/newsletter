#!/bin/sh

until pg_isready; do
    sleep 1
done

if psql ${DB_NAME} -c '\q' 2>&1; then
    exit 0
fi

set -e

# Read Password configuration
subscriber_pw=$( cat /run/secrets/pg-subscriber )
mailbot_pw=$( cat /run/secrets/pg-mailbot )

# Update Passwords in case they changes since DB init
psql -v ON_ERROR_STOP --username postgres -d newsletter <<-EOSQL
    ALTER USER subscriber WITH PASSWORD '$subscriber_pw';
EOSQL

psql -v ON_ERROR_STOP --username postgres -d newsletter <<-EOSQL
    ALTER USER mailbot WITH PASSWORD '$mailbot_pw';
EOSQL
