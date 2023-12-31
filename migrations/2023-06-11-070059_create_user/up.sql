CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR UNIQUE NOT NULL,
    password VARCHAR NOT NULL,
    display_name VARCHAR NOT NULL,
    bio TEXT,
    pfp VARCHAR,
    banner VARCHAR,
    created_at TIMESTAMP NOT NULL DEFAULT (now() AT TIME ZONE 'UTC'),
    v INTEGER NOT NULL DEFAULT 0,
    admin BOOLEAN NOT NULL DEFAULT FALSE
);
CREATE INDEX user_index ON users USING btree (id DESC, username);