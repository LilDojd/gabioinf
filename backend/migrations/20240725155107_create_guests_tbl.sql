CREATE TABLE IF NOT EXISTS guests (
    id              BIGSERIAL PRIMARY KEY,
    github_id       VARCHAR(255) UNIQUE NOT NULL,
    name            VARCHAR(255) NOT NULL,
    username        VARCHAR(255) NOT NULL,
    is_naughty      BOOLEAN NOT NULL DEFAULT FALSE,
    naughty_reason  VARCHAR(255),
    created_at      TIMESTAMP WITH TIME ZONE DEFAULT current_timestamp,
    updated_at      TIMESTAMP WITH TIME ZONE DEFAULT current_timestamp

    CONSTRAINT check_naughty_reason CHECK (
        (is_naughty = TRUE AND naughty_reason IS NOT NULL) OR
        (is_naughty = FALSE AND naughty_reason IS NULL)
    )

);

-- Index for faster lookups by github_id
CREATE INDEX idx_guests_github_id ON guests(github_id);

-- Add a unique constraint to ensure one signature per guest
ALTER TABLE guestbook
ADD CONSTRAINT unique_author_signature UNIQUE (author_id);


