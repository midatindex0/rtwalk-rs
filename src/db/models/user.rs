use crate::{db::models::MaybeEmptyFile, search::ToDoc};

use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tantivy::{doc, Document};

#[derive(Clone, Debug, SimpleObject, Deserialize, Serialize, FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    #[graphql(skip)]
    pub password: String,
    pub display_name: String,
    pub bio: Option<String>,
    #[sqlx(try_from = "Option<String>")]
    pub pfp: MaybeEmptyFile,
    #[sqlx(try_from = "Option<String>")]
    pub banner: MaybeEmptyFile,
    pub created_at: NaiveDateTime,
    #[graphql(skip)]
    // Not currently in use
    pub v: i32,
    pub admin: bool,
}

#[derive(Debug)]
pub struct UpdateUser {
    pub id: i32,
    pub username: Option<String>,
    pub password: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<Option<String>>,
    pub pfp: Option<MaybeEmptyFile>,
    pub banner: Option<MaybeEmptyFile>,
    pub admin: Option<bool>,
}

/// Represents a new user that will be inserted into the db
#[derive(Debug)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub display_name: &'a str,
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

    fn id(&self) -> i64 {
        self.id as i64
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
