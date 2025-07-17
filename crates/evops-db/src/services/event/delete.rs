use diesel::ExpressionMethods as _;
use diesel::QueryDsl as _;
use diesel_async::AsyncConnection as _;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl as _;
use diesel_async::scoped_futures::ScopedFutureExt as _;

use evops_models::{ApiError, ApiResult};

use crate::schema;

impl crate::Database {
    pub async fn delete_event(&mut self, id: evops_models::EventId) -> ApiResult<()> {
        self.conn
            .transaction(|conn| {
                async {
                    unsafe { Self::delete_events_to_tags(conn, id) }.await?;
                    unsafe { Self::delete_event_table(conn, id) }.await
                }
                .scope_boxed()
            })
            .await
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
