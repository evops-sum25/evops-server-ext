CREATE EXTENSION citext;

CREATE TABLE users (
    id uuid PRIMARY KEY,
    user_login citext NOT NULL UNIQUE,
    password_argon2 text NOT NULL,
    display_name text NOT NULL
);

CREATE TABLE refresh_tokens (
    user_id uuid REFERENCES users (id) ON DELETE CASCADE,
    token_blake3 bytea NOT NULL UNIQUE,
    PRIMARY KEY (user_id)
);

CREATE TABLE tags (
    id uuid PRIMARY KEY,
    name text UNIQUE NOT NULL,
    owner_id uuid REFERENCES users (id)
);

CREATE INDEX tag_id_idx ON tags (id);

CREATE TABLE tag_aliases (
    tag_id uuid REFERENCES tags (id),
    alias text,
    PRIMARY KEY (tag_id, alias)
);

CREATE TABLE events (
    id uuid PRIMARY KEY,
    title text NOT NULL,
    description text NOT NULL,
    author_id uuid NOT NULL REFERENCES users (id),
    created_at timestamptz NOT NULL,
    modified_at timestamptz NOT NULL
);

CREATE INDEX event_id_idx ON events (id);

CREATE TABLE event_images (
    id uuid PRIMARY KEY,
    event_id uuid NOT NULL REFERENCES events (id) ON DELETE CASCADE,
    position smallint NOT NULL,
    UNIQUE (event_id, position)
);

CREATE TABLE events_to_tags (
    event_id uuid REFERENCES events (id),
    tag_id uuid REFERENCES tags (id),
    PRIMARY KEY (event_id, tag_id)
);

CREATE EXTENSION pg_trgm;

CREATE INDEX events_title_trgm_idx ON events USING GIN (title gin_trgm_ops);

CREATE INDEX events_desc_trgm_idx ON events USING GIN (description gin_trgm_ops);
