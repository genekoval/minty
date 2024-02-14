CREATE TYPE visibility AS ENUM (
    'draft',
    'public'
);

ALTER TABLE post
ADD COLUMN visibility visibility;

UPDATE post
SET visibility = 'public';

ALTER TABLE post
ALTER COLUMN visibility SET NOT NULL;
