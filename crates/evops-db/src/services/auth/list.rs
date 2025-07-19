use diesel::result::QueryResult;
use diesel::{QueryDsl as _, SelectableHelper as _};
use diesel_async::{AsyncPgConnection, RunQueryDsl as _};

use evops_models::ApiResult;

use crate::models;
use crate::schema;

impl crate::Database {
    pub async fn list_users(&mut self) -> ApiResult<Vec<evops_models::User>> {
        let user_models = Self::list_all_user_models(&mut self.conn).await?;

        let users = {
            user_models
                .into_iter()
                .map(|user_model| evops_models::User {
                    id: evops_models::UserId::new(user_model.id),
                    login: unsafe { evops_models::UserLogin::new_unchecked(user_model.user_login) },
                    display_name: unsafe {
                        evops_models::UserDisplayName::new_unchecked(user_model.display_name)
                    },
                })
                .collect()
        };
        Ok(users)
    }

    async fn list_all_user_models(conn: &mut AsyncPgConnection) -> QueryResult<Vec<models::User>> {
        schema::users::table
            .select(models::User::as_select())
            .get_results(conn)
            .await
    }
}
