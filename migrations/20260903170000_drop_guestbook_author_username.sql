-- The author's username is read from `guests` via `author_id`, so a GitHub
-- rename no longer breaks sign-in on the old `guests(username)` foreign key.
ALTER TABLE guestbook DROP COLUMN author_username;
