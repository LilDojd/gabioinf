CREATE TABLE comments (
    id         BIGSERIAL PRIMARY KEY,
    post_slug  VARCHAR(80) NOT NULL,
    author_id  BIGINT NOT NULL REFERENCES guests(id),
    parent_id  BIGINT REFERENCES comments(id) ON DELETE CASCADE,
    body       VARCHAR(2000) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT current_timestamp
);

CREATE INDEX comments_post_slug_created_at_idx ON comments (post_slug, created_at);
