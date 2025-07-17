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
        id: evops_models::EventId,
        form: evops_models::UpdateEventForm,
    ) -> ApiResult<()> {
        self.conn
            .transaction(|conn| {
                async {
                    if form.description.is_some()
                        || form.title.is_some()
                        || form.track_attendance.is_some()
                    {
                        unsafe { Self::update_basic_fields(conn, id, &form) }.await?;
                    }
                    if let Some(tag_ids) = form.tag_ids {
                        unsafe { Self::delete_tags_for_event(conn, id) }.await?;
                        unsafe { Self::create_tags_for_event(conn, id, tag_ids) }.await?;
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
                let track_attendance_eq = {
                    form.track_attendance
                        .map(|it| schema::events::with_attendance.eq(it))
                };
                (description_eq, title_eq, track_attendance_eq)
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
