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
    forum_id INTEGER NOT NULL REFERENCES forums(id),
    poster_id INTEGER NOT NULL REFERENCES users(id)
);

CREATE INDEX post_index ON posts USING btree (id DESC);
