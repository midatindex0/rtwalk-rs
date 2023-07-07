use crate::{
    db::{models::File, pool::PostgresPool},
    schema::{forums, users},
    search::ToDoc,
};

use async_graphql::{ComplexObject, Context, SimpleObject};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::{Insertable, Queryable, Selectable};
use tantivy::{doc, Document};

use super::forum::Forum;

/// Represents a user in the db.
/// Password is not public to restrict direct access. Use [`User::match_password`] instead
#[derive(Debug, Queryable, Selectable, SimpleObject)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[graphql(complex)]
pub struct User {
    pub id: i32,
    pub username: String,
    #[graphql(skip)]
    pub password: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub pfp: Option<File>,
    pub banner: Option<File>,
    pub created_at: NaiveDateTime,
    #[graphql(skip)]
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

#[derive(Debug)]
pub struct SearchUser {
    pub id: i32,
    pub username: String,
    pub display_name: String,
    pub bio: Option<String>,
}

impl ToDoc for SearchUser {
    fn to_doc(self, schema: &tantivy::schema::Schema) -> anyhow::Result<Document> {
        let id = schema.get_field("id")?;
        let username = schema.get_field("username")?;
        let display_name = schema.get_field("display_name")?;
        let bio = schema.get_field("bio")?;
        Ok(doc!(
            id => self.id as i64,
            username => self.username,
            display_name => self.display_name,
            bio => self.bio.unwrap_or(String::from("")),
        ))
    }
}

impl From<User> for SearchUser {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username,
            display_name: value.display_name,
            bio: value.bio,
        }
    }
}
