-- Your SQL goes here
CREATE TABLE forums (
    id SERIAL PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    display_name VARCHAR NOT NULL,
    icon VARCHAR,
    banner VARCHAR,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT (now() AT TIME ZONE 'UTC'),
    owner_id INTEGER NOT NULL REFERENCES users(id)
);

CREATE INDEX forum_index ON forums USING btree (id DESC);