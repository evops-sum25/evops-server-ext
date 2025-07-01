use diesel::QueryDsl as _;
use diesel::result::{DatabaseErrorKind, OptionalExtension as _};
use diesel::{ExpressionMethods, Insertable};
use diesel_async::RunQueryDsl as _;
use tap::TryConv as _;
use uuid::Uuid;

use evops_models::{ApiError, ApiResult};

use crate::schema;

#[derive(Insertable)]
#[diesel(table_name = schema::event_images)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct NewEventImage {
    pub id: Uuid,
    pub event_id: Uuid,
    pub position: i16,
}

impl crate::Database {
    pub async fn reserve_image(
        &mut self,
        event_id: evops_models::EventId,
    ) -> ApiResult<evops_models::EventImageId> {
        // This function doesn't start a transaction
        // because there's only one insert,* where it can fail.

        let id = evops_models::EventImageId::new(Uuid::now_v7());

        let position: i16 = {
            schema::event_images::table
                .select(schema::event_images::position)
                .filter(schema::event_images::event_id.eq(event_id.into_inner()))
                .order(schema::event_images::position.desc())
                .first(&mut self.conn)
                .await
                .optional()?
                .unwrap_or_default()
        };

        #[allow(clippy::missing_panics_doc)]
        let max_position = evops_models::EVENT_MAX_IMAGES.try_conv::<i16>().unwrap() - 1;
        if position == max_position {
            return Err(ApiError::AlreadyExists(format!(
                "Event {event_id} already has {} images.",
                evops_models::EVENT_MAX_IMAGES,
            )));
        }

        diesel::insert_into(schema::event_images::table)
            .values(self::NewEventImage {
                id: id.into_inner(),
                event_id: event_id.into_inner(),
                position: 0,
            })
            .execute(&mut self.conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
                    ApiError::AlreadyExists(info.message().to_owned())
                }
                _ => e.into(),
            })?; // *

        Ok(id)
    }
}
