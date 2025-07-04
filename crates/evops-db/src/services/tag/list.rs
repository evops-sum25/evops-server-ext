use diesel::{
    BelongingToDsl as _, ExpressionMethods as _, GroupedBy as _, QueryDsl as _,
    SelectableHelper as _,
};
use diesel_async::RunQueryDsl as _;
use uuid::Uuid;

use evops_models::ApiResult;

use crate::models;
use crate::schema;

impl crate::Database {
    pub async fn list_tags(
        &mut self,
        last_id: Option<evops_models::TagId>,
        limit: Option<evops_models::PgLimit>,
    ) -> ApiResult<Vec<evops_models::Tag>> {
        let tag_ids: Vec<Uuid> = {
            let mut query = schema::tags::table.select(schema::tags::id).into_boxed();
            if let Some(last_id) = last_id {
                query = query.filter(schema::tags::id.gt(last_id.into_inner()));
            }
            query = query.order(schema::tags::id.asc());
            if let Some(limit) = limit {
                query = query.limit(limit.into());
            }
            query.load(&mut self.conn).await?
        };

        let tag_models: Vec<models::Tag> = {
            schema::tags::table
                .filter(schema::tags::id.eq_any(&tag_ids))
                .select(models::Tag::as_select())
                .load(&mut self.conn)
                .await?
        };

        let tag_alias_models: Vec<models::TagAlias> = {
            models::TagAlias::belonging_to(&tag_models)
                .select(models::TagAlias::as_select())
                .load(&mut self.conn)
                .await?
        };

        let tags_with_aliases_models: Vec<(models::Tag, Vec<models::TagAlias>)> = {
            tag_alias_models
                .grouped_by(&tag_models)
                .into_iter()
                .zip(tag_models)
                .map(|(alias_models, tag_model)| (tag_model, alias_models))
                .collect()
        };

        let tags = {
            tags_with_aliases_models
                .into_iter()
                .map(|(tag_model, alias_models)| evops_models::Tag {
                    id: evops_models::TagId::new(tag_model.id),
                    name: unsafe { evops_models::TagName::new_unchecked(tag_model.name) },
                    aliases: {
                        let tag_aliases_raw = {
                            alias_models
                                .into_iter()
                                .map(|a| unsafe { evops_models::TagAlias::new_unchecked(a.alias) })
                                .collect()
                        };
                        unsafe { evops_models::TagAliases::new_unchecked(tag_aliases_raw) }
                    },
                })
                .collect()
        };
        Ok(tags)
    }
}
