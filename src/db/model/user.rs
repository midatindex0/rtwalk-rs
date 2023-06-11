use crate::schema::users;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use juniper::{GraphQLInputObject, GraphQLObject};

/// represents a user in the db
/// password is not public to restrict direct access. Use `User::match_password` instead
#[derive(Queryable, GraphQLObject)]
pub struct User {
    pub id: i32,
    pub username: String,
    password: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub display_name: &'a str,
    pub bio: Option<&'a str>,
}

#[derive(GraphQLInputObject)]
pub struct NewUserGql {
    pub username: String,
    pub password: String,
}
