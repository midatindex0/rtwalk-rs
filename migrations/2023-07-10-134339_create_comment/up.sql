CREATE TABLE comments (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    post_id INTEGER NOT NULL REFERENCES posts(id),
    forum_id INTEGER NOT NULL REFERENCES forums(id),
    parent_id INTEGER REFERENCES comments(id),
    content TEXT NOT NULL,
    media TEXT[],
    created_at TIMESTAMP NOT NULL DEFAULT (now() AT TIME ZONE 'UTC'),
    edited BOOLEAN NOT NULL DEFAULT FALSE,
    edited_at TIMESTAMP
);

CREATE INDEX comment_index ON comments USING btree (id DESC);