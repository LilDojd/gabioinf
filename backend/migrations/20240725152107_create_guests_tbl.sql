CREATE TABLE IF NOT EXISTS guests (
    id              BIGSERIAL PRIMARY KEY,
    github_id       BIGSERIAL UNIQUE NOT NULL,
    name            VARCHAR(255) NOT NULL,
    username        VARCHAR(255) NOT NULL UNIQUE,
    is_naughty      BOOLEAN NOT NULL DEFAULT FALSE,
    is_admin        BOOLEAN NOT NULL DEFAULT FALSE,
    naughty_reason  VARCHAR(255),
    created_at      TIMESTAMP WITH TIME ZONE DEFAULT current_timestamp NOT NULL,
    updated_at      TIMESTAMP WITH TIME ZONE DEFAULT current_timestamp NOT NULL,
    access_token    TEXT NOT NULL

    CONSTRAINT check_naughty_reason CHECK (
        (is_naughty = TRUE AND naughty_reason IS NOT NULL) OR
        (is_naughty = FALSE AND naughty_reason IS NULL)
    )
);

-- Index for faster lookups by github_id
CREATE INDEX idx_guests_github_id ON guests(github_id);

-- `groups` table
CREATE TABLE IF NOT EXISTS groups (
    id              BIGSERIAL PRIMARY KEY,
    name            TEXT NOT NULL UNIQUE
);
-- Create `permissions` table.
CREATE TABLE IF NOT EXISTS permissions (
    id              BIGSERIAL PRIMARY KEY,
    name            TEXT NOT NULL UNIQUE
);


-- Create `guests_groups` table for many-to-many relationships between guests and groups.
CREATE TABLE IF NOT EXISTS guests_groups (
    guest_id        BIGSERIAL NOT NULL REFERENCES guests(id) ON DELETE RESTRICT,
    group_id        BIGSERIAL NOT NULL REFERENCES groups(id) ON DELETE RESTRICT,
    PRIMARY KEY     (guest_id, group_id)
);

-- Create `groups_permissions` table for many-to-many relationships between groups and permissions.
CREATE TABLE IF NOT EXISTS groups_permissions (
    group_id        BIGSERIAL NOT NULL REFERENCES groups(id) ON DELETE RESTRICT,
    permission_id   BIGSERIAL NOT NULL REFERENCES permissions(id) ON DELETE RESTRICT,
    PRIMARY KEY     (group_id, permission_id)
);

-- Create `guests_permissions` table for many-to-many relationships between guests and permissions.
CREATE TABLE IF NOT EXISTS guests_permissions (
    guest_id        BIGSERIAL NOT NULL REFERENCES guests(id) ON DELETE RESTRICT,
    permission_id   BIGSERIAL NOT NULL REFERENCES permissions(id) ON DELETE RESTRICT,
    PRIMARY KEY     (guest_id, permission_id)
);

-- # Fixture hydration.

INSERT INTO groups (name) VALUES ('admins');
INSERT INTO groups (name) VALUES ('guests');
INSERT INTO groups (name) VALUES ('naughty_guests');

-- Insert individual permissions

-- Guestbook entries permission
INSERT INTO permissions (name) VALUES ('guestbook.read');
INSERT INTO permissions (name) VALUES ('guestbook.write');
INSERT INTO permissions (name) VALUES ('guestbook.delete');
INSERT INTO permissions (name) VALUES ('guestbook.update');

-- User-related permissions
INSERT INTO permissions (name) VALUES ('guest.read');
INSERT INTO permissions (name) VALUES ('guest.write');
INSERT INTO permissions (name) VALUES ('guest.delete');
INSERT INTO permissions (name) VALUES ('guest.promote');
INSERT INTO permissions (name) VALUES ('guest.demote');
INSERT INTO permissions (name) VALUES ('guest.marknauhgty');
INSERT INTO permissions (name) VALUES ('guest.dashboard');

-- All guests can read entries
INSERT INTO groups_permissions (group_id, permission_id) VALUES (
    (SELECT id FROM groups WHERE name = 'guests'),
    (SELECT id FROM permissions WHERE name = 'guestbook.read')
);

-- Admins can do everything

INSERT INTO groups_permissions (group_id, permission_id) VALUES (
    (SELECT id FROM groups WHERE name = 'admins'),
    (SELECT id FROM permissions WHERE name = 'guestbook.delete')
);

INSERT INTO groups_permissions (group_id, permission_id) VALUES (
    (SELECT id FROM groups WHERE name = 'admins'),
    (SELECT id FROM permissions WHERE name = 'guestbook.update')
);

INSERT INTO groups_permissions (group_id, permission_id) VALUES (
    (SELECT id FROM groups WHERE name = 'admins'),
    (SELECT id FROM permissions WHERE name = 'guest.read')
);

INSERT INTO groups_permissions (group_id, permission_id) VALUES (
    (SELECT id FROM groups WHERE name = 'admins'),
    (SELECT id FROM permissions WHERE name = 'guest.write')
);

INSERT INTO groups_permissions (group_id, permission_id) VALUES (
    (SELECT id FROM groups WHERE name = 'admins'),
    (SELECT id FROM permissions WHERE name = 'guest.delete')
);

INSERT INTO groups_permissions (group_id, permission_id) VALUES (
    (SELECT id FROM groups WHERE name = 'admins'),
    (SELECT id FROM permissions WHERE name = 'guest.promote')
);

INSERT INTO groups_permissions (group_id, permission_id) VALUES (
    (SELECT id FROM groups WHERE name = 'admins'),
    (SELECT id FROM permissions WHERE name = 'guest.demote')
);

INSERT INTO groups_permissions (group_id, permission_id) VALUES (
    (SELECT id FROM groups WHERE name = 'admins'),
    (SELECT id FROM permissions WHERE name = 'guest.marknauhgty')
);


