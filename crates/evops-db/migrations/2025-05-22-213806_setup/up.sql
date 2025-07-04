CREATE TABLE users (
    id uuid PRIMARY KEY,
    name text NOT NULL
);

CREATE TABLE tags (
    id uuid PRIMARY KEY,
    name text UNIQUE NOT NULL
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
    with_attendance bool NOT NULL,
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
