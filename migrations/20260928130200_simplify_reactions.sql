ALTER TABLE reactions DROP COLUMN target_kind;
DROP TYPE reaction_target;
DROP INDEX reactions_post_slug_idx;
ALTER TABLE reactions ADD UNIQUE NULLS NOT DISTINCT (post_slug, comment_id, guest_id, emoji);
