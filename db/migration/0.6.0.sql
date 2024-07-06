CREATE TABLE entity_profile (
    profile_id      uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    description     text NOT NULL DEFAULT '',
    avatar          uuid REFERENCES object_ref ON DELETE NO ACTION,
    banner          uuid REFERENCES object_ref ON DELETE NO ACTION,
    created         timestamptz NOT NULL DEFAULT NOW()
);

CREATE TABLE entity_name (
    profile_id      uuid NOT NULL REFERENCES entity_profile ON DELETE CASCADE,
    name            text NOT NULL,
    main            boolean NOT NULL DEFAULT false,

    PRIMARY KEY (profile_id, name)
);

CREATE TABLE entity_link (
    profile_id      uuid NOT NULL REFERENCES entity_profile ON DELETE CASCADE,
    source_id       bigint NOT NULL REFERENCES source ON DELETE NO ACTION,

    PRIMARY KEY (profile_id, source_id)
);

CREATE TABLE user_account (
    user_id         uuid PRIMARY KEY REFERENCES entity_profile ON DELETE CASCADE,
    email           text UNIQUE NOT NULL,
    password        text NOT NULL,
    admin           boolean NOT NULL DEFAULT false
);

CREATE TABLE user_session (
    session_id      bytea PRIMARY KEY,
    user_id         uuid NOT NULL REFERENCES user_account ON DELETE CASCADE,
    expiration      timestamptz NOT NULL
);

WITH profile AS (
    INSERT INTO entity_profile DEFAULT VALUES RETURNING profile_id
), name AS (
    INSERT INTO entity_name (profile_id, name, main)
    SELECT profile_id, 'minty', true
    FROM profile
)
INSERT INTO user_account (user_id, email, password, admin)
SELECT
    profile_id,
    'minty@example.com',
    -- password is 'password'
    '$argon2id$v=19$m=19456,t=2,p=1$l5wX+Rot3RRzyN4JFJMHKQ$mTZbLtNo3zyfJinzsgqgYdSuj/N1jz6i1AWLCGOna2o',
    true
FROM profile;

INSERT INTO entity_profile (profile_id, description, avatar, banner, created)
SELECT tag_id, description, avatar, banner, date_created
FROM tag;

INSERT INTO entity_name (profile_id, name, main)
SELECT tag_id, value, main
FROM tag_name;

INSERT INTO entity_link (profile_id, source_id)
SELECT tag_id, source_id
FROM tag_source
JOIN tag USING (tag_id);

ALTER TABLE tag
ADD CONSTRAINT tag_id_fkey FOREIGN KEY (tag_id)
    REFERENCES entity_profile(profile_id) ON DELETE CASCADE,
ADD COLUMN creator uuid REFERENCES user_account ON DELETE SET NULL,
DROP COLUMN description,
DROP COLUMN avatar,
DROP COLUMN banner,
DROP COLUMN date_created;

DROP TABLE tag_name;
DROP TABLE tag_source;

UPDATE tag
SET creator = (SELECT user_id FROM user_account);

ALTER TABLE post
ADD COLUMN poster uuid REFERENCES user_account ON DELETE SET NULL;

UPDATE post
SET poster = (SELECT user_id FROM user_account);

ALTER TABLE post_comment
ADD COLUMN user_id uuid REFERENCES user_account ON DELETE SET NULL;

UPDATE post_comment
SET user_id = (SELECT user_id FROM user_account);
