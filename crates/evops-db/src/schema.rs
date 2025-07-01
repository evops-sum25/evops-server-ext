// @generated automatically by Diesel CLI.

diesel::table! {
    event_images (id) {
        id -> Uuid,
        event_id -> Uuid,
        position -> Int2,
    }
}

diesel::table! {
    events (id) {
        id -> Uuid,
        title -> Text,
        description -> Text,
        author_id -> Uuid,
        with_attendance -> Bool,
        created_at -> Timestamptz,
        modified_at -> Timestamptz,
    }
}

diesel::table! {
    events_to_tags (event_id, tag_id) {
        event_id -> Uuid,
        tag_id -> Uuid,
    }
}

diesel::table! {
    tag_aliases (tag_id, alias) {
        tag_id -> Uuid,
        alias -> Text,
    }
}

diesel::table! {
    tags (id) {
        id -> Uuid,
        name -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        name -> Text,
    }
}

diesel::joinable!(event_images -> events (event_id));
diesel::joinable!(events -> users (author_id));
diesel::joinable!(events_to_tags -> events (event_id));
diesel::joinable!(events_to_tags -> tags (tag_id));
diesel::joinable!(tag_aliases -> tags (tag_id));

diesel::allow_tables_to_appear_in_same_query!(
    event_images,
    events,
    events_to_tags,
    tag_aliases,
    tags,
    users,
);
