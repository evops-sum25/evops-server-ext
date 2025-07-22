use std::collections::HashMap;

use diesel::{
    BoolExpressionMethods as _, ExpressionMethods as _, JoinOnDsl as _,
    PgSortExpressionMethods as _, PgTextExpressionMethods as _, QueryDsl as _, QueryResult,
    SelectableHelper as _,
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
        tags: Vec<evops_models::TagId>,
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
        tags: Vec<evops_models::TagId>,
        search: Option<String>,
    ) -> QueryResult<Vec<Uuid>> {
        let tags: Vec<_> = tags
            .into_iter()
            .map(evops_models::TagId::into_inner)
            .collect();
        let mut query = schema::events::table
            .left_join(
                schema::events_to_tags::table.on(schema::events::id
                    .eq(schema::events_to_tags::event_id)
                    .and(schema::events_to_tags::tag_id.eq_any(&tags))),
            )
            .group_by(schema::events::id)
            .select(schema::events::id)
            .order_by(
                diesel::dsl::count(schema::events_to_tags::tag_id)
                    .desc()
                    .nulls_last(),
            )
            .then_order_by(schema::events::id.desc())
            .into_boxed();
        tracing::info!(
            "Query: {}",
            diesel::debug_query::<diesel::pg::Pg, _>(&query).to_string()
        );
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
        if let Some(limit) = limit {
            query = query.limit(limit.into());
        }
        let event_ids_raw = query.load(conn).await?;
        Ok(event_ids_raw)
    }

    async fn get_events_with_authors(
        conn: &mut AsyncPgConnection,
        event_ids_raw: &Vec<Uuid>,
    ) -> ApiResult<Vec<(models::Event, models::User)>> {
        let result_raw: Vec<(models::Event, models::User)> = schema::events::table
            .inner_join(schema::users::table)
            .filter(schema::events::id.eq_any(event_ids_raw))
            .select((models::Event::as_select(), models::User::as_select()))
            .load(conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => evops_models::ApiError::NotFound(e.to_string()),
                _ => e.into(),
            })?;
        Ok(result_raw)
    }

    async fn get_tags(
        conn: &mut AsyncPgConnection,
        event_ids_raw: &Vec<Uuid>,
    ) -> ApiResult<HashMap<Uuid, HashMap<models::Tag, Option<Vec<models::TagAlias>>>>> {
        let result: HashMap<Uuid, HashMap<models::Tag, Option<Vec<models::TagAlias>>>> = {
            let event_tags: Vec<(Uuid, models::Tag)> = schema::events_to_tags::table
                .filter(schema::events_to_tags::event_id.eq_any(event_ids_raw))
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
        Ok(result)
    }

    async fn get_images(
        conn: &mut AsyncPgConnection,
        event_ids_raw: &Vec<Uuid>,
    ) -> ApiResult<HashMap<uuid::Uuid, Vec<models::EventImage>>> {
        let result = schema::event_images::table
            .filter(schema::event_images::event_id.eq_any(event_ids_raw))
            .select(models::EventImage::as_select())
            .load(conn)
            .await?
            .into_iter()
            .into_group_map_by(|img| img.event_id);
        Ok(result)
    }

    async fn list_events_private(
        conn: &mut AsyncPgConnection,
        event_ids_raw: Vec<Uuid>,
    ) -> ApiResult<Vec<evops_models::Event>> {
        if event_ids_raw.is_empty() {
            return Ok(Vec::default());
        }

        let events_with_authors = Self::get_events_with_authors(conn, &event_ids_raw).await?;
        let images = Self::get_images(conn, &event_ids_raw).await?;
        let mut tags = Self::get_tags(conn, &event_ids_raw).await?;

        let mut events: HashMap<Uuid, evops_models::Event> = {
            events_with_authors
                .into_iter()
                .map(|(event, author)| {
                    (
                        event.id,
                        evops_models::Event {
                            id: evops_models::EventId::new(event.id),
                            author: evops_models::User {
                                id: evops_models::UserId::new(author.id),
                                display_name: unsafe {
                                    evops_models::UserDisplayName::new_unchecked(
                                        author.display_name,
                                    )
                                },
                                login: unsafe {
                                    evops_models::UserLogin::new_unchecked(author.user_login)
                                },
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
                                            unsafe {
                                                evops_models::TagAliases::new_unchecked(inner_value)
                                            }
                                        },
                                    })
                                    .collect();
                                unsafe { evops_models::EventTags::new_unchecked(inner_value) }
                            },
                            created_at: event.created_at,
                            modified_at: event.modified_at,
                        },
                    )
                })
                .collect()
        };
        let mut result = Vec::with_capacity(event_ids_raw.len());
        for event_id in event_ids_raw {
            if let Some(pair) = events.remove(&event_id) {
                result.push(pair);
            }
        }
        Ok(result)
    }
}
