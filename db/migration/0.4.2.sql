ALTER TABLE post
ADD COLUMN objects uuid[] NOT NULL DEFAULT '{}';

UPDATE post
SET objects = coalesce((
    SELECT array_agg(object_id ORDER BY sequence)
    FROM post_object
    WHERE post_object.post_id = post.post_id
    GROUP BY post_id
), '{}'::uuid[]);

ALTER TABLE post_object
DROP COLUMN sequence;
