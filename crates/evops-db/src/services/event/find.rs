use diesel::{BelongingToDsl as _, QueryResult};
use diesel::{QueryDsl as _, SelectableHelper as _};
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl as _;

use evops_models::{ApiError, ApiResult};

use crate::models;
use crate::schema;

impl crate::Database {
    pub async fn find_event(
        &mut self,
        id: evops_models::EventId,
    ) -> ApiResult<evops_models::Event> {
        let event_model = Self::find_event_model(&mut self.conn, id).await?;
        let event = evops_models::Event {
            id,
            author: {
                self.find_user(evops_models::UserId::new(event_model.author_id))
                    .await?
            },
            image_ids: Self::image_ids_of_event_model_sorted(&mut self.conn, &event_model).await?,
            tags: Self::tags_of_event_model(&mut self.conn, &event_model).await?,
            title: unsafe { evops_models::EventTitle::new_unchecked(event_model.title) },
            description: unsafe {
                evops_models::EventDescription::new_unchecked(event_model.description)
            },
            with_attendance: event_model.with_attendance,
            created_at: event_model.created_at,
            modified_at: event_model.modified_at,
        };
        Ok(event)
    }

    pub(crate) async fn find_event_model(
        conn: &mut AsyncPgConnection,
        id: evops_models::EventId,
    ) -> ApiResult<models::Event> {
        schema::events::table
            .find(id.into_inner())
            .select(models::Event::as_select())
            .get_result(conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => {
                    ApiError::NotFound(format!("No event with ID {id} found."))
                }
                _ => e.into(),
            })
    }

    pub(crate) async fn image_ids_of_event_model_sorted(
        conn: &mut AsyncPgConnection,
        event_model: &models::Event,
    ) -> QueryResult<evops_models::EventImageIds> {
        let image_ids_raw = {
            models::EventImage::belonging_to(event_model)
                .select(schema::event_images::id)
                .order(schema::event_images::position)
                .load(conn)
                .await?
                .into_iter()
                .map(evops_models::EventImageId::new)
                .collect()
        };
        let image_ids = unsafe { evops_models::EventImageIds::new_unchecked(image_ids_raw) };
        Ok(image_ids)
    }

    pub(crate) async fn tags_of_event_model(
        conn: &mut AsyncPgConnection,
        event_model: &models::Event,
    ) -> QueryResult<evops_models::EventTags> {
        let tag_models = Self::tag_models_of_event_model(conn, event_model).await?;
        let tags_raw = Self::tags_from_models(conn, tag_models).await?;
        let event_tags = unsafe { evops_models::EventTags::new_unchecked(tags_raw) };
        Ok(event_tags)
    }

    pub(crate) async fn tag_models_of_event_model(
        conn: &mut AsyncPgConnection,
        event_model: &models::Event,
    ) -> QueryResult<Vec<models::Tag>> {
        models::EventToTag::belonging_to(event_model)
            .inner_join(schema::tags::table)
            .select(models::Tag::as_select())
            .load(conn)
            .await
    }
}
