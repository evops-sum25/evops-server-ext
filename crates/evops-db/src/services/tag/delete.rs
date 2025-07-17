use diesel::{BelongingToDsl as _, GroupedBy as _, QueryResult};
use diesel::{ExpressionMethods as _, QueryDsl as _, SelectableHelper as _};
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl as _;

use evops_models::{ApiError, ApiResult};

use crate::models;
use crate::schema;

impl crate::Database {
    pub async fn delete_tag(&mut self, id: evops_models::TagId) -> ApiResult<()> {
        todo!();
    }
}
