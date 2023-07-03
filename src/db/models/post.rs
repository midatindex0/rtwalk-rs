
use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::db::models::{forum::Forum, user::User};

use crate::schema::{posts};

#[derive(Queryable, Selectable, Insertable, Debug, Associations)]
#[diesel(belongs_to(User, foreign_key=poster))]
#[diesel(belongs_to(Forum, foreign_key=forum))]
#[diesel(table_name=posts)]
pub struct Post {
    pub id: i32,
    pub tags: Option<Vec<String>>,
    pub stars: i32,
    pub title: String,
    pub slug: String,
    pub content: Option<String>,
    pub media: Option<Vec<Option<String>>>,
    pub created_at: NaiveDateTime,
    pub edited: bool,
    pub edited_at: Option<NaiveDateTime>,
    pub forum: i32,
    pub poster: i32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = posts)]
pub struct NewPost<'a> {
    pub tags: Option<Vec<&'a str>>,
    pub title: &'a str,
    pub slug: &'a str,
    pub content: Option<&'a str>,
    pub media: Option<Vec<Option<&'a str>>>,
    pub forum: i32,
    pub poster: i32,
}
