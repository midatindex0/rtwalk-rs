CREATE TABLE users (
    id SERIAL UNIQUE,
    username VARCHAR UNIQUE PRIMARY KEY,
    password VARCHAR NOT NULL,
    display_name VARCHAR NOT NULL,
    bio TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT (now() AT TIME ZONE 'UTC')
);