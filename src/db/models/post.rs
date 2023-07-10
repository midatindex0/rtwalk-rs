use async_graphql::{ComplexObject, Context, InputObject, SimpleObject};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use tantivy::{doc, Document};

use crate::db::models::{forum::Forum, user::User};

use crate::db::pool::PostgresPool;
use crate::schema::{forums, posts, users};
use crate::search::ToDoc;

#[derive(Clone, Queryable, Selectable, Debug, SimpleObject)]
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

#[derive(SimpleObject)]
pub struct RawPost {
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

impl From<Post> for RawPost {
    fn from(value: Post) -> Self {
        Self {
            id: value.id,
            tags: value.tags,
            stars: value.stars,
            title: value.title,
            slug: value.slug,
            content: value.content,
            media: value.media,
            created_at: value.created_at,
            edited: value.edited,
            edited_at: value.edited_at,
            forum_id: value.forum_id,
            poster_id: value.poster_id,
        }
    }
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

#[derive(InputObject)]
pub struct InputPost {
    #[graphql(default)]
    pub tags: Option<Vec<String>>,
    pub title: String,
    #[graphql(default)]
    pub content: Option<String>,
    #[graphql(default)]
    pub media: Option<Vec<String>>,
    pub forum: i32,
}

#[derive(Debug)]
pub struct SearchPost {
    pub id: i32,
    pub tags: Option<Vec<Option<String>>>,
    pub title: String,
    pub content: Option<String>,
}

impl ToDoc for SearchPost {
    fn to_doc(self, schema: &tantivy::schema::Schema) -> anyhow::Result<Document> {
        let id = schema.get_field("id")?;
        let tags = schema.get_field("tags")?;
        let title = schema.get_field("title")?;
        let content = schema.get_field("content")?;
        Ok(doc!(
            id => self.id as i64,
            tags => self.tags.unwrap_or(vec![]).into_iter().filter_map(|x| x).collect::<Vec::<_>>().join(" "),
            title => self.title,
            content => self.content.unwrap_or(String::from("")),
        ))
    }
}

impl Into<SearchPost> for Post {
    fn into(self) -> SearchPost {
        SearchPost {
            id: self.id,
            tags: self.tags,
            title: self.title,
            content: self.content,
        }
    }
}
