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
        image_id: evops_models::EventImageId,
        user_id: evops_models::UserId,
    ) -> ApiResult<()> {
        let event_model = Self::find_event_model(&mut self.conn, event_id).await?;
        if user_id.into_inner() != event_model.author_id {
            return Err(ApiError::Forbidden({
                "You can't modify this event.".to_owned()
            }));
        }
        self.conn
            .transaction(|conn| {
                async { unsafe { Self::reserve_image_unatomic(conn, event_id, image_id).await } }
                    .scope_boxed()
            })
            .await
    }

    async unsafe fn reserve_image_unatomic(
        conn: &mut AsyncPgConnection,
        event_id: evops_models::EventId,
        image_id: evops_models::EventImageId,
    ) -> ApiResult<()> {
        let position = {
            let current_last_position: i16 = {
                schema::event_images::table
                    .select(schema::event_images::position)
                    .filter(schema::event_images::event_id.eq(event_id.into_inner()))
                    .order(schema::event_images::position.desc())
                    .first(conn)
                    .await
                    .optional()?
                    .unwrap_or_default()
            };
            current_last_position + 1
        };
        #[allow(clippy::missing_panics_doc)]
        let max_position = {
            evops_models::EventImageIds::ITEMS_MAX
                .try_conv::<i16>()
                .unwrap()
        };
        if position == max_position {
            return Err(ApiError::AlreadyExists(format!(
                "Event {event_id} already has {} images.",
                evops_models::EventImageIds::ITEMS_MAX,
            )));
        }

        diesel::insert_into(schema::event_images::table)
            .values(self::NewEventImage {
                id: image_id.into_inner(),
                event_id: event_id.into_inner(),
                position,
            })
            .execute(conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
                    ApiError::AlreadyExists(info.message().to_owned())
                }
                _ => e.into(),
            })?;

        let now = Utc::now();
        diesel::update(schema::events::table)
            .filter(schema::events::id.eq(event_id.into_inner()))
            .set(schema::events::modified_at.eq(now))
            .execute(conn)
            .await?;

        Ok(())
    }
}
