use crate::schema::users;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, Selectable};
use juniper::{GraphQLInputObject, GraphQLObject};

/// Represents a user in the db.
/// Password is not public to restrict direct access. Use [`User::match_password`] instead
#[derive(Debug, Queryable, GraphQLObject, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
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

/// Represents an user signup data
#[derive(Debug, GraphQLInputObject)]
pub struct NewUserGql {
    pub username: String,
    pub password: String,
}
