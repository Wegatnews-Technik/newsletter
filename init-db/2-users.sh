set -e
# Read Password configuration
subscriber_pw=$( cat /run/secrets/pg-subscriber )
mailbot_pw=$( cat /run/secrets/pg-mailbot )

psql -v ON_ERROR_STOP --username "$POSTGRES_USER" --dbname $POSTGRES_DB <<-EOSQL
    CREATE USER subscriber WITH PASSWORD '$subscriber_pw';
    GRANT SELECT, INSERT, DELETE ON emails TO subscriber;
EOSQL

psql -v ON_ERROR_STOP --username "$POSTGRES_USER" --dbname $POSTGRES_DB <<-EOSQL
    CREATE USER mailbot WITH PASSWORD '$mailbot_pw';
    GRANT SELECT ON TABLE emails TO mailbot;
EOSQL
