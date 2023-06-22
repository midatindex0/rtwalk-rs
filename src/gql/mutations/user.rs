use diesel::{insert_into, RunQueryDsl};
use log;

use crate::db::models::user::NewUser;
use crate::error::UserCreationError;
use crate::schema::users::dsl::*;

type Conn = r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

pub fn create_user(
    _username: String,
    _password: String,
    conn: &mut Conn,
) -> Result<usize, UserCreationError> {
    let new_user = NewUser {
        username: &_username,
        password: &_password,
        display_name: &_username,
        bio: None,
    };
    match insert_into(users).values(&new_user).execute(conn) {
        Ok(_id) => Ok(_id),
        Err(err) => match err {
            diesel::result::Error::DatabaseError(kind, info) => match kind {
                diesel::result::DatabaseErrorKind::UniqueViolation => Err(
                    UserCreationError::UsernameAlreadyExists("Username already exists."),
                ),
                _ => {
                    log::error!("{:?}", info);
                    Err(UserCreationError::InternalError(
                        "Some error occured, try again later.",
                    ))
                }
            },
            _ => {
                log::error!("{:?}", err);
                Err(UserCreationError::InternalError(
                    "Some error occured, try again later.",
                ))
            }
        },
    }
}