use diesel::{BelongingToDsl as _, QueryResult};
use diesel::{QueryDsl as _, SelectableHelper as _};
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl as _;

use evops_models::{ApiError, ApiResult};

use crate::models;
use crate::schema;

impl crate::Database {
    pub async fn delete_event(&mut self, _id: evops_models::EventId) -> ApiResult<()> {
        todo!();
    }
}
