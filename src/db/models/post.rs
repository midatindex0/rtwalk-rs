use async_graphql::{ComplexObject, Context, SimpleObject};
use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::db::models::{forum::Forum, user::User};

use crate::db::pool::PostgresPool;
use crate::schema::{forums, posts, users};

#[derive(Queryable, Selectable, Insertable, Debug, Associations, SimpleObject)]
#[diesel(belongs_to(User, foreign_key=poster_id))]
#[diesel(belongs_to(Forum, foreign_key=forum_id))]
#[diesel(table_name=posts)]
#[graphql(complex)]
pub struct Post {
    pub id: i32,
    pub tags: Option<Vec<Option<String>>>,
    pub stars: i32,
    pub title: String,
    pub slug: String,
    pub content: Option<String>,
    pub media: Option<Vec<Option<String>>>,
    pub created_at: NaiveDateTime,
    pub edited: bool,
    pub edited_at: Option<NaiveDateTime>,
    pub forum_id: i32,
    pub poster_id: i32,
}

#[ComplexObject]
impl Post {
    async fn forum<'ctx>(&self, ctx: &Context<'ctx>) -> async_graphql::Result<Forum> {
        let pool = ctx.data::<PostgresPool>().unwrap();
        let mut conn = pool.get()?;
        let forum = forums::table
            .find(self.forum_id)
            .get_result::<Forum>(&mut conn)?;
        Ok(forum)
    }

    async fn poster<'ctx>(&self, ctx: &Context<'ctx>) -> async_graphql::Result<User> {
        let pool = ctx.data::<PostgresPool>().unwrap();
        let mut conn = pool.get()?;
        let user = users::table
            .find(self.poster_id)
            .get_result::<User>(&mut conn)?;
        Ok(user)
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = posts)]
pub struct NewPost {
    pub tags: Option<Vec<String>>,
    pub title: String,
    pub slug: String,
    pub content: Option<String>,
    pub media: Option<Vec<String>>,
    pub forum_id: i32,
    pub poster_id: i32,
}
