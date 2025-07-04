use diesel::{BelongingToDsl as _, GroupedBy as _, QueryResult};
use diesel::{ExpressionMethods as _, QueryDsl as _, SelectableHelper as _};
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl as _;

use evops_models::{ApiError, ApiResult};

use crate::models;
use crate::schema;

impl crate::Database {
    pub async fn find_tag(&mut self, id: evops_models::TagId) -> ApiResult<evops_models::Tag> {
        let tag_model = Self::find_tag_model(&mut self.conn, id).await?;

        let tag = evops_models::Tag {
            id,
            name: unsafe { evops_models::TagName::new_unchecked(tag_model.name) },
            aliases: {
                let tag_aliases_raw = {
                    schema::tag_aliases::table
                        .filter(schema::tag_aliases::tag_id.eq(id.into_inner()))
                        .select(models::TagAlias::as_select())
                        .load(&mut self.conn)
                        .await?
                        .into_iter()
                        .map(|tag_alias_model| unsafe {
                            evops_models::TagAlias::new_unchecked(tag_alias_model.alias)
                        })
                        .collect()
                };
                unsafe { evops_models::TagAliases::new_unchecked(tag_aliases_raw) }
            },
        };
        Ok(tag)
    }

    pub(crate) async fn find_tag_model(
        conn: &mut AsyncPgConnection,
        id: evops_models::TagId,
    ) -> ApiResult<models::Tag> {
        schema::tags::table
            .find(id.into_inner())
            .select(models::Tag::as_select())
            .get_result(conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => {
                    ApiError::NotFound(format!("No tag with ID {id} found."))
                }
                _ => e.into(),
            })
    }

    pub(crate) async fn tags_from_models(
        conn: &mut AsyncPgConnection,
        tag_models: Vec<models::Tag>,
    ) -> QueryResult<Vec<evops_models::Tag>> {
        let alias_models = Self::alias_models_of_tag_models(conn, &tag_models).await?;
        let alias_models_of_tag_models: Vec<(models::Tag, Vec<models::TagAlias>)> = {
            alias_models
                .grouped_by(&tag_models)
                .into_iter()
                .zip(tag_models)
                .map(|(alias_models, tag_model)| (tag_model, alias_models))
                .collect()
        };

        let tags = alias_models_of_tag_models
            .into_iter()
            .map(|(tag, aliases)| evops_models::Tag {
                id: evops_models::TagId::new(tag.id),
                name: unsafe { evops_models::TagName::new_unchecked(tag.name) },
                aliases: {
                    let aliases_raw = {
                        aliases
                            .into_iter()
                            .map(|a| unsafe { evops_models::TagAlias::new_unchecked(a.alias) })
                            .collect()
                    };
                    unsafe { evops_models::TagAliases::new_unchecked(aliases_raw) }
                },
            })
            .collect();

        Ok(tags)
    }

    pub(crate) async fn alias_models_of_tag_models(
        conn: &mut AsyncPgConnection,
        tag_models: &[models::Tag],
    ) -> QueryResult<Vec<models::TagAlias>> {
        models::TagAlias::belonging_to(tag_models)
            .select(models::TagAlias::as_select())
            .load(conn)
            .await
    }
}
