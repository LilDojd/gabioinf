-- Add migration script here
CREATE TABLE IF NOT EXISTS sessions (
    id              BIGSERIAL PRIMARY KEY,
    user_id         INT NOT NULL UNIQUE,
    session_id      VARCHAR NOT NULL,
    expires_at      TIMESTAMP WITH TIME ZONE NOT NULL,
    FOREIGN KEY     (user_id) REFERENCES guests(id)
);
