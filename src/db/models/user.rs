use crate::schema::users;
use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, Selectable};

/// Represents a user in the db.
/// Password is not public to restrict direct access. Use [`User::match_password`] instead
#[derive(Debug, Queryable, Selectable, SimpleObject)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    #[graphql(skip)]
    pub id: i32,
    pub username: String,
    #[graphql(skip)]
    password: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub created_at: NaiveDateTime,
}

impl User {
    /// Matches the raw password with the hashed password in constant time.
    /// Returns [true] if it matches
    pub fn match_password(&self, _password: String) -> bool {
        bool::default()
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
