// @generated automatically by Diesel CLI.

diesel::table! {
    event_images (id) {
        id -> Uuid,
        event_id -> Uuid,
        position -> Int2,
    }
}

diesel::table! {
    event_translations (event_id, language_id) {
        event_id -> Uuid,
        language_id -> Uuid,
        title -> Text,
        description -> Text,
    }
}

diesel::table! {
    events (id) {
        id -> Uuid,
        author_id -> Nullable<Uuid>,
        primary_language_id -> Nullable<Uuid>,
        start -> Nullable<Timestamptz>,
        location_id -> Nullable<Uuid>,
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
    languages (id) {
        id -> Uuid,
        name -> Text,
    }
}

diesel::table! {
    location_translations (location_id, language_id) {
        location_id -> Uuid,
        language_id -> Uuid,
        name -> Text,
    }
}

diesel::table! {
    locations (id) {
        id -> Uuid,
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
diesel::joinable!(event_translations -> events (event_id));
diesel::joinable!(event_translations -> languages (language_id));
diesel::joinable!(events -> languages (primary_language_id));
diesel::joinable!(events -> locations (location_id));
diesel::joinable!(events -> users (author_id));
diesel::joinable!(events_to_tags -> events (event_id));
diesel::joinable!(events_to_tags -> tags (tag_id));
diesel::joinable!(location_translations -> languages (language_id));
diesel::joinable!(location_translations -> locations (location_id));
diesel::joinable!(tag_aliases -> tags (tag_id));

diesel::allow_tables_to_appear_in_same_query!(
    event_images,
    event_translations,
    events,
    events_to_tags,
    languages,
    location_translations,
    locations,
    tag_aliases,
    tags,
    users,
);
