--{{{( Types )

CREATE TYPE password AS (
    user_id         uuid,
    password        text
);

CREATE TYPE post_comment AS (
    comment_id      uuid,
    user_id         uuid,
    parent_id       uuid,
    indent          smallint,
    content         text,
    date_created    timestamptz
);

CREATE TYPE profile_name AS (
    name            text,
    aliases         text[]
);

CREATE TYPE profile_name_update AS (
    names           profile_name,
    old_name        text
);

CREATE TYPE user_session AS (
    user_id         uuid,
    expiration      timestamptz
);

--}}}

--{{{( Views )

CREATE VIEW object AS
SELECT
    object_id,
    preview_id,
    array_remove(array_agg(post_id ORDER BY date_added DESC), NULL) AS posts
FROM data.object
LEFT JOIN data.post_object USING (object_id)
GROUP BY object_id;

CREATE VIEW object_preview_error AS
SELECT
    object_id,
    message
FROM data.object_preview_error;

CREATE VIEW object_ref_view AS
SELECT
    object_id,
    count(objects) +
    count(previews) +
    count(icons) +
    count(avatars) +
    count(banners)
    AS reference_count
FROM data.object_ref
LEFT JOIN (
    SELECT object_id
    FROM data.object
) objects USING (object_id)
LEFT JOIN (
    SELECT preview_id AS object_id
    FROM data.object
) previews USING (object_id)
LEFT JOIN (
    SELECT icon AS object_id
    FROM data.site
) icons USING (object_id)
LEFT JOIN (
    SELECT avatar AS object_id
    FROM data.entity_profile
) avatars USING (object_id)
LEFT JOIN (
    SELECT banner AS object_id
    FROM data.entity_profile
) banners USING (object_id)
GROUP BY object_id;

CREATE VIEW source AS
SELECT
    source_id,
    scheme || '://' || host || resource AS url,
    icon
FROM data.source
JOIN data.site USING (site_id);

CREATE VIEW entity_link AS
SELECT
    profile_id,
    array_agg(source) AS sources
FROM data.entity_link
JOIN source USING (source_id)
GROUP BY profile_id;

CREATE VIEW entity_name AS
SELECT
    profile_id,
    name,
    coalesce(aliases, '{}') AS aliases
FROM (
    SELECT profile_id, name
    FROM data.entity_name
    WHERE main = true
) AS main
LEFT JOIN (
    SELECT profile_id, array_agg(name ORDER BY name) AS aliases
    FROM data.entity_name
    WHERE main = false
    GROUP BY profile_id
) AS alias USING (profile_id);

CREATE VIEW entity_profile AS
SELECT
    profile_id,
    name,
    aliases,
    description,
    coalesce(sources, '{}') AS sources,
    avatar,
    banner,
    created
FROM data.entity_profile
JOIN entity_name USING (profile_id)
LEFT JOIN entity_link USING (profile_id);

CREATE VIEW user_account AS
SELECT
    user_id,
    email,
    admin,
    name,
    aliases,
    description,
    sources,
    avatar,
    banner,
    created,
    coalesce(post_count, 0) AS post_count,
    coalesce(comment_count, 0) AS comment_count,
    coalesce(tag_count, 0) AS tag_count
FROM data.user_account
JOIN entity_profile ON user_id = profile_id
LEFT JOIN (
    SELECT
        poster AS user_id,
        count(*)::int4 AS post_count
    FROM data.post
    WHERE visibility = 'public'
    GROUP BY poster
) posts USING (user_id)
LEFT JOIN (
    SELECT
        user_id,
        count(*)::int4 AS comment_count
    FROM data.post_comment
    WHERE content <> ''
    GROUP BY user_id
) comments USING (user_id)
LEFT JOIN (
    SELECT
        creator AS user_id,
        count(*)::int4 AS tag_count
    FROM data.tag
    GROUP BY creator
) tags USING (user_id);

CREATE VIEW tag AS
SELECT
    tag_id,
    entity.name,
    aliases,
    description,
    sources,
    entity.avatar,
    banner,
    created,
    creator,
    coalesce(post_count, 0) AS post_count
FROM data.tag t
JOIN entity_profile entity ON tag_id = profile_id
LEFT JOIN (
    SELECT
        tag_id,
        count(*)::int4 AS post_count
    FROM data.post_tag
    GROUP BY tag_id
) p USING (tag_id);

CREATE FUNCTION read_related_posts(a_post_id uuid) RETURNS uuid[] AS $$
    SELECT array_agg(related ORDER BY title ASC, date_created DESC)
    FROM data.related_post r
    JOIN data.post p ON p.post_id = related
    GROUP BY r.post_id
    HAVING r.post_id = a_post_id;
$$ LANGUAGE SQL;

CREATE VIEW post AS
SELECT
    post_id,
    poster,
    title,
    description,
    objects,
    coalesce(read_related_posts(post_id), '{}') AS posts,
    coalesce(tags, '{}') AS tags,
    coalesce(comment_count, 0)::int4 AS comment_count,
    visibility,
    date_created,
    date_modified
FROM data.post
LEFT JOIN (
    SELECT post_id, array_agg(tag_id) AS tags
    FROM data.post_tag
    GROUP BY post_id
) tags USING (post_id)
LEFT JOIN (
    SELECT post_id, count(comment_id) AS comment_count
    FROM data.post_comment
    WHERE content <> ''
    GROUP BY post_id
) comments USING (post_id);

CREATE VIEW post_object_ref_view AS
SELECT
    object_id,
    count(post_objects.object_id) AS reference_count
FROM data.object
LEFT JOIN data.post_object post_objects USING (object_id)
GROUP BY object_id;

CREATE VIEW post_search AS
SELECT
    post_id,
    poster,
    title,
    description,
    visibility,
    date_created,
    date_modified,
    coalesce(array_agg(tag_id) FILTER (WHERE tag_id IS NOT NULL), '{}') AS tags
FROM data.post
LEFT JOIN data.post_tag USING (post_id)
GROUP BY post_id;

CREATE VIEW site_ref_view AS
SELECT
    site_id,
    count(sources) AS reference_count
FROM data.site
LEFT JOIN data.source sources USING (site_id)
GROUP BY site_id;

CREATE VIEW source_ref_view AS
SELECT
    source_id,
    count(object) + count(entity) AS reference_count
FROM data.source
LEFT JOIN data.object object USING (source_id)
LEFT JOIN data.entity_link entity USING (source_id)
GROUP BY source_id;

CREATE VIEW tag_search AS
SELECT
    tag_id,
    array_agg(name) AS names
FROM data.tag
JOIN data.entity_name ON tag_id = profile_id
GROUP BY tag_id, profile_id;

CREATE VIEW user_search AS
SELECT
    user_id,
    array_agg(name) AS names
FROM data.user_account
JOIN data.entity_name ON user_id = profile_id
GROUP BY user_id, profile_id;

--}}}

--{{{( Functions )

CREATE FUNCTION array_distinct(anyarray) RETURNS anyarray AS $$
    SELECT coalesce(array_agg(element.value ORDER BY element.ordinality), '{}')
    FROM (
        SELECT DISTINCT ON(value) unnest AS value, ordinality
        FROM unnest($1) WITH ORDINALITY
        ORDER BY value, ordinality
    ) element;
$$ LANGUAGE sql;

CREATE FUNCTION array_remove(array1 anyarray, array2 anyarray)
RETURNS anyarray AS $$
    SELECT coalesce(array_agg(element.value ORDER BY element.ordinality), '{}')
    FROM (
        SELECT ordinality, unnest AS value
        FROM unnest(array1) WITH ORDINALITY
    ) element
    WHERE element.value <> ALL(array2);
$$ LANGUAGE sql;

CREATE FUNCTION create_comment(a_user_id uuid, a_post_id uuid, a_content text)
RETURNS SETOF post_comment AS $$
    INSERT INTO data.post_comment (
        user_id,
        post_id,
        content
    ) VALUES (
        a_user_id,
        a_post_id,
        a_content
    ) RETURNING
        comment_id,
        user_id,
        parent_id,
        indent,
        content,
        date_created;
$$ LANGUAGE SQL;

CREATE FUNCTION create_entity(a_name text) RETURNS uuid AS $$
    WITH profile AS (
        INSERT INTO data.entity_profile DEFAULT VALUES RETURNING profile_id
    )
    INSERT INTO data.entity_name (profile_id, name, main)
    SELECT profile_id, a_name, true FROM profile
    RETURNING profile_id;
$$ LANGUAGE SQL;

CREATE FUNCTION create_entity_alias(a_profile_id uuid, a_alias text)
RETURNS SETOF profile_name AS $$
BEGIN
    INSERT INTO data.entity_name (profile_id, name)
    VALUES (a_profile_id, a_alias)
    ON CONFLICT DO NOTHING;

    RETURN QUERY
    SELECT name, aliases
    FROM entity_name
    WHERE profile_id = a_profile_id;

    EXCEPTION WHEN foreign_key_violation THEN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION create_entity_link(a_profile_id uuid, a_source_id bigint)
RETURNS void AS $$
BEGIN
    INSERT INTO data.entity_link(profile_id, source_id)
    VALUES (a_profile_id, a_source_id)
    ON CONFLICT DO NOTHING;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION create_object(
    a_object_id     uuid,
    a_preview_id    uuid,
    a_source_id     bigint
) RETURNS void AS $$
BEGIN
    PERFORM create_object_refs(ARRAY[a_object_id, a_preview_id]);

    INSERT INTO data.object (
        object_id,
        preview_id,
        source_id
    ) VALUES (
        a_object_id,
        a_preview_id,
        a_source_id
    ) ON CONFLICT (object_id) DO UPDATE SET
        preview_id = a_preview_id,
        source_id = a_source_id;

    DELETE FROM data.object_preview_error
    WHERE object_id = a_object_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION create_object_preview_error(
    a_object_id     uuid,
    a_message       text
) RETURNS void AS $$
BEGIN
    INSERT INTO data.object_preview_error (
        object_id,
        message
    ) VALUES (
        a_object_id,
        a_message
    ) ON CONFLICT (object_id)
    DO UPDATE SET message = a_message;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION create_object_refs(
    a_objects       uuid[]
) RETURNS void AS $$
BEGIN
    WITH object_table AS (
        SELECT unnest AS object_id
        FROM unnest(a_objects)
    )
    INSERT INTO data.object_ref (object_id)
    SELECT object_id
    FROM object_table
    WHERE object_id IS NOT NULL
    ON CONFLICT DO NOTHING;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION create_post(
    poster uuid,
    title text,
    description text,
    visibility data.visibility,
    objects uuid[],
    posts uuid[],
    tags uuid[]
) RETURNS SETOF post AS $$
DECLARE l_post_id uuid;
BEGIN
    INSERT INTO data.post (
        poster,
        title,
        description,
        objects,
        visibility
    ) VALUES (
        poster,
        coalesce(title, ''),
        coalesce(description, ''),
        coalesce(objects, '{}'),
        coalesce(visibility, 'public')
    )
    RETURNING post_id INTO l_post_id;

    INSERT INTO data.post_object (post_id, object_id)
    SELECT l_post_id, object_id
    FROM (
        SELECT unnest AS object_id
        FROM unnest(objects)
    ) obj;

    INSERT INTO data.related_post (post_id, related)
    SELECT l_post_id, related
    FROM (
        SELECT unnest AS related
        FROM unnest(posts)
    ) p;

    INSERT INTO data.post_tag (post_id, tag_id)
    SELECT l_post_id, tag_id
    FROM (
        SELECT unnest AS tag_id
        FROM unnest(tags)
    ) t;

    RETURN QUERY
    SELECT * FROM post WHERE post_id = l_post_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION create_post_objects(
    a_post_id uuid,
    a_objects uuid[],
    a_destination uuid
) RETURNS TABLE (modified timestamptz, objects uuid[]) AS $$
DECLARE l_position integer;
DECLARE l_tmp uuid[];
BEGIN
    INSERT INTO data.post_object (post_id, object_id)
    SELECT a_post_id, object_id
    FROM (
        SELECT unnest AS object_id
        FROM unnest(a_objects)
    ) obj
    ON CONFLICT DO NOTHING;

    l_tmp := (SELECT array_remove(
        (SELECT p.objects FROM data.post p WHERE post_id = a_post_id),
        a_objects
    ));

    l_position := (SELECT array_position(l_tmp, a_destination));

    UPDATE data.post p
    SET objects =
        l_tmp[0:(SELECT coalesce(l_position - 1, cardinality(l_tmp)))] ||
        array_distinct(a_objects) ||
        l_tmp[(SELECT coalesce(l_position, cardinality(l_tmp) + 1)):]
    WHERE p.post_id = a_post_id
    RETURNING p.objects INTO l_tmp;

    RETURN QUERY
    SELECT read_post_date_modified(a_post_id), l_tmp;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION create_post_tag(
    a_post_id       uuid,
    a_tag_id        uuid
) RETURNS void AS $$
BEGIN
    INSERT INTO data.post_tag (post_id, tag_id)
    VALUES (a_post_id, a_tag_id)
    ON CONFLICT DO NOTHING;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION create_related_post(a_post_id uuid, a_related uuid)
RETURNS uuid[] AS $$
BEGIN
    INSERT INTO data.related_post (post_id, related)
    VALUES (a_post_id, a_related)
    ON CONFLICT DO NOTHING;

    RETURN (SELECT read_related_posts(a_post_id));
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION create_reply(a_user_id uuid, a_parent_id uuid, a_content text)
RETURNS SETOF post_comment AS $$
    INSERT INTO data.post_comment(
        user_id,
        post_id,
        parent_id,
        indent,
        content
    )
    SELECT
        a_user_id,
        parent.post_id,
        a_parent_id,
        parent.indent + 1,
        a_content
    FROM data.post_comment parent
    WHERE comment_id = a_parent_id
    RETURNING
        comment_id,
        user_id,
        parent_id,
        indent,
        content,
        date_created;
$$ LANGUAGE SQL;

CREATE FUNCTION create_site(
    a_scheme        text,
    a_host          text,
    a_icon          uuid
) RETURNS SETOF data.site AS $$
BEGIN
    PERFORM create_object_refs(ARRAY[a_icon]);

    RETURN QUERY
    INSERT INTO data.site (
        scheme,
        host,
        icon
    ) VALUES (
        a_scheme,
        a_host,
        a_icon
    ) RETURNING *;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION create_source(
    a_site_id       bigint,
    a_resource      text
) RETURNS SETOF source AS $$
BEGIN
    INSERT INTO data.source (
        site_id,
        resource
    ) VALUES (
        a_site_id,
        a_resource
    ) ON CONFLICT DO NOTHING;

    RETURN QUERY
    SELECT
        source_id,
        scheme || '://' || host || resource AS url,
        icon
    FROM data.source
    JOIN data.site USING (site_id)
    WHERE site_id = a_site_id AND resource = a_resource;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION create_tag(a_name text, a_creator uuid) RETURNS SETOF tag AS $$
DECLARE l_tag_id uuid;
BEGIN
    INSERT INTO data.tag (tag_id, creator)
    SELECT create_entity(a_name), a_creator
    RETURNING tag_id INTO l_tag_id;

    RETURN QUERY
    SELECT * FROM read_tag(l_tag_id);
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION create_user(
    a_name text,
    a_email text,
    a_password text
) RETURNS SETOF user_account AS $$
DECLARE l_user_id uuid;
BEGIN
    INSERT INTO data.user_account (user_id, email, password)
    SELECT create_entity(a_name), a_email, a_password
    RETURNING user_id INTO l_user_id;

    RETURN QUERY
    SELECT * FROM read_user(l_user_id);
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION create_user_session(
    a_user_id uuid,
    a_session_id bytea,
    a_expiration timestamptz
) RETURNS void AS $$
    INSERT INTO data.user_session (session_id, user_id, expiration)
    VALUES (a_session_id, a_user_id, a_expiration);
$$ LANGUAGE SQL;

CREATE FUNCTION delete_comment(a_comment_id uuid, recursive boolean)
RETURNS boolean AS $$
DECLARE
    l_parent_id uuid;
    l_found bool;
BEGIN
    IF
        NOT recursive AND
        EXISTS (SELECT FROM data.post_comment WHERE parent_id = a_comment_id)
    THEN
        UPDATE data.post_comment SET content = ''
        WHERE comment_id = a_comment_id;

        RETURN FOUND;
    ELSE
        WITH deleted AS (
            DELETE FROM data.post_comment WHERE comment_id = a_comment_id
            RETURNING parent_id
        ) SELECT INTO l_parent_id parent_id FROM deleted;

        l_found := FOUND;

        PERFORM prune_comments(l_parent_id);
    END IF;

    RETURN l_found;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION delete_entity(a_profile_id uuid) RETURNS boolean AS $$
BEGIN
    DELETE FROM data.entity_profile
    WHERE profile_id = a_profile_id;

    RETURN FOUND;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION delete_entity_alias(a_profile_id uuid, a_alias text)
RETURNS SETOF profile_name AS $$
BEGIN
    DELETE FROM data.entity_name
    WHERE profile_id = a_profile_id
        AND main = false
        AND name = a_alias;

    RETURN QUERY
    SELECT name, aliases
    FROM entity_name
    WHERE profile_id = a_profile_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION delete_entity_link(a_profile_id uuid, a_source_id bigint)
RETURNS boolean AS $$
BEGIN
    DELETE FROM data.entity_link
    WHERE profile_id = a_profile_id AND source_id = a_source_id;

    RETURN FOUND;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION delete_post(
    a_post_id       uuid
) RETURNS boolean AS $$
BEGIN
    DELETE FROM data.post
    WHERE post_id = a_post_id;

    RETURN FOUND;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION delete_post_objects(a_post_id uuid, a_objects uuid[])
RETURNS timestamptz AS $$
BEGIN
    UPDATE data.post p
    SET objects = array_remove(p.objects, a_objects)
    WHERE p.post_id = a_post_id;

    DELETE FROM data.post_object po
    WHERE po.post_id = a_post_id AND object_id = ANY(a_objects);

    RETURN read_post_date_modified(a_post_id);
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION delete_post_tag(
    a_post_id       uuid,
    a_tag_id        uuid
) RETURNS boolean AS $$
BEGIN
    DELETE FROM data.post_tag
    WHERE post_id = a_post_id AND tag_id = a_tag_id;

    RETURN FOUND;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION delete_related_post(a_post_id uuid, a_related uuid)
RETURNS uuid[] AS $$
BEGIN
    DELETE FROM data.related_post
    WHERE post_id = a_post_id AND related = a_related;

    IF FOUND THEN
        RETURN (SELECT coalesce(read_related_posts(a_post_id), '{}'));
    ELSE
        RETURN NULL;
    END IF;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION delete_user_session(a_session_id bytea) RETURNS void AS $$
    DELETE FROM data.user_session
    WHERE session_id = a_session_id;
$$ LANGUAGE SQL;

CREATE FUNCTION prune() RETURNS void AS $$
BEGIN
    PERFORM prune_post_objects();
    PERFORM prune_sessions();
    PERFORM prune_sources();
    PERFORM prune_sites();
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION prune_comments(a_comment_id uuid) RETURNS void AS $$
BEGIN
    WHILE
        a_comment_id IS NOT NULL AND
        (
            SELECT length(content)
            FROM data.post_comment
            WHERE comment_id = a_comment_id
        ) = 0 AND
        NOT EXISTS (
            SELECT FROM data.post_comment
            WHERE parent_id = a_comment_id
        ) LOOP
        WITH deleted AS (
            DELETE FROM data.post_comment WHERE comment_id = a_comment_id
            RETURNING parent_id
        )
        SELECT INTO a_comment_id parent_id FROM deleted;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION prune_objects() RETURNS SETOF uuid AS $$
BEGIN
    RETURN QUERY
    DELETE FROM data.object_ref obj USING object_ref_view ref
    WHERE obj.object_id = ref.object_id AND ref.reference_count = 0
    RETURNING obj.object_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION prune_post_objects() RETURNS void AS $$
BEGIN
    DELETE FROM data.object obj USING post_object_ref_view ref
    WHERE obj.object_id = ref.object_id AND ref.reference_count = 0;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION prune_sessions() RETURNS void AS $$
    DELETE FROM data.user_session WHERE expiration <= now();
$$ LANGUAGE SQL;

CREATE FUNCTION prune_sites() RETURNS void AS $$
BEGIN
    DELETE FROM data.site site USING site_ref_view ref
    WHERE site.site_id = ref.site_id AND ref.reference_count = 0;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION prune_sources() RETURNS void AS $$
BEGIN
    DELETE FROM data.source source USING source_ref_view ref
    WHERE source.source_id = ref.source_id AND ref.reference_count = 0;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION publish_post(draft_id uuid) RETURNS timestamptz AS $$
DECLARE
    created CONSTANT timestamptz = now();
BEGIN
    UPDATE data.post
    SET
        visibility = 'public',
        date_created = created,
        date_modified = created
    WHERE post_id = draft_id AND visibility = 'draft';

    IF NOT FOUND THEN
        RAISE 'Draft with ID (%) does not exist', draft_id
        USING ERRCODE = 'no_data_found';
    END IF;

    RETURN created;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_comment_post(a_comment_id uuid) RETURNS uuid AS $$
    SELECT post_id FROM data.post_comment WHERE comment_id = a_comment_id;
$$ LANGUAGE SQL;

CREATE FUNCTION read_comments(a_post_id uuid)
RETURNS SETOF post_comment AS $$
    SELECT
        comment_id,
        user_id,
        parent_id,
        indent,
        content,
        date_created
    FROM data.post_comment
    WHERE post_id = a_post_id
    ORDER BY
        indent,
        parent_id,
        date_created;
$$ LANGUAGE SQL;

CREATE FUNCTION read_entity_sources(a_profile_id uuid)
RETURNS SETOF source AS $$
BEGIN
    RETURN QUERY
    SELECT source.*
    FROM data.entity_link
    JOIN source USING (source_id)
    WHERE profile_id = a_profile_id
    ORDER BY url;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_object_preview_errors()
RETURNS SETOF object_preview_error AS $$
BEGIN
    RETURN QUERY
    SELECT *
    FROM object_preview_error
    ORDER BY object_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_post(
    a_post_id       uuid
) RETURNS SETOF post AS $$
BEGIN
    RETURN QUERY
    SELECT *
    FROM post
    WHERE post_id = a_post_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_post_date_modified(
    a_post_id       uuid
) RETURNS timestamptz AS $$
DECLARE
    l_date_modified timestamptz;
BEGIN
    SELECT date_modified INTO l_date_modified
    FROM data.post
    WHERE post_id = a_post_id;

    RETURN l_date_modified;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_posts(a_posts uuid[]) RETURNS SETOF post AS $$
    SELECT post.*
    FROM (
        SELECT *
        FROM post
        WHERE post_id = ANY(a_posts)
    ) post
    JOIN (
        SELECT
            ordinality,
            unnest AS post_id
        FROM unnest(a_posts) WITH ORDINALITY
    ) list USING (post_id)
    ORDER BY ordinality;
$$ LANGUAGE SQL;

CREATE FUNCTION read_post_search() RETURNS SETOF post_search AS $$
BEGIN
    RETURN QUERY
    SELECT *
    FROM post_search
    ORDER BY post_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_site(
    a_scheme        text,
    a_host          text
) RETURNS bigint AS $$
DECLARE
    result          bigint;
BEGIN
    SELECT INTO result site_id
    FROM data.site
    WHERE host = a_host AND scheme = a_scheme;

    RETURN result;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_tag(
    a_tag_id        uuid
) RETURNS SETOF tag AS $$
BEGIN
    RETURN QUERY
    SELECT *
    FROM tag
    WHERE tag_id = a_tag_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_tag_search()
RETURNS SETOF tag_search AS $$
BEGIN
    RETURN QUERY
    SELECT *
    FROM tag_search
    ORDER BY tag_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_tags(a_tags uuid[]) RETURNS SETOF tag AS $$
    SELECT tag.*
    FROM (
        SELECT
            ordinality,
            unnest AS tag_id
        FROM unnest(a_tags) WITH ORDINALITY
    ) tags
    JOIN tag USING (tag_id)
    ORDER BY ordinality;
$$ LANGUAGE SQL;

CREATE FUNCTION read_user(a_user_id uuid) RETURNS SETOF user_account AS $$
    SELECT * FROM user_account WHERE user_id = a_user_id;
$$ LANGUAGE SQL;

CREATE FUNCTION read_user_password(a_email text) RETURNS SETOF password AS $$
    SELECT user_id, password FROM data.user_account WHERE email = a_email;
$$ LANGUAGE SQL;

CREATE FUNCTION read_user_search()
RETURNS SETOF user_search AS $$
BEGIN
    RETURN QUERY
    SELECT *
    FROM user_search
    ORDER BY user_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_user_session(a_session_id bytea)
RETURNS SETOF user_session AS $$
    SELECT user_id, expiration
    FROM data.user_session
    WHERE session_id = a_session_id AND expiration > now();
$$ LANGUAGE SQL;

CREATE FUNCTION read_users(a_users uuid[]) RETURNS SETOF user_account AS $$
    SELECT user_account.*
    FROM (
        SELECT
            ordinality,
            unnest AS user_id
        FROM unnest(a_users) WITH ORDINALITY
    ) users
    JOIN user_account USING (user_id)
    ORDER BY ordinality;
$$ LANGUAGE SQL;

CREATE FUNCTION read_object(a_object_id uuid) RETURNS SETOF object AS $$
BEGIN
    RETURN QUERY
    SELECT *
    FROM object
    WHERE object_id = a_object_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_object_total() RETURNS int8 AS $$
BEGIN
    RETURN (SELECT count(*) FROM data.object);
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_objects(a_objects uuid[]) RETURNS SETOF object AS $$
    SELECT object.*
    FROM (
        SELECT
            ordinality,
            unnest AS object_id
        FROM unnest(a_objects) WITH ORDINALITY
    ) objects
    JOIN object USING (object_id)
    ORDER BY ordinality;
$$ LANGUAGE SQL;

CREATE FUNCTION read_post_total() RETURNS int8 AS $$
BEGIN
    RETURN (SELECT count(*) FROM data.post);
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_tag_total() RETURNS int8 AS $$
BEGIN
    RETURN (SELECT count(*) FROM data.tag);
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_user_total() RETURNS int8 AS $$
BEGIN
    RETURN (SELECT count(*) FROM data.user_account);
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION stream_objects()
RETURNS SETOF object AS $$
BEGIN
    RETURN QUERY
    SELECT *
    FROM object
    ORDER BY object_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION update_admin(a_user_id uuid, a_admin boolean)
RETURNS boolean AS $$
BEGIN
    UPDATE data.user_account
    SET admin = a_admin
    WHERE user_id = a_user_id;

    RETURN FOUND;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION update_comment(
    a_comment_id    uuid,
    a_content       text
) RETURNS bool AS $$
BEGIN
    UPDATE data.post_comment
    SET content = a_content
    WHERE comment_id = a_comment_id;

    RETURN FOUND;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION update_entity_description(a_profile_id uuid, a_description text)
RETURNS boolean AS $$
BEGIN
    UPDATE data.entity_profile
    SET description = a_description
    WHERE profile_id = a_profile_id;

    RETURN FOUND;
END;
$$ LANGUAGE plpgsql;

/**
 * Updates an entity's main name.
 * If the new name already exists as an alias, the alias and the main name are
 * swapped. Otherwise, the main name is replaced with the new value.
 */
CREATE FUNCTION update_entity_name(a_profile_id uuid, a_name text)
RETURNS SETOF profile_name_update AS $$
DECLARE l_old_name text;
BEGIN
    IF EXISTS (
        SELECT FROM data.entity_name
        WHERE profile_id = a_profile_id AND name = a_name AND main = false
    ) THEN
        UPDATE data.entity_name
        SET main = false
        WHERE profile_id = a_profile_id AND main = true;

        UPDATE data.entity_name
        SET main = true
        WHERE profile_id = a_profile_id AND name = a_name;
    ELSE
        SELECT INTO l_old_name name
        FROM data.entity_name
        WHERE profile_id = a_profile_id AND main = true;

        UPDATE data.entity_name
        SET name = a_name
        WHERE profile_id = a_profile_id AND main = true;
    END IF;

    RETURN QUERY
    SELECT
        ROW(name, aliases)::profile_name,
        l_old_name
    FROM entity_name
    WHERE profile_id = a_profile_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION update_object_preview(
    a_object_id     uuid,
    a_preview_id    uuid
) RETURNS void AS $$
BEGIN
    PERFORM create_object_refs(ARRAY[a_preview_id]);

    UPDATE data.object
    SET preview_id = a_preview_id
    WHERE object_id = a_object_id;

    DELETE FROM data.object_preview_error
    WHERE object_id = a_object_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION update_post_description(a_post_id uuid, a_description text)
RETURNS timestamptz AS $$
DECLARE result timestamptz;
BEGIN
    WITH updated AS (
        UPDATE data.post
        SET description = a_description
        WHERE post_id = a_post_id
        RETURNING date_modified
    )
    SELECT INTO result date_modified
    FROM updated;

    RETURN result;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION update_post_title(a_post_id uuid, a_title text)
RETURNS timestamptz AS $$
DECLARE result timestamptz;
BEGIN
    WITH updated AS (
        UPDATE data.post
        SET title = a_title
        WHERE post_id = a_post_id
        RETURNING date_modified
    )
    SELECT INTO result date_modified
    FROM updated;

    RETURN result;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION update_user_email(a_user_id uuid, a_email text)
RETURNS bool AS $$
BEGIN
    UPDATE data.user_account
    SET email = a_email
    WHERE user_id = a_user_id;

    RETURN FOUND;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION update_user_password(a_user_id uuid, a_password text)
RETURNS bool AS $$
BEGIN
    UPDATE data.user_account
    SET password = a_password
    WHERE user_id = a_user_id;

    RETURN FOUND;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION import_entity(data jsonb) RETURNS void AS $$
BEGIN
    PERFORM create_object_refs(ARRAY[avatar, banner])
    FROM jsonb_to_recordset(data) AS (avatar uuid, banner uuid);

    INSERT INTO data.entity_profile (
        profile_id,
        description,
        avatar,
        banner,
        created
    )
    SELECT *
    FROM jsonb_to_recordset(data) AS (
        id uuid,
        description text,
        avatar uuid,
        banner uuid,
        created timestamptz
    );

    INSERT INTO data.entity_name (profile_id, name, main)
    SELECT id, name, true
    FROM jsonb_to_recordset(data) AS (id uuid, name text);

    INSERT INTO data.entity_name (profile_id, name)
    SELECT id, unnest(aliases)
    FROM jsonb_to_recordset(data) AS (id uuid, aliases text[]);
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION import(data jsonb) RETURNS void AS $$
BEGIN
    PERFORM import_entity(data -> 'users');

    INSERT INTO data.user_account (user_id, email, password, admin)
    SELECT id, email, password, admin
    FROM jsonb_to_recordset(data -> 'users') AS (
        id uuid,
        email text,
        password text,
        admin boolean
    );

    PERFORM import_entity(data -> 'tags');

    INSERT INTO data.tag (tag_id, creator)
    SELECT id, creator
    FROM jsonb_to_recordset(data -> 'tags') AS (id uuid, creator uuid);

    INSERT INTO data.post (
        post_id,
        poster,
        title,
        description,
        objects,
        visibility,
        date_created,
        date_modified
    )
    SELECT *
    FROM jsonb_to_recordset(data -> 'posts') AS (
        id uuid,
        poster uuid,
        title text,
        description text,
        objects uuid[],
        visibility data.visibility,
        created timestamptz,
        modified timestamptz
    );

    INSERT INTO data.post_object (post_id, object_id)
    SELECT id, unnest(objects)
    FROM jsonb_to_recordset(data -> 'posts') AS (id uuid, objects uuid[]);

    INSERT INTO data.related_post (post_id, related)
    SELECT id, unnest(posts)
    FROM jsonb_to_recordset(data -> 'posts') AS (id uuid, posts uuid[]);

    INSERT INTO data.post_tag (post_id, tag_id)
    SELECT id, unnest(tags)
    FROM jsonb_to_recordset(data -> 'posts') AS (id uuid, tags uuid[]);

    INSERT INTO data.post_comment (
        comment_id,
        user_id,
        post_id,
        parent_id,
        indent,
        content,
        date_created
    )
    SELECT comment_id, "user", post_id, parent_id, indent, content, created
    FROM jsonb_to_recordset(data -> 'posts') AS (id uuid, comments jsonb),
        LATERAL (SELECT id AS post_id),
        LATERAL (
            SELECT id AS comment_id, *
            FROM jsonb_to_recordset(comments) AS (
                id uuid,
                "user" uuid,
                parent_id uuid,
                indent smallint,
                content text,
                created timestamptz
            )
        );
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION export() RETURNS json AS $$
SELECT json_build_object(
    'posts', (
        SELECT (coalesce((
            SELECT json_agg(p)
            FROM (
                SELECT
                    post_id AS id,
                    poster,
                    title,
                    description,
                    visibility,
                    date_created AS created,
                    date_modified AS modified,
                    objects,
                    coalesce(posts, '[]'::json) AS posts,
                    coalesce(tags, '[]'::json) AS tags,
                    coalesce(comments, '[]'::Json) AS comments
                FROM data.post p
                LEFT JOIN (
                    SELECT
                        post_id,
                        json_agg(related) AS posts
                    FROM data.related_post
                    GROUP BY post_id
                ) r USING (post_id)
                LEFT JOIN (
                    SELECT
                        post_id,
                        json_agg(tag_id) AS tags
                    FROM data.post_tag t
                    GROUP BY post_id
                ) t USING (post_id)
                LEFT JOIN (
                    SELECT
                        post_id,
                        json_agg(json_build_object(
                            'id', comment_id,
                            'user', user_id,
                            'parent_id', parent_id,
                            'indent', indent,
                            'content', content,
                            'created', date_created
                        ) ORDER BY
                            indent,
                            parent_id,
                            date_created DESC
                        ) AS comments
                    FROM data.post_comment c
                    GROUP by post_id
                ) c USING (post_id)
                ORDER BY date_created
            ) p
        ), '[]'::json))
    ),
    'tags', (
        SELECT (coalesce((
            SELECT json_agg(t)
            FROM (
                SELECT
                    tag_id AS id,
                    name,
                    aliases,
                    description,
                    sources,
                    avatar,
                    banner,
                    created,
                    creator
                FROM tag
                ORDER BY name
            ) t
        ), '[]'::json))
    ),
    'users', (
        SELECT (coalesce((
            SELECT json_agg(u)
            FROM (
                SELECT
                    user_id AS id,
                    email,
                    password,
                    admin,
                    name,
                    aliases,
                    description,
                    sources,
                    avatar,
                    banner,
                    created
                FROM data.user_account
                JOIN entity_profile on user_id = profile_id
                ORDER BY name
            ) u
        ), '[]'::json))
    )
)
$$ LANGUAGE SQL;

--}}}

--{{{( Trigger Functions )

CREATE FUNCTION update_date_modified() RETURNS trigger AS $$
BEGIN
    IF OLD <> NEW THEN
        NEW.date_modified = NOW();
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

--}}}

--{{{( Triggers )

CREATE TRIGGER update_post_date_modified BEFORE UPDATE ON data.post
FOR EACH ROW EXECUTE FUNCTION update_date_modified();

--}}}
