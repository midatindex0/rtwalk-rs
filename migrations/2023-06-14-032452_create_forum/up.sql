-- Your SQL goes here
CREATE TABLE forums (
    id SERIAL PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT (now() AT TIME ZONE 'UTC'),
    owner_id SERIAL REFERENCES users(id)
);