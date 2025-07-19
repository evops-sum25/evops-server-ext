use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl as _;
use evops_models::ApiError;

use evops_models::ApiResult;
use uuid::Uuid;

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
            .map_err(|_| ApiError::Forbidden("Wrong credentials.".to_string()))
    }

    pub async fn reissue_refresh_token(
        &mut self,
        token_hash: &evops_models::JsonWebTokenHash,
        user_id: evops_models::UserId,
    ) -> ApiResult<()> {
        Self::insert_refresh_token(&mut self.conn, token_hash, user_id)
            .await
            .map_err(Into::into)
    }
}
