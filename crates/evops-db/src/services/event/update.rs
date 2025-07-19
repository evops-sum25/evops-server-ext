use diesel::ExpressionMethods as _;
use diesel::QueryDsl as _;
use diesel_async::AsyncConnection as _;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl as _;
use diesel_async::scoped_futures::ScopedFutureExt as _;

use evops_models::{ApiError, ApiResult};

use crate::schema;

impl crate::Database {
    pub async fn update_event(
        &mut self,
        event_id: evops_models::EventId,
        user_id: evops_models::UserId,
        form: evops_models::UpdateEventForm,
    ) -> ApiResult<()> {
        let event_model = Self::find_event_model(&mut self.conn, event_id).await?;
        if user_id.into_inner() != event_model.author_id {
            return Err(ApiError::Forbidden({
                "You can't modify this event.".to_owned()
            }));
        }
        self.conn
            .transaction(|conn| {
                async {
                    if form.description.is_some() || form.title.is_some() {
                        unsafe { Self::update_basic_fields(conn, event_id, &form) }.await?;
                    }
                    if let Some(tag_ids) = form.tag_ids {
                        unsafe { Self::delete_tags_for_event(conn, event_id) }.await?;
                        unsafe { Self::create_tags_for_event(conn, event_id, tag_ids) }.await?;
                    }
                    ApiResult::Ok(())
                }
                .scope_boxed()
            })
            .await?;
        Ok(())
    }

    async unsafe fn update_basic_fields(
        conn: &mut AsyncPgConnection,
        id: evops_models::EventId,
        form: &evops_models::UpdateEventForm,
    ) -> ApiResult<()> {
        diesel::update(schema::events::table.find(id.into_inner()))
            .set({
                let description_eq = {
                    form.description
                        .as_ref()
                        .map(|it| schema::events::description.eq(it.as_ref()))
                };
                let title_eq = {
                    form.title
                        .as_ref()
                        .map(|it| schema::events::title.eq(it.as_ref()))
                };
                (description_eq, title_eq)
            })
            .execute(conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => ApiError::NotFound(e.to_string()),
                _ => e.into(),
            })?;
        Ok(())
    }

    async unsafe fn delete_tags_for_event(
        conn: &mut AsyncPgConnection,
        id: evops_models::EventId,
    ) -> ApiResult<()> {
        diesel::delete({
            schema::events_to_tags::table
                .filter(schema::events_to_tags::event_id.eq(id.into_inner()))
        })
        .execute(conn)
        .await
        .map_err(|e| match e {
            diesel::result::Error::NotFound => ApiError::NotFound(e.to_string()),
            _ => e.into(),
        })?;
        Ok(())
    }

    async unsafe fn create_tags_for_event(
        conn: &mut AsyncPgConnection,
        id: evops_models::EventId,
        new_tags: evops_models::EventTagIds,
    ) -> ApiResult<()> {
        let records: Vec<_> = {
            new_tags
                .into_inner()
                .into_iter()
                .map(|t| {
                    (
                        schema::events_to_tags::event_id.eq(id.into_inner()),
                        schema::events_to_tags::tag_id.eq(t.into_inner()),
                    )
                })
                .collect()
        };
        diesel::insert_into(schema::events_to_tags::table)
            .values(&records)
            .execute(conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => ApiError::NotFound(e.to_string()),
                _ => e.into(),
            })?;
        Ok(())
    }
}
