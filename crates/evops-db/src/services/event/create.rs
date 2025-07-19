use chrono::{DateTime, Utc};
use diesel::Insertable;
use diesel_async::scoped_futures::ScopedFutureExt as _;
use diesel_async::{AsyncConnection as _, AsyncPgConnection, RunQueryDsl as _};
use itertools::Itertools as _;
use uuid::Uuid;

use evops_models::{ApiError, ApiResult};

use crate::schema;

#[derive(Insertable)]
#[diesel(table_name = schema::events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct NewEvent<'a> {
    id: Uuid,
    title: &'a str,
    description: &'a str,
    author_id: Uuid,
    created_at: &'a DateTime<Utc>,
    modified_at: &'a DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = schema::events_to_tags)]
#[diesel(primary_key(event_id, tag_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct NewEventToTag {
    event_id: Uuid,
    tag_id: Uuid,
}

impl crate::Database {
    pub async fn create_event(
        &mut self,
        form: evops_models::NewEventForm,
        author_id: evops_models::UserId,
    ) -> ApiResult<evops_models::EventId> {
        self.conn
            .transaction(|conn| {
                async { unsafe { Self::create_event_unatomic(conn, form, author_id).await } }
                    .scope_boxed()
            })
            .await
    }

    async unsafe fn create_event_unatomic(
        conn: &mut AsyncPgConnection,
        form: evops_models::NewEventForm,
        author_id: evops_models::UserId,
    ) -> ApiResult<evops_models::EventId> {
        let event_id = evops_models::EventId::new(Uuid::now_v7());

        let now = Utc::now();
        diesel::insert_into(schema::events::table)
            .values(self::NewEvent {
                id: event_id.into_inner(),
                title: form.title.as_ref(),
                description: form.description.as_ref(),
                author_id: author_id.into_inner(),
                created_at: &now,
                modified_at: &now,
            })
            .execute(conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => ApiError::NotFound(e.to_string()),
                _ => e.into(),
            })?;

        diesel::insert_into(schema::events_to_tags::table)
            .values({
                form.tag_ids
                    .into_inner()
                    .iter()
                    .map(|tag_id| self::NewEventToTag {
                        event_id: event_id.into_inner(),
                        tag_id: tag_id.into_inner(),
                    })
                    .collect_vec()
            })
            .execute(conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => ApiError::NotFound(e.to_string()),
                _ => e.into(),
            })?;

        Ok(event_id)
    }
}
