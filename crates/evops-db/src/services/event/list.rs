use std::collections::HashMap;

use diesel::{
    BoolExpressionMethods as _, ExpressionMethods as _, PgTextExpressionMethods as _,
    QueryDsl as _, QueryResult, SelectableHelper as _,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl as _};
use itertools::Itertools as _;
use uuid::Uuid;

use evops_models::ApiResult;

use crate::models;
use crate::schema;

impl crate::Database {
    pub async fn list_events(
        &mut self,
        last_id: Option<evops_models::EventId>,
        limit: Option<evops_models::PgLimit>,
        tags: Option<evops_models::EventTagIds>,
        search: Option<String>,
    ) -> ApiResult<Vec<evops_models::Event>> {
        let event_ids =
            Self::list_event_ids_raw(&mut self.conn, last_id, limit, tags, search).await?;
        Self::list_events_private(&mut self.conn, event_ids).await
    }

    pub(crate) async fn list_event_ids_raw(
        conn: &mut AsyncPgConnection,
        last_id: Option<evops_models::EventId>,
        limit: Option<evops_models::PgLimit>,
        tags: Option<evops_models::EventTagIds>,
        search: Option<String>,
    ) -> QueryResult<Vec<Uuid>> {
        let mut query = {
            schema::events::table
                .select(schema::events::id)
                .into_boxed()
        };
        let tags: Option<Vec<_>> =
            tags.map(|e| e.into_inner().iter().map(|e| e.into_inner()).collect());
        if tags.is_some() {
            let tagged_event_ids = schema::events_to_tags::table
                .filter(schema::events_to_tags::tag_id.eq_any(tags.unwrap()))
                .select(schema::events_to_tags::event_id);
            query = query.filter(schema::events::id.eq_any(tagged_event_ids));
        }
        if let Some(search_term) = search {
            query = query.filter(
                schema::events::title
                    .ilike(format!("%{search_term}%"))
                    .or(schema::events::description.ilike(format!("%{search_term}%"))),
            );
        }
        if let Some(last_id) = last_id {
            query = query.filter(schema::events::id.gt(last_id.into_inner()));
        }
        query = query.order(schema::events::id.asc());
        if let Some(limit) = limit {
            query = query.limit(limit.into());
        }
        let event_ids_raw = query.load(conn).await?;
        Ok(event_ids_raw)
    }

    // TODO: refactor this.
    #[allow(clippy::too_many_lines)]
    async fn list_events_private(
        conn: &mut AsyncPgConnection,
        event_ids_raw: Vec<Uuid>,
    ) -> ApiResult<Vec<evops_models::Event>> {
        if event_ids_raw.is_empty() {
            return Ok(Vec::default());
        }

        let events_with_authors: Vec<(models::Event, models::User)> = {
            schema::events::table
                .inner_join(schema::users::table)
                .filter(schema::events::id.eq_any(&event_ids_raw))
                .order(schema::events::id.asc())
                .select((models::Event::as_select(), models::User::as_select()))
                .load(conn)
                .await?
        };
        let images = {
            schema::event_images::table
                .filter(schema::event_images::event_id.eq_any(&event_ids_raw))
                .select(models::EventImage::as_select())
                .load(conn)
                .await?
                .into_iter()
                .into_group_map_by(|img| img.event_id)
        };

        let mut tags: HashMap<Uuid, HashMap<models::Tag, Option<Vec<models::TagAlias>>>> = {
            let event_tags: Vec<(Uuid, models::Tag)> = schema::events_to_tags::table
                .filter(schema::events_to_tags::event_id.eq_any(&event_ids_raw))
                .inner_join(schema::tags::table)
                .select((schema::events_to_tags::event_id, models::Tag::as_select()))
                .load::<(Uuid, models::Tag)>(conn)
                .await?;

            let tag_ids: Vec<Uuid> = event_tags.iter().map(|(_, tag)| tag.id).collect();
            let tag_aliases = {
                schema::tag_aliases::table
                    .filter(schema::tag_aliases::tag_id.eq_any(tag_ids))
                    .select(models::TagAlias::as_select())
                    .load(conn)
                    .await?
                    .into_iter()
                    .into_group_map_by(|alias| alias.tag_id)
            };
            event_tags
                .into_iter()
                .fold(HashMap::new(), |mut outer_map, (event_id, tag)| {
                    outer_map
                        .entry(event_id)
                        .or_default()
                        .insert(tag.clone(), tag_aliases.get(&tag.id).cloned());

                    outer_map
                })
        };

        let events: Vec<evops_models::Event> = {
            events_with_authors
                .into_iter()
                .map(|(event, author)| evops_models::Event {
                    id: evops_models::EventId::new(event.id),
                    author: evops_models::User {
                        id: evops_models::UserId::new(author.id),
                        name: unsafe { evops_models::UserName::new_unchecked(author.name) },
                    },
                    image_ids: {
                        let inner_value = images
                            .get(&event.id)
                            .unwrap_or(&Vec::new())
                            .iter()
                            .map(|img| evops_models::EventImageId::new(img.id))
                            .collect();
                        unsafe { evops_models::EventImageIds::new_unchecked(inner_value) }
                    },
                    title: unsafe { evops_models::EventTitle::new_unchecked(event.title) },
                    description: unsafe {
                        evops_models::EventDescription::new_unchecked(event.description)
                    },
                    tags: {
                        let inner_value = tags
                            .remove(&event.id)
                            .unwrap_or_default()
                            .into_iter()
                            .map(|t| evops_models::Tag {
                                id: evops_models::TagId::new(t.0.id),
                                name: unsafe {
                                    evops_models::TagName::new_unchecked(t.0.name.clone())
                                },
                                aliases: {
                                    let inner_value =
                                        t.1.map(|aliases| {
                                            aliases
                                                .into_iter()
                                                .map(|alias| unsafe {
                                                    evops_models::TagAlias::new_unchecked({
                                                        alias.alias
                                                    })
                                                })
                                                .collect()
                                        })
                                        .unwrap_or_default();
                                    unsafe { evops_models::TagAliases::new_unchecked(inner_value) }
                                },
                            })
                            .collect();
                        unsafe { evops_models::EventTags::new_unchecked(inner_value) }
                    },
                    with_attendance: event.with_attendance,
                    created_at: event.created_at,
                    modified_at: event.modified_at,
                })
                .collect()
        };
        // Maybe we will do it... Later
        // last_id = event_ids.last().map(|id| evops_models::EventId::new(*id))

        Ok(events)
    }
}
