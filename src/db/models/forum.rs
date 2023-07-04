use async_graphql::{ComplexObject, Context, SimpleObject};
use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::db::models::user::User;
use crate::db::pool::PostgresPool;
use crate::schema::{forums, users};

#[derive(Queryable, Selectable, Insertable, Debug, Associations, SimpleObject)]
#[diesel(belongs_to(User, foreign_key=owner_id))]
#[diesel(table_name=forums)]
#[graphql(complex)]
pub struct Forum {
    pub id: i32,
    pub name: String,
    pub display_name: String,
    pub icon: Option<String>,
    pub banner: Option<String>,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    #[graphql(skip)]
    pub owner_id: i32,
}

#[ComplexObject]
impl Forum {
    async fn owner<'ctx>(&self, ctx: &Context<'ctx>) -> async_graphql::Result<User> {
        let pool = ctx.data::<PostgresPool>().unwrap();
        let mut conn = pool.get()?;
        let owner = users::table
            .filter(users::dsl::id.eq(self.owner_id))
            .get_result::<User>(&mut conn)?;
        Ok(owner)
    }
}

/// Represents a new forum that will be inserted into the db
#[derive(Debug, Insertable)]
#[diesel(table_name = forums)]
pub struct NewForum<'a> {
    pub name: &'a str,
    pub display_name: &'a str,
    pub description: Option<&'a str>,
    pub owner_id: i32,
}
