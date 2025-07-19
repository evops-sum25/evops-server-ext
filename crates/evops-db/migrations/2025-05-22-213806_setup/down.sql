DROP INDEX event_id_idx;

DROP INDEX tag_id_idx;

DROP INDEX events_title_trgm_idx;

DROP INDEX events_desc_trgm_idx;

DROP EXTENSION pg_trgm;

DROP TABLE events_to_tags;

DROP TABLE tag_aliases;

DROP TABLE event_images;

DROP TABLE events;

DROP TABLE tags;

DROP TABLE refresh_tokens;

DROP TABLE users;

DROP EXTENSION citext;
