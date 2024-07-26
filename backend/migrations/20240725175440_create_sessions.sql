-- Add migration script here
CREATE TABLE IF NOT EXISTS sessions (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGSERIAL NOT NULL UNIQUE,
    token           VARCHAR(255) NOT NULL,
    expires_at      TIMESTAMP WITH TIME ZONE NOT NULL,
    FOREIGN KEY     (user_id) REFERENCES guests(id)
);
