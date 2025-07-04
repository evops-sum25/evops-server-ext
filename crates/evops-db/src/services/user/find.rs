use diesel::{QueryDsl as _, SelectableHelper as _};
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl as _;

use evops_models::{ApiError, ApiResult};

use crate::models;
use crate::schema;

impl crate::Database {
    pub async fn find_user(&mut self, id: evops_models::UserId) -> ApiResult<evops_models::User> {
        let user_model = Self::find_user_model(&mut self.conn, id).await?;

        let user = evops_models::User {
            id,
            name: unsafe { evops_models::UserName::new_unchecked(user_model.name) },
        };
        Ok(user)
    }

    async fn find_user_model(
        conn: &mut AsyncPgConnection,
        id: evops_models::UserId,
    ) -> ApiResult<models::User> {
        schema::users::table
            .find(id.into_inner())
            .select(models::User::as_select())
            .get_result(conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => {
                    ApiError::NotFound(format!("No user with ID {id} found."))
                }
                _ => e.into(),
            })
    }
}
