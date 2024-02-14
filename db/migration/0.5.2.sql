ALTER TABLE post_comment
DROP CONSTRAINT post_comment_parent_id_fkey;

ALTER TABLE post_comment
ADD CONSTRAINT post_comment_parent_id_fkey FOREIGN KEY (parent_id)
REFERENCES post_comment ON DELETE CASCADE;
