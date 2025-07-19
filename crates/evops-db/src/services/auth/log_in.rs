use diesel::{ExpressionMethods as _, QueryDsl as _};
use diesel_async::RunQueryDsl as _;
use uuid::Uuid;

use evops_models::{ApiError, ApiResult};

use crate::schema;

impl crate::Database {
    pub async fn get_password_hash(
        &mut self,
        login: &evops_models::UserLogin,
    ) -> ApiResult<(evops_models::UserId, evops_models::UserPasswordHash)> {
        schema::users::table
            .filter(schema::users::user_login.eq(login.as_ref()))
            .select((schema::users::id, schema::users::password_argon2))
            .get_result(&mut self.conn)
            .await
            .map(|(id, hash): (Uuid, String)| {
                (
                    evops_models::UserId::new(id),
                    evops_models::UserPasswordHash::new(hash),
                )
            })
            .map_err(|e| match e {
                diesel::result::Error::NotFound => {
                    ApiError::Forbidden("Wrong credentials.".to_string())
                }
                _ => e.into(),
            })
    }

    pub async fn insert_refresh_token(
        &mut self,
        token_hash: &evops_models::JsonWebTokenHash,
        user_id: evops_models::UserId,
    ) -> ApiResult<()> {
        Self::insert_refresh_token_inner(&mut self.conn, token_hash, user_id)
            .await
            .map_err(Into::into)
    }

    pub async fn check_refresh_token(
        &mut self,
        token_hash: &evops_models::JsonWebTokenHash,
    ) -> ApiResult<()> {
        let exists: bool = {
            diesel::select(diesel::dsl::exists({
                schema::refresh_tokens::table
                    .filter(schema::refresh_tokens::token_blake3.eq(token_hash.as_ref()))
            }))
            .get_result(&mut self.conn)
            .await?
        };

        if exists {
            Ok(())
        } else {
            Err(ApiError::Auth("Invalid refresh JWT token.".to_owned()))
        }
    }
}
