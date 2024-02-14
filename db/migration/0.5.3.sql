UPDATE post
SET title = ''
WHERE title IS NULL;

UPDATE post
SET description = ''
WHERE description IS NULL;

ALTER TABLE post
ALTER COLUMN title SET NOT NULL,
ALTER COLUMN title SET DEFAULT '',
ALTER COLUMN description SET NOT NULL,
ALTER COLUMN description SET DEFAULT '';

UPDATE tag
SET description = ''
WHERE description IS NULL;

ALTER TABLE tag
ALTER COLUMN description SET NOT NULL,
ALTER COLUMN description SET DEFAULT '';
