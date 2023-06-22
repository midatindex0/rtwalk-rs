CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR UNIQUE NOT NULL,
    password VARCHAR NOT NULL,
    display_name VARCHAR NOT NULL,
    bio TEXT,
    pfp VARCHAR,
    created_at TIMESTAMP NOT NULL DEFAULT (now() AT TIME ZONE 'UTC')
);