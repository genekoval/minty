--{{{( Types )

CREATE TYPE object_preview AS (
    object_id       uuid,
    preview_id      uuid
);

CREATE TYPE tag_name AS (
    name            text,
    aliases         text[]
);

CREATE TYPE tag_name_update AS (
    names           tag_name,
    old_name        text
);

--}}}

CREATE FUNCTION read_object_previews(objects uuid[])
RETURNS object_preview[] AS $$
DECLARE result object_preview[];
BEGIN
    SELECT INTO result
        array_agg(
            ROW(object_id, preview_id)::object_preview
            ORDER BY ordinality
        ) AS objects
    FROM (
        SELECT unnest as object_id, ordinality
        FROM unnest(objects) WITH ORDINALITY
    ) obj
    JOIN data.object USING (object_id);

    RETURN result;
END;
$$ LANGUAGE plpgsql;

--{{{( Views )

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
    FROM data.tag
) avatars USING (object_id)
LEFT JOIN (
    SELECT banner AS object_id
    FROM data.tag
) banners USING (object_id)
GROUP BY object_id;

CREATE VIEW post AS
SELECT
    post_id,
    title,
    description,
    read_object_previews(objects) AS objects,
    coalesce(comment_count, 0)::int4 AS comment_count,
    visibility,
    date_created,
    date_modified
FROM data.post
LEFT JOIN (
    SELECT post_id, count(comment_id) AS comment_count
    FROM data.post_comment
    WHERE content <> ''
    GROUP BY post_id
) comments USING (post_id);

CREATE VIEW post_comment AS
SELECT
    comment_id,
    post_id,
    parent_id,
    indent,
    content,
    date_created
FROM data.post_comment;

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
    title,
    description,
    visibility,
    date_created,
    date_modified,
    array_agg(tag_id) FILTER (WHERE tag_id IS NOT NULL) AS tags
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
    count(object) + count(tag) AS reference_count
FROM data.source
LEFT JOIN data.object object USING (source_id)
LEFT JOIN data.tag_source tag USING (source_id)
GROUP BY source_id;

CREATE VIEW source AS
SELECT
    source_id,
    resource,
    site
FROM data.source
JOIN data.site site USING (site_id);

CREATE VIEW tag_name_view AS
SELECT
    tag_id,
    name,
    aliases
FROM (
    SELECT
        tag_id,
        value AS name
    FROM data.tag_name
    WHERE main = true
) AS main
LEFT JOIN (
    SELECT
        tag_id,
        array_agg(value ORDER BY value) AS aliases
    FROM data.tag_name
    WHERE main = false
    GROUP BY tag_id
) AS alias USING (tag_id);

CREATE VIEW tag_preview AS
SELECT
    tag_id,
    name,
    avatar
FROM data.tag
LEFT JOIN (
    SELECT
        tag_id,
        value AS name
    FROM data.tag_name
    WHERE main = true
) AS main USING (tag_id);

CREATE VIEW tag_search AS
SELECT
    tag_id,
    array_agg(value) AS names
FROM data.tag_name
GROUP BY tag_id;

CREATE VIEW tag AS
SELECT
    tag_id,
    name,
    aliases,
    description,
    avatar,
    banner,
    count(post_id)::int4 AS post_count,
    date_created
FROM data.tag
LEFT JOIN tag_name_view USING (tag_id)
LEFT JOIN data.post_tag USING (tag_id)
GROUP BY tag_id, name, aliases;

CREATE VIEW object AS
SELECT
    object_id,
    preview_id,
    source
FROM data.object
LEFT JOIN source USING (source_id);

CREATE VIEW post_preview AS
SELECT
    post_id,
    title,
    preview,
    coalesce(comment_count, 0)::int4 AS comment_count,
    coalesce(object_count, 0)::int4 AS object_count,
    post.date_created AS date_created
FROM data.post
LEFT JOIN (
    SELECT
        post_id,
        count(comment_id) comment_count
    FROM data.post_comment
    WHERE content <> ''
    GROUP BY post_id
) comments USING (post_id)
LEFT JOIN (
    SELECT
        post_id,
        count(object_id) AS object_count
    FROM data.post_object
    GROUP BY post_id
) objects USING (post_id)
LEFT JOIN (
    SELECT
        post_id,
        ROW(object_id, preview_id)::object_preview AS preview
    FROM (
        SELECT
            post_id,
            objects[1] AS object_id
        FROM data.post
    ) post
    JOIN data.object USING (object_id)
) previews USING (post_id);

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

CREATE FUNCTION create_comment(
    a_post_id       uuid,
    a_content       text
) RETURNS SETOF post_comment AS $$
BEGIN
    RETURN QUERY
    INSERT INTO data.post_comment (
        post_id,
        content
    ) VALUES (
        a_post_id,
        a_content
    ) RETURNING
        comment_id,
        post_id,
        parent_id,
        indent,
        content,
        date_created AS date_created;
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

CREATE FUNCTION create_post(draft_id uuid) RETURNS timestamptz AS $$
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

CREATE FUNCTION create_post_draft()
RETURNS TABLE (id uuid, created timestamptz) AS $$
BEGIN
    RETURN QUERY
    INSERT INTO data.post (visibility) VALUES ('draft')
    RETURNING post_id, date_created;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION create_post_objects(
    post_id uuid,
    objects uuid[],
    destination uuid
) RETURNS timestamptz AS $$
DECLARE position integer;
DECLARE tmp uuid[];
BEGIN
    INSERT INTO data.post_object (post_id, object_id)
    SELECT create_post_objects.post_id, object_id
    FROM (
        SELECT unnest AS object_id
        FROM unnest(objects)
    ) obj
    ON CONFLICT DO NOTHING;

    tmp := (SELECT array_remove(
        (
            SELECT p.objects
            FROM data.post p
            WHERE p.post_id = create_post_objects.post_id
        ),
        objects
    ));

    position := (SELECT array_position(tmp, destination));

    UPDATE data.post p
    SET objects =
        tmp[0:(SELECT coalesce(position - 1, cardinality(tmp)))] ||
        array_distinct(create_post_objects.objects) ||
        tmp[(SELECT coalesce(position, cardinality(tmp) + 1)):]
    WHERE p.post_id = create_post_objects.post_id;

    RETURN read_post_date_modified(post_id);
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION create_post_tag(
    a_post_id       uuid,
    a_tag_id        uuid
) RETURNS void AS $$
BEGIN
    INSERT INTO data.post_tag (post_id, tag_id)
    VALUES (a_post_id, a_tag_id);
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION create_related_post(
    a_post_id       uuid,
    a_related       uuid
) RETURNS void AS $$
BEGIN
    INSERT INTO data.related_post (post_id, related)
    VALUES (a_post_id, a_related)
    ON CONFLICT DO NOTHING;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION create_reply(
    a_parent_id     uuid,
    a_content       text
) RETURNS SETOF post_comment AS $$
DECLARE l_parent record;
BEGIN
    SELECT post_id, indent
    INTO l_parent
    FROM data.post_comment
    WHERE comment_id = a_parent_id;

    RETURN QUERY
    INSERT INTO data.post_comment(
        post_id,
        parent_id,
        indent,
        content
    ) VALUES (
        l_parent.post_id,
        a_parent_id,
        l_parent.indent + 1,
        a_content
    ) RETURNING
        comment_id,
        post_id,
        parent_id,
        indent,
        content,
        date_created;
END;
$$ LANGUAGE plpgsql;

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
    SELECT *
    FROM source
    WHERE (site).site_id = a_site_id AND resource = a_resource;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION create_tag(
    name            text
) RETURNS uuid AS $$
DECLARE
    id              uuid;
BEGIN
    WITH new_tag AS (
        INSERT INTO data.tag DEFAULT VALUES
        RETURNING *
    )
    SELECT INTO id
        tag_id
    FROM new_tag;

    INSERT INTO data.tag_name (
        tag_id,
        value,
        main
    ) VALUES (
        id,
        name,
        true
    );

    RETURN id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION create_tag_alias(
    a_tag_id        uuid,
    a_alias         text
) RETURNS SETOF tag_name AS $$
BEGIN
    INSERT INTO data.tag_name (tag_id, value)
    VALUES (a_tag_id, a_alias)
    ON CONFLICT DO NOTHING;

    RETURN QUERY
    SELECT t.name, t.aliases
    FROM tag_name_view t
    WHERE tag_id = a_tag_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION create_tag_source(
    a_tag_id        uuid,
    a_source_id     bigint
) RETURNS void AS $$
BEGIN
    INSERT INTO data.tag_source (
        tag_id,
        source_id
    ) VALUES (
        a_tag_id,
        a_source_id
    ) ON CONFLICT DO NOTHING;
END;
$$ LANGUAGE plpgsql;

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

CREATE FUNCTION delete_object_preview_error(
    a_object_id     uuid
) RETURNS void AS $$
BEGIN
    DELETE FROM data.object_preview_error
    WHERE object_id = a_object_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION delete_post(
    a_post_id       uuid
) RETURNS void AS $$
BEGIN
    DELETE FROM data.post
    WHERE post_id = a_post_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION delete_post_objects(post_id uuid, objects uuid[])
RETURNS timestamptz AS $$
BEGIN
    UPDATE data.post p
    SET objects = array_remove(p.objects, delete_post_objects.objects)
    WHERE p.post_id = delete_post_objects.post_id;

    DELETE FROM data.post_object po
    WHERE
        po.post_id = delete_post_objects.post_id AND
        object_id = ANY(objects);

    RETURN read_post_date_modified(post_id);
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION delete_post_tag(
    a_post_id       uuid,
    a_tag_id        uuid
) RETURNS void AS $$
BEGIN
    DELETE FROM data.post_tag
    WHERE post_id = a_post_id AND tag_id = a_tag_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION delete_related_post(
    a_post_id       uuid,
    a_related       uuid
) RETURNS void AS $$
BEGIN
    DELETE FROM data.related_post
    WHERE post_id = a_post_id AND related = a_related;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION delete_tag(
    a_tag_id        uuid
) RETURNS void AS $$
BEGIN
    DELETE FROM data.tag
    WHERE tag_id = a_tag_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION delete_tag_alias(
    a_tag_id        uuid,
    a_alias         text
) RETURNS SETOF tag_name AS $$
BEGIN
    DELETE FROM data.tag_name
    WHERE tag_id = a_tag_id
        AND main = false
        AND value = a_alias;

    RETURN QUERY
    SELECT t.name, t.aliases
    FROM tag_name_view t
    WHERE tag_id = a_tag_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION delete_tag_source(
    a_tag_id        uuid,
    a_source_id     bigint
) RETURNS void AS $$
BEGIN
    DELETE FROM data.tag_source
    WHERE tag_id = a_tag_id AND source_id = a_source_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION prune() RETURNS void AS $$
BEGIN
    PERFORM prune_post_objects();
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

CREATE FUNCTION read_comment(
    a_comment_id    uuid
) RETURNS SETOF post_comment AS $$
BEGIN
    RETURN QUERY
    SELECT *
    FROM post_comment
    WHERE comment_id = a_comment_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_comments(
    a_post_id       uuid
) RETURNS SETOF post_comment AS $$
BEGIN
    RETURN QUERY
    SELECT *
    FROM post_comment
    WHERE post_id = a_post_id
    ORDER BY
        indent,
        parent_id,
        date_created DESC;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_object_preview_errors()
RETURNS SETOF data.object_preview_error AS $$
BEGIN
    RETURN QUERY
    SELECT *
    FROM object_preview_error
    ORDER BY object_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_objects()
RETURNS SETOF object_preview AS $$
BEGIN
    RETURN QUERY
    SELECT
        object_id,
        preview_id
    FROM data.object
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

CREATE FUNCTION read_posts(
    a_posts         uuid[]
) RETURNS SETOF post_preview AS $$
BEGIN
    RETURN QUERY
    SELECT post_preview.*
    FROM (
        SELECT
            ordinality,
            unnest AS post_id
        FROM unnest(a_posts) WITH ORDINALITY
    ) posts
    JOIN post_preview USING (post_id)
    ORDER BY ordinality;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_post_search() RETURNS SETOF post_search AS $$
BEGIN
    RETURN QUERY
    SELECT *
    FROM post_search
    ORDER BY post_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_related_posts(
    a_post_id       uuid
) RETURNS SETOF post_preview AS $$
BEGIN
    RETURN QUERY
    SELECT preview.*
    FROM post_preview preview
    JOIN data.related_post post ON post.related = preview.post_id
    WHERE post.post_id = a_post_id
    ORDER BY title ASC, date_created DESC;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_post_tags(
    a_post_id       uuid
) RETURNS SETOF tag_preview AS $$
BEGIN
    RETURN QUERY
    SELECT tag_preview.*
    FROM tag_preview
    JOIN data.post_tag USING (tag_id)
    WHERE post_id = a_post_id
    ORDER BY name;
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

CREATE FUNCTION read_tag_previews(
    a_tags          uuid[]
) RETURNS SETOF tag_preview AS $$
BEGIN
    RETURN QUERY
    SELECT tag_preview.*
    FROM (
        SELECT
            ordinality,
            unnest AS tag_id
        FROM unnest(a_tags) WITH ORDINALITY
    ) tags
    JOIN tag_preview USING (tag_id)
    ORDER BY ordinality;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_tag_sources(
    a_tag_id        uuid
) RETURNS SETOF source AS $$
BEGIN
    RETURN QUERY
    SELECT source.*
    FROM data.tag_source
    JOIN source USING (source_id)
    WHERE tag_id = a_tag_id
    ORDER BY (site).host;
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

CREATE FUNCTION read_total_objects()
RETURNS SETOF bigint AS $$
BEGIN
    RETURN QUERY
    SELECT count(*) FROM data.object;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_object(
    a_object_id     uuid
) RETURNS SETOF object AS $$
BEGIN
    RETURN QUERY
    SELECT *
    FROM object
    WHERE object_id = a_object_id;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION read_object_posts(
    a_object_id     uuid
) RETURNS SETOF post_preview AS $$
BEGIN
    RETURN QUERY
    SELECT post_preview.*
    FROM post_preview
    JOIN data.post_object post_object USING (post_id)
    WHERE post_object.object_id = a_object_id
    ORDER BY date_created DESC;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION update_comment(
    a_comment_id    uuid,
    a_content       text
) RETURNS void AS $$
BEGIN
    UPDATE data.post_comment
    SET content = a_content
    WHERE comment_id = a_comment_id;
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

CREATE FUNCTION update_tag_description(a_tag_id uuid, a_description text)
RETURNS boolean AS $$
BEGIN
    UPDATE data.tag
    SET description = a_description
    WHERE tag_id = a_tag_id;

    RETURN FOUND;
END;
$$ LANGUAGE plpgsql;

/**
 * Updates a tag's main name.
 * If the new name already exists as an alias, the alias and the main name are
 * swapped. Otherwise, the main name is replaced with the new value.
 */
CREATE FUNCTION update_tag_name(
    -- The tag for which to update the main name.
    a_tag_id        uuid,
    -- The new main name.
    a_name          text
) RETURNS SETOF tag_name_update AS $$
DECLARE
    l_old_name      text;
BEGIN
    IF EXISTS (
        SELECT FROM data.tag_name
        WHERE tag_id = a_tag_id AND value = a_name AND main = false
    ) THEN
        UPDATE data.tag_name
        SET main = false
        WHERE tag_id = a_tag_id AND main = true;

        UPDATE data.tag_name
        SET main = true
        WHERE tag_id = a_tag_id AND value = a_name;
    ELSE
        SELECT INTO l_old_name value
        FROM data.tag_name
        WHERE tag_id = a_tag_id AND main = true;

        UPDATE data.tag_name
        SET value = a_name
        WHERE tag_id = a_tag_id AND main = true;
    END IF;

    RETURN QUERY
    SELECT
        ROW(t.name, t.aliases)::tag_name,
        l_old_name
    FROM tag_name_view t
    WHERE tag_id = a_tag_id;
END;
$$ LANGUAGE plpgsql;

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
