CREATE TABLE emails (
    uuid uuid PRIMARY KEY NOT NULL,
    address VARCHAR(255) UNIQUE NOT NULL
);
