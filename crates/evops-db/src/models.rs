use chrono::{DateTime, Utc};
use diesel::{Associations, Identifiable, Queryable, Selectable};
use uuid::Uuid;

use crate::schema;

#[derive(Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::languages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Language {
    pub id: Uuid,
    pub name: String,
}

#[derive(Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub name: String,
}

#[derive(Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Eq, Hash, PartialEq, Clone)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
}

#[derive(Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = schema::tag_aliases)]
#[diesel(belongs_to(Tag))]
#[diesel(primary_key(tag_id, alias))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Eq, Hash, PartialEq, Clone)]
pub struct TagAlias {
    pub tag_id: Uuid,
    pub alias: String,
}

#[derive(Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Event {
    pub id: Uuid,
    pub author_id: Uuid,
    pub primary_language: Option<Uuid>,
    pub with_attendance: bool,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

#[derive(Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = schema::event_images)]
#[diesel(belongs_to(Event))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EventImage {
    pub id: Uuid,
    pub event_id: Uuid,
    pub position: i16,
}

#[derive(Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = schema::events_to_tags)]
#[diesel(belongs_to(Event))]
#[diesel(belongs_to(Tag))]
#[diesel(primary_key(event_id, tag_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EventToTag {
    pub event_id: Uuid,
    pub tag_id: Uuid,
}

#[derive(Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = schema::event_translations)]
#[diesel(belongs_to(Event))]
#[diesel(belongs_to(Language))]
#[diesel(primary_key(event_id, language_id))]
pub struct EventTranslation {
    pub event_id: Uuid,
    pub language_id: Uuid,
    pub title: String,
    pub description: String,
}
