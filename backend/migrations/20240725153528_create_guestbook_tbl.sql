-- Add migration script here
create table if not exists articles (
    id              BIGSERIAL,
    message         VARCHAR(255),
    signature       VARCHAR(255)                DEFAULT '',
    created_at      TIMESTAMP WITH TIME ZONE    DEFAULT current_timestamp,
    updated_at      TIMESTAMP WITH TIME ZONE    DEFAULT current_timestamp,
    author_id       BIGSERIAL,

    PRIMARY KEY(id),

    CONSTRAINT sig_author_id    FOREIGN KEY(author_id)   REFERENCES guests(id),
);
