CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    tags TEXT[],
    stars INTEGER NOT NULL DEFAULT 0,
    title VARCHAR NOT NULL,
    slug VARCHAR UNIQUE NOT NULL,
    content TEXT,
    media TEXT[],
    created_at TIMESTAMP NOT NULL DEFAULT (now() AT TIME ZONE 'UTC'),
    edited BOOLEAN NOT NULL DEFAULT FALSE,
    edited_at TIMESTAMP,
    forum INTEGER NOT NULL REFERENCES forums(id),
    poster INTEGER NOT NULL REFERENCES users(id)
);