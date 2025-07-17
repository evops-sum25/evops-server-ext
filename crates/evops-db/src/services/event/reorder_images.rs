use evops_models::ApiResult;

impl crate::Database {
    #[allow(clippy::unused_async)]
    pub async fn reorder_images(
        &mut self,
        _event_id: evops_models::EventId,
        _image_order: evops_models::EventImageIds,
    ) -> ApiResult<()> {
        todo!();
    }
}
