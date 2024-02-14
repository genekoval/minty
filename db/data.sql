CREATE TYPE visibility AS ENUM (
    'draft',
    'public'
);

CREATE TABLE object_ref (
    object_id       uuid PRIMARY KEY
);

CREATE TABLE site (
    site_id         bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    scheme          text NOT NULL,
    host            text UNIQUE NOT NULL,
    icon            uuid REFERENCES object_ref ON DELETE NO ACTION
);

CREATE TABLE source (
    source_id       bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    resource        text NOT NULL,
    site_id         bigint NOT NULL REFERENCES site ON DELETE NO ACTION,

    UNIQUE (site_id, resource)
);

CREATE TABLE object (
    object_id       uuid PRIMARY KEY REFERENCES object_ref ON DELETE NO ACTION,
    preview_id      uuid REFERENCES object_ref ON DELETE NO ACTION,
    source_id       bigint REFERENCES source ON DELETE NO ACTION
);

CREATE TABLE object_preview_error (
    object_id       uuid PRIMARY KEY REFERENCES object ON DELETE CASCADE,
    message         text NOT NULL
);

CREATE TABLE post (
    post_id         uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    title           text NOT NULL DEFAULT '',
    description     text NOT NULL DEFAULT '',
    objects         uuid[] NOT NULL DEFAULT '{}',
    visibility      visibility NOT NULL,
    date_created    timestamptz NOT NULL DEFAULT NOW(),
    date_modified   timestamptz NOT NULL DEFAULT NOW()
);

CREATE TABLE post_object (
    post_id         uuid NOT NULL REFERENCES post ON DELETE CASCADE,
    object_id       uuid NOT NULL REFERENCES object ON DELETE CASCADE,
    date_added      timestamptz NOT NULL DEFAULT NOW(),

    PRIMARY KEY (post_id, object_id)
);

CREATE TABLE related_post (
    post_id         uuid NOT NULL REFERENCES post ON DELETE CASCADE,
    related         uuid NOT NULL REFERENCES post ON DELETE CASCADE,

    PRIMARY KEY (post_id, related)
);

CREATE TABLE post_comment (
    comment_id      uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    post_id         uuid NOT NULL REFERENCES post ON DELETE CASCADE,
    parent_id       uuid REFERENCES post_comment ON DELETE CASCADE,
    indent          smallint NOT NULL DEFAULT 0,
    content         text NOT NULL,
    date_created    timestamptz NOT NULL DEFAULT NOW()
);

CREATE TABLE tag (
    tag_id          uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    description     text NOT NULL DEFAULT '',
    avatar          uuid REFERENCES object_ref ON DELETE NO ACTION,
    banner          uuid REFERENCES object_ref ON DELETE NO ACTION,
    date_created    timestamptz NOT NULL DEFAULT NOW()
);

CREATE TABLE tag_name (
    tag_id          uuid NOT NULL REFERENCES tag ON DELETE CASCADE,
    value           text NOT NULL,
    main            boolean NOT NULL DEFAULT false,

    PRIMARY KEY (tag_id, value)
);

CREATE TABLE post_tag (
    post_id         uuid NOT NULL REFERENCES post ON DELETE CASCADE,
    tag_id          uuid NOT NULL REFERENCES tag ON DELETE CASCADE,

    PRIMARY KEY (post_id, tag_id)
);

CREATE TABLE tag_source (
    tag_id          uuid NOT NULL REFERENCES tag ON DELETE CASCADE,
    source_id       bigint NOT NULL REFERENCES source ON DELETE NO ACTION,

    PRIMARY KEY (tag_id, source_id)
);
