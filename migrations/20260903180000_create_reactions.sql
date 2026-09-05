CREATE TYPE reaction_target AS ENUM ('post', 'comment');

CREATE TABLE reactions (
    id          BIGSERIAL PRIMARY KEY,
    target_kind reaction_target NOT NULL,
    post_slug   VARCHAR(80) NOT NULL,
    comment_id  BIGINT REFERENCES comments(id) ON DELETE CASCADE,
    guest_id    BIGINT NOT NULL REFERENCES guests(id),
    emoji       VARCHAR(16) NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,
    CHECK ((target_kind = 'post') = (comment_id IS NULL)),
    UNIQUE NULLS NOT DISTINCT (target_kind, post_slug, comment_id, guest_id, emoji)
);

CREATE INDEX reactions_post_slug_idx ON reactions (post_slug);
