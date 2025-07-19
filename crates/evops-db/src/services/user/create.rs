use diesel::{Insertable, SelectableHelper as _};
use diesel_async::AsyncConnection as _;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl as _;
use diesel_async::scoped_futures::ScopedFutureExt as _;
use uuid::Uuid;

use evops_models::ApiResult;

use crate::models;
use crate::schema;

#[derive(Insertable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct NewUser<'a> {
    id: Uuid,
    user_login: &'a str,
    password_argon2: &'a str,
    display_name: &'a str,
}

impl crate::Database {
    pub async fn sign_up_user(
        &mut self,
        user_id: evops_models::UserId,
        login: &evops_models::UserLogin,
        password_hash: &evops_models::UserPasswordHash,
        display_name: &evops_models::UserDisplayName,
        refresh_token: &evops_models::JsonWebToken,
    ) -> ApiResult<()> {
        self.conn
            .transaction(|conn| {
                async {
                    unsafe {
                        Self::sign_up_user_unatomic(
                            conn,
                            user_id,
                            login,
                            password_hash,
                            display_name,
                            refresh_token,
                        )
                        .await
                    }
                }
                .scope_boxed()
            })
            .await
    }

    pub async unsafe fn sign_up_user_unatomic(
        conn: &mut AsyncPgConnection,
        user_id: evops_models::UserId,
        login: &evops_models::UserLogin,
        password_hash: &evops_models::UserPasswordHash,
        display_name: &evops_models::UserDisplayName,
        refresh_token: &evops_models::JsonWebToken,
    ) -> ApiResult<()> {
        diesel::insert_into(schema::users::table)
            .values(self::NewUser {
                id: user_id.into_inner(),
                user_login: login.as_ref(),
                password_argon2: password_hash.as_ref(),
                display_name: display_name.as_ref(),
            })
            .returning(models::User::as_returning())
            .execute(conn)
            .await?;

        Ok(())
    }
}
