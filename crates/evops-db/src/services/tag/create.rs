use diesel::Insertable;
use diesel::result::DatabaseErrorKind;
use diesel_async::scoped_futures::ScopedFutureExt as _;
use diesel_async::{AsyncConnection as _, AsyncPgConnection, RunQueryDsl as _};
use itertools::Itertools as _;
use uuid::Uuid;

use evops_models::{ApiError, ApiResult};

use crate::schema;

#[derive(Insertable)]
#[diesel(table_name = schema::tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct NewTag<'a> {
    id: Uuid,
    name: &'a str,
    owner_id: Option<Uuid>,
}

#[derive(Insertable)]
#[diesel(table_name = schema::tag_aliases)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct NewTagAlias<'a> {
    tag_id: Uuid,
    alias: &'a str,
}

impl crate::Database {
    pub async fn create_tag(
        &mut self,
        form: evops_models::NewTagForm,
        owner_id: evops_models::UserId,
    ) -> ApiResult<evops_models::TagId> {
        self.conn
            .transaction(|conn| {
                async { unsafe { Self::create_tag_unatomic(conn, form, owner_id).await } }
                    .scope_boxed()
            })
            .await
    }

    async unsafe fn create_tag_unatomic(
        conn: &mut AsyncPgConnection,
        form: evops_models::NewTagForm,
        owner_id: evops_models::UserId,
    ) -> ApiResult<evops_models::TagId> {
        let id = evops_models::TagId::new(Uuid::now_v7());

        diesel::insert_into(schema::tags::table)
            .values(self::NewTag {
                id: id.into_inner(),
                name: form.name.as_ref(),
                owner_id: Some(owner_id.into_inner()),
            })
            .execute(conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
                    ApiError::AlreadyExists(info.message().to_owned())
                }
                _ => e.into(),
            })?;

        diesel::insert_into(schema::tag_aliases::table)
            .values({
                form.aliases
                    .as_ref()
                    .iter()
                    .map(|tag_alias_model| self::NewTagAlias {
                        tag_id: id.into_inner(),
                        alias: tag_alias_model.as_ref(),
                    })
                    .collect_vec()
            })
            .execute(conn)
            .await?;

        Ok(id)
    }
}
