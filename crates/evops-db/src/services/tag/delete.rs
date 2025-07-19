use diesel::{ExpressionMethods as _, QueryDsl as _};
use diesel_async::{
    AsyncConnection as _, AsyncPgConnection, RunQueryDsl as _, scoped_futures::ScopedFutureExt as _,
};

use evops_models::{ApiError, ApiResult};

use crate::schema;

impl crate::Database {
    pub async fn delete_tag(
        &mut self,
        id: evops_models::TagId,
        user_id: evops_models::UserId,
    ) -> ApiResult<()> {
        let tag_model = Self::find_tag_model(&mut self.conn, id).await?;
        if Some(user_id.into_inner()) != tag_model.owner_id {
            return Err(ApiError::Forbidden("You can't delete this tag.".to_owned()));
        }
        self.conn
            .transaction(|conn| {
                async { unsafe { Self::delete_tag_unatomic(conn, id) }.await }.scope_boxed()
            })
            .await
    }

    async unsafe fn delete_tag_unatomic(
        conn: &mut AsyncPgConnection,
        id: evops_models::TagId,
    ) -> ApiResult<()> {
        diesel::delete({
            schema::tag_aliases::table.filter(schema::tag_aliases::tag_id.eq(id.into_inner()))
        })
        .execute(conn)
        .await?;

        diesel::delete({
            schema::events_to_tags::table.filter(schema::events_to_tags::tag_id.eq(id.into_inner()))
        })
        .execute(conn)
        .await?;

        diesel::delete(schema::tags::table.find(id.into_inner()))
            .execute(conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => ApiError::NotFound(e.to_string()),
                _ => e.into(),
            })?;

        Ok(())
    }
}
