use diesel::ExpressionMethods as _;
use diesel::QueryDsl as _;
use diesel_async::AsyncConnection as _;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl as _;
use diesel_async::scoped_futures::ScopedFutureExt as _;

use evops_models::{ApiError, ApiResult};

use crate::schema;

impl crate::Database {
    pub async fn delete_event(
        &mut self,
        event_id: evops_models::EventId,
        user_id: evops_models::UserId,
    ) -> ApiResult<evops_models::EventImageIds> {
        let event = self.find_event(event_id).await?;
        if user_id != event.author.id {
            return Err(ApiError::Forbidden({
                "You can't delete this event.".to_owned()
            }));
        }

        self.conn
            .transaction(|conn| {
                async {
                    unsafe { Self::delete_events_to_tags(conn, event_id) }.await?;
                    unsafe { Self::delete_event_table(conn, event_id) }.await?;
                    ApiResult::Ok(())
                }
                .scope_boxed()
            })
            .await?;

        Ok(event.image_ids)
    }

    async unsafe fn delete_events_to_tags(
        conn: &mut AsyncPgConnection,
        id: evops_models::EventId,
    ) -> ApiResult<()> {
        diesel::delete(
            schema::events_to_tags::table
                .filter(schema::events_to_tags::event_id.eq(id.into_inner())),
        )
        .execute(conn)
        .await
        .map_err(|e| match e {
            diesel::result::Error::NotFound => ApiError::NotFound(e.to_string()),
            _ => e.into(),
        })?;
        Ok(())
    }

    async unsafe fn delete_event_table(
        conn: &mut AsyncPgConnection,
        id: evops_models::EventId,
    ) -> ApiResult<()> {
        diesel::delete(schema::events::table.find(id.into_inner()))
            .execute(conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => ApiError::NotFound(e.to_string()),
                _ => e.into(),
            })?;
        Ok(())
    }
}
