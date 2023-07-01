use crate::{
    db::{models::File, pool::PostgresPool},
    schema::{forums, users},
};

use async_graphql::{ComplexObject, Context, SimpleObject};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::{Identifiable, Insertable, Queryable, Selectable};

use super::forum::Forum;

/// Represents a user in the db.
/// Password is not public to restrict direct access. Use [`User::match_password`] instead
#[derive(Debug, Queryable, Selectable, Identifiable, SimpleObject)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[graphql(complex)]
pub struct User {
    #[graphql(skip)]
    pub id: i32,
    pub username: String,
    #[graphql(skip)]
    pub password: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub pfp: Option<File>,
    pub banner: Option<File>,
    pub created_at: NaiveDateTime,
    pub v: i32,
}

#[ComplexObject]
impl User {
    async fn owned_forums<'ctx>(&self, ctx: &Context<'ctx>) -> async_graphql::Result<Vec<Forum>> {
        let pool = ctx.data::<PostgresPool>().unwrap();
        let mut conn = pool.get()?;
        let forums = forums::table
            .filter(forums::dsl::owner_id.eq(self.id))
            .load::<Forum>(&mut conn)?;
        Ok(forums)
    }
}

/// Represents a new user that will be inserted into the db
#[derive(Debug, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub display_name: &'a str,
    pub bio: Option<&'a str>,
}
