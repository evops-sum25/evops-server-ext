use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl as _;
use evops_models::ApiError;

use evops_models::ApiResult;

use crate::schema;

impl crate::Database {
    pub async fn get_password_hash(
        &mut self,
        login: &evops_models::UserLogin,
    ) -> ApiResult<evops_models::UserPasswordHash> {
        schema::users::table
            .filter(schema::users::user_login.eq(login.as_ref()))
            .select(schema::users::password_argon2)
            .get_result(&mut self.conn)
            .await
            .map(|it: String| evops_models::UserPasswordHash::new(it))
            .map_err(|_| ApiError::Forbidden("Wrong credentials.".to_string()))
    }
}
