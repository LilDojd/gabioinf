CREATE TABLE IF NOT EXISTS guests (
    id              BIGSERIAL PRIMARY KEY,
    github_id       BIGSERIAL UNIQUE NOT NULL,
    name            VARCHAR(255) NOT NULL,
    username        VARCHAR(255) NOT NULL,
    is_naughty      BOOLEAN NOT NULL DEFAULT FALSE,
    is_admin        BOOLEAN NOT NULL DEFAULT FALSE,
    naughty_reason  VARCHAR(255),
    created_at      TIMESTAMP WITH TIME ZONE DEFAULT current_timestamp NOT NULL,
    updated_at      TIMESTAMP WITH TIME ZONE DEFAULT current_timestamp NOT NULL,

    CONSTRAINT check_naughty_reason CHECK (
        (is_naughty = TRUE AND naughty_reason IS NOT NULL) OR
        (is_naughty = FALSE AND naughty_reason IS NULL)
    )

);

-- Index for faster lookups by github_id
CREATE INDEX idx_guests_github_id ON guests(github_id);

