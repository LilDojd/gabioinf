ALTER TABLE guestbook ALTER COLUMN signature DROP DEFAULT;
UPDATE guestbook SET signature = NULL WHERE signature = '';
