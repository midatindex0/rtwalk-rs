use async_graphql::{InputObject, SimpleObject};
use chrono::NaiveDateTime;
use sqlx::FromRow;
use tantivy::{doc, Document};

use super::FileList;
use crate::search::ToDoc;

#[derive(Clone, Debug, SimpleObject, FromRow)]
pub struct Post {
    pub id: i32,
    pub tags: Option<Vec<String>>,
    pub stars: i32,
    pub title: String,
    pub slug: String,
    pub content: Option<String>,
    #[sqlx(try_from = "Option<Vec<String>>")]
    pub media: FileList,
    pub created_at: NaiveDateTime,
    pub edited: bool,
    pub edited_at: Option<NaiveDateTime>,
    pub forum_id: i32,
    pub poster_id: i32,
}

#[derive(Debug)]
pub struct UpdatePost {
    pub id: i32,
    pub tags: Option<Option<Vec<String>>>,
    pub stars: Option<i32>,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<Option<String>>,
    pub media: Option<FileList>,
    pub edited: bool,
    pub edited_at: Option<NaiveDateTime>,
    pub poster_id: Option<i32>,
}

#[derive(Debug)]
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
    pub tags: Option<Vec<String>>,
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
            tags => self.tags.unwrap_or(vec![]).into_iter().collect::<Vec::<_>>().join(" "),
            title => self.title,
            content => self.content.unwrap_or(String::from("")),
        ))
    }

    fn id(&self) -> i64 {
        self.id as i64
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
