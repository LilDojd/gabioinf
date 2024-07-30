CREATE TABLE IF NOT EXISTS guests (
    id              BIGSERIAL PRIMARY KEY,
    github_id       BIGSERIAL UNIQUE NOT NULL,
    name            VARCHAR(255) NOT NULL,
    username        VARCHAR(255) NOT NULL UNIQUE,
    created_at      TIMESTAMP WITH TIME ZONE DEFAULT current_timestamp NOT NULL,
    updated_at      TIMESTAMP WITH TIME ZONE DEFAULT current_timestamp NOT NULL,
    access_token    TEXT NOT NULL
);

-- Index for faster lookups by github_id
CREATE INDEX idx_guests_github_id ON guests(github_id);

-- `groups` table
CREATE TYPE groupvariant AS ENUM ('guests', 'admins', 'naughty_guests');
CREATE TABLE IF NOT EXISTS groups (
    id              SMALLSERIAL PRIMARY KEY,
    name            groupvariant NOT NULL
);
-- Create `permissions` table.
CREATE TYPE permissionvariant AS ENUM (
    'addsignature',
    'deleteownsignature',
    'deleteanysignature',
    'editownsignature',
    'deleteuser',
    'markasnaughty',
    'prodemoteuser',
    'edituserpermissions'
);
CREATE TABLE IF NOT EXISTS permissions (
    id              SMALLSERIAL PRIMARY KEY,
    name            permissionvariant NOT NULL
);


-- Create `guests_groups` table for many-to-many relationships between guests and groups.
CREATE TABLE IF NOT EXISTS guests_groups (
    guest_id        BIGSERIAL NOT NULL REFERENCES guests(id) ON DELETE RESTRICT,
    group_id        SMALLSERIAL NOT NULL REFERENCES groups(id) ON DELETE RESTRICT,
    PRIMARY KEY     (guest_id, group_id)
);

-- Create `groups_permissions` table for many-to-many relationships between groups and permissions.
CREATE TABLE IF NOT EXISTS groups_permissions (
    group_id        SMALLSERIAL NOT NULL REFERENCES groups(id) ON DELETE RESTRICT,
    permission_id   BIGSERIAL NOT NULL REFERENCES permissions(id) ON DELETE RESTRICT,
    PRIMARY KEY     (group_id, permission_id)
);

-- Create `guests_permissions` table for many-to-many relationships between guests and permissions.
CREATE TABLE IF NOT EXISTS guests_permissions (
    guest_id        BIGSERIAL NOT NULL REFERENCES guests(id) ON DELETE RESTRICT,
    permission_id   SMALLSERIAL NOT NULL REFERENCES permissions(id) ON DELETE RESTRICT,
    PRIMARY KEY     (guest_id, permission_id)
);

-- # Fixture hydration.

INSERT INTO groups (name) VALUES ('guests'), ('admins'), ('naughty_guests');
INSERT INTO permissions (name) VALUES 
('addsignature'),
('deleteownsignature'),
('deleteanysignature'),
('editownsignature'),
('deleteuser'),
('markasnaughty'),
('prodemoteuser'),
('edituserpermissions');

-- All guests can read entries and leave their signature or delete their own entries
INSERT INTO groups_permissions (group_id, permission_id)
SELECT g.id, p.id
FROM groups g, permissions p
WHERE g.name = 'guests' AND p.name IN ('addsignature', 'deleteownsignature', 'editownsignature');

-- Admins can do everything
INSERT INTO groups_permissions (group_id, permission_id)
SELECT g.id, p.id
FROM groups g, permissions p
WHERE g.name = 'admins' AND p.name IN ('deleteanysignature', 'deleteuser', 'markasnaughty', 'prodemoteuser', 'edituserpermissions');
