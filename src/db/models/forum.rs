use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::db::models::user::User;
use crate::schema::forums;

#[derive(Queryable, Selectable, Insertable, Debug, Associations, SimpleObject)]
#[diesel(belongs_to(User, foreign_key=owner_id))]
#[diesel(table_name=forums)]
pub struct Forum {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub owner_id: i32,
}

/// Represents a new forum that will be inserted into the db
#[derive(Debug, Insertable)]
#[diesel(table_name = forums)]
pub struct NewForum<'a> {
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub owner_id: i32,
}
