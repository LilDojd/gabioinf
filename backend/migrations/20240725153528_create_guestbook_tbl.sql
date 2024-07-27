-- Add migration script here
create table if not exists guestbook (
    id              BIGSERIAL PRIMARY KEY,
    message         VARCHAR(255) NOT NULL,
    signature       VARCHAR(255)                DEFAULT '',
    created_at      TIMESTAMP WITH TIME ZONE    DEFAULT current_timestamp NOT NULL,
    updated_at      TIMESTAMP WITH TIME ZONE    DEFAULT current_timestamp NOT NULL,
    author_id       BIGSERIAL NOT NULL,


    CONSTRAINT sig_author_id    FOREIGN KEY(author_id)   REFERENCES guests(id),
    CONSTRAINT unique_author_signature UNIQUE (author_id)
);
