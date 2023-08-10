use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use sqlx::FromRow;
use tantivy::{doc, Document};

use super::MaybeEmptyFile;
use crate::search::ToDoc;

#[derive(Clone, Debug, SimpleObject, FromRow)]
pub struct Forum {
    pub id: i32,
    pub name: String,
    pub display_name: String,
    #[sqlx(try_from = "Option<String>")]
    pub icon: MaybeEmptyFile,
    #[sqlx(try_from = "Option<String>")]
    pub banner: MaybeEmptyFile,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    #[graphql(skip)]
    pub owner_id: i32,
}

#[derive(Debug)]
pub struct UpdateForum {
    pub id: i32,
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub icon: Option<MaybeEmptyFile>,
    pub banner: Option<MaybeEmptyFile>,
    pub description: Option<Option<String>>,
    pub owner_id: Option<i32>,
}

/// Represents a new forum that will be inserted into the db
#[derive(Debug)]
pub struct NewForum<'a> {
    pub name: &'a str,
    pub display_name: &'a str,
    pub description: Option<&'a str>,
    pub owner_id: i32,
}

#[derive(Debug)]
pub struct SearchForum {
    pub id: i32,
    pub name: String,
    pub display_name: String,
    pub decsription: Option<String>,
}

impl ToDoc for SearchForum {
    fn to_doc(self, schema: &tantivy::schema::Schema) -> anyhow::Result<Document> {
        let id = schema.get_field("id")?;
        let name = schema.get_field("name")?;
        let display_name = schema.get_field("display_name")?;
        let description = schema.get_field("description")?;
        Ok(doc!(
            id => self.id as i64,
            name => self.name,
            display_name => self.display_name,
            description => self.decsription.unwrap_or(String::from("")),
        ))
    }

    fn id(&self) -> i64 {
        self.id as i64
    }
}

impl From<Forum> for SearchForum {
    fn from(value: Forum) -> Self {
        Self {
            id: value.id,
            name: value.name,
            display_name: value.display_name,
            decsription: value.description,
        }
    }
}
