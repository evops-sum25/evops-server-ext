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

#[derive(Insertable)]
#[diesel(table_name = schema::refresh_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct NewRefreshToken<'a> {
    id: Uuid,
    user_id: Uuid,
    token_blake3: &'a [u8],
}

impl crate::Database {
    pub async fn sign_up(
        &mut self,
        user_id: evops_models::UserId,
        login: &evops_models::UserLogin,
        password_hash: &evops_models::UserPasswordHash,
        display_name: &evops_models::UserDisplayName,
        refresh_token_hash: &evops_models::JsonWebTokenHash,
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
                            refresh_token_hash,
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
        refresh_token_hash: &evops_models::JsonWebTokenHash,
    ) -> ApiResult<()> {
        diesel::insert_into(schema::refresh_tokens::table)
            .values(self::NewRefreshToken {
                id: Uuid::now_v7(),
                user_id: user_id.into_inner(),
                token_blake3: refresh_token_hash.as_ref(),
            })
            .execute(conn)
            .await?;

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
