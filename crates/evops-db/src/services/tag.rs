use diesel::result::DatabaseErrorKind;
use diesel::{
    BelongingToDsl as _, ExpressionMethods as _, GroupedBy as _, Insertable, QueryDsl as _,
    SelectableHelper as _,
};
use diesel_async::scoped_futures::ScopedFutureExt as _;
use diesel_async::{AsyncConnection as _, RunQueryDsl as _};
use uuid::Uuid;

use evops_models::{ApiError, ApiResult};

use crate::models;
use crate::schema;

#[derive(Insertable)]
#[diesel(table_name = schema::tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct NewTag<'a> {
    id: Uuid,
    name: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = schema::tags_aliases)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct NewTagAlias<'a> {
    tag_id: Uuid,
    alias: &'a str,
}

impl crate::Database {
    pub async fn find_tag(&mut self, id: evops_models::TagId) -> ApiResult<evops_models::Tag> {
        self.conn
            .transaction(|conn| {
                async move {
                    let find_tag_result: Result<models::Tag, diesel::result::Error> = {
                        schema::tags::table
                            .find(id.into_inner())
                            .select(models::Tag::as_select())
                            .get_result(conn)
                            .await
                    };

                    let aliases = {
                        let find_result = {
                            schema::tags_aliases::table
                                .filter(schema::tags_aliases::tag_id.eq(id.into_inner()))
                                .select(models::TagAlias::as_select())
                                .load(conn)
                                .await
                        };
                        match find_result {
                            Ok(aliases) => aliases
                                .into_iter()
                                .map(|a| unsafe { evops_models::TagAlias::new_unchecked(a.alias) })
                                .collect(),
                            Err(e) => return Err(e.into()),
                        }
                    };

                    match find_tag_result {
                        Ok(tag) => Ok(evops_models::Tag {
                            id: evops_models::TagId::new(tag.id),
                            name: unsafe { evops_models::TagName::new_unchecked(tag.name) },
                            aliases,
                        }),
                        Err(e) => Err(match e {
                            diesel::result::Error::NotFound => {
                                ApiError::NotFound(format!("No tag with ID {id} found."))
                            }
                            _ => e.into(),
                        }),
                    }
                }
                .scope_boxed()
            })
            .await
    }

    pub async fn list_tags(
        &mut self,
        last_id: Option<evops_models::TagId>,
        limit: Option<evops_models::PgLimit>,
    ) -> ApiResult<Vec<evops_models::Tag>> {
        self.conn
            .transaction(|conn| {
                async move {
                    let tag_ids: Vec<Uuid> = {
                        let mut query = schema::tags::table.select(schema::tags::id).into_boxed(); // Runtime query

                        if let Some(id) = last_id {
                            query = query.filter(schema::tags::id.gt(id.into_inner()));
                        }

                        query = query.order(schema::tags::id.asc());

                        if let Some(lim) = limit {
                            query = query.limit(lim.into());
                        }
                        query.load(conn).await?
                    };
                    let tags: Vec<models::Tag> = {
                        schema::tags::table
                            .filter(schema::tags::id.eq_any(&tag_ids))
                            .select(models::Tag::as_select())
                            .load(conn)
                            .await?
                    };

                    let aliases: Vec<models::TagAlias> = {
                        models::TagAlias::belonging_to(&tags)
                            .select(models::TagAlias::as_select())
                            .load(conn)
                            .await?
                    };

                    let tags_with_aliases: Vec<(models::Tag, Vec<models::TagAlias>)> = {
                        aliases
                            .grouped_by(&tags)
                            .into_iter()
                            .zip(tags)
                            .map(|(a, t)| (t, a))
                            .collect()
                    };

                    Ok(tags_with_aliases
                        .into_iter()
                        .map(|(tag, aliases)| evops_models::Tag {
                            id: evops_models::TagId::new(tag.id),
                            name: unsafe { evops_models::TagName::new_unchecked(tag.name) },
                            aliases: {
                                aliases
                                    .into_iter()
                                    .map(|a| unsafe {
                                        evops_models::TagAlias::new_unchecked(a.alias)
                                    })
                                    .collect()
                            },
                        })
                        .collect())
                }
                .scope_boxed()
            })
            .await
    }

    pub async fn create_tag(
        &mut self,
        form: evops_models::NewTagForm,
    ) -> ApiResult<evops_models::Tag> {
        self.conn
            .transaction(|conn| {
                async move {
                    let tag_id = Uuid::now_v7();

                    let insert_tag_result = {
                        diesel::insert_into(schema::tags::table)
                            .values(NewTag {
                                id: tag_id,
                                name: form.name.as_ref(),
                            })
                            .execute(conn)
                            .await
                    };
                    if let Err(diesel::result::Error::DatabaseError(
                        DatabaseErrorKind::UniqueViolation,
                        info,
                    )) = insert_tag_result
                    {
                        return Err(ApiError::AlreadyExists(info.message().to_owned()));
                    }

                    let aliases = form.aliases.unwrap_or_default();
                    diesel::insert_into(schema::tags_aliases::table)
                        .values({
                            aliases
                                .iter()
                                .map(|a| NewTagAlias {
                                    tag_id,
                                    alias: a.as_ref(),
                                })
                                .collect::<Vec<_>>()
                        })
                        .execute(conn)
                        .await?;

                    Ok(evops_models::Tag {
                        id: evops_models::TagId::new(tag_id),
                        name: form.name,
                        aliases,
                    })
                }
                .scope_boxed()
            })
            .await
    }
}
