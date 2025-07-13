use chrono::Utc;
use diesel::QueryDsl as _;
use diesel::result::{DatabaseErrorKind, OptionalExtension as _};
use diesel::{ExpressionMethods, Insertable};
use diesel_async::scoped_futures::ScopedFutureExt as _;
use diesel_async::{AsyncConnection as _, AsyncPgConnection, RunQueryDsl as _};
use tap::TryConv as _;
use uuid::Uuid;

use evops_models::{ApiError, ApiResult};

use crate::schema;

impl crate::Database {
    pub async fn reorder_images(
        &mut self,
        event_id: evops_models::EventId,
        image_order: evops_models::EventImageIds,
    ) -> ApiResult<()> {
        todo!();
    }
}
