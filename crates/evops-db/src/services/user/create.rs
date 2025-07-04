use diesel::{Insertable, SelectableHelper as _};
use diesel_async::RunQueryDsl as _;
use uuid::Uuid;

use evops_models::ApiResult;

use crate::models;
use crate::schema;

#[derive(Insertable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct NewUser<'a> {
    id: Uuid,
    name: &'a str,
}

impl crate::Database {
    pub async fn create_user(
        &mut self,
        form: evops_models::NewUserForm,
    ) -> ApiResult<evops_models::UserId> {
        // This function doesn't start a transaction
        // because there's only one insert,* where it can fail.

        let user_id = evops_models::UserId::new(Uuid::now_v7());

        diesel::insert_into(schema::users::table)
            .values(self::NewUser {
                id: user_id.into_inner(),
                name: form.name.as_ref(),
            })
            .returning(models::User::as_returning())
            .execute(&mut self.conn)
            .await?; // *

        Ok(user_id)
    }
}
