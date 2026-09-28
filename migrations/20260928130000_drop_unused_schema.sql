DROP TABLE guests_permissions, groups_permissions, guests_groups, permissions, groups;
DROP TYPE permissionvariant, groupvariant;

DROP INDEX idx_guests_github_id;

ALTER TABLE guests ALTER COLUMN github_id DROP DEFAULT;
DROP SEQUENCE guests_github_id_seq;

ALTER TABLE guestbook ALTER COLUMN author_id DROP DEFAULT;
DROP SEQUENCE guestbook_author_id_seq;
