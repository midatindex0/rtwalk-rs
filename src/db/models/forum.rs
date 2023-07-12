use async_graphql::{ComplexObject, Context, SimpleObject};
use chrono::NaiveDateTime;
use diesel::pg::Pg;
use diesel::prelude::*;
use tantivy::{doc, Document};

use super::File;
use crate::db::models::user::User;
use crate::db::pool::PostgresPool;
use crate::schema::{forums, users};
use crate::search::ToDoc;

#[derive(Clone, Queryable, Selectable, Insertable, Debug, Associations, SimpleObject)]
#[diesel(belongs_to(User, foreign_key=owner_id))]
#[diesel(table_name=forums)]
#[diesel(check_for_backend(Pg))]
#[graphql(complex)]
pub struct Forum {
    pub id: i32,
    pub name: String,
    pub display_name: String,
    pub icon: Option<File>,
    pub banner: Option<File>,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    #[graphql(skip)]
    pub owner_id: i32,
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = forums)]
pub struct UpdateForum {
    pub id: i32,
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub icon: Option<Option<File>>,
    pub banner: Option<Option<File>>,
    pub description: Option<Option<String>>,
    pub owner_id: Option<i32>,
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
