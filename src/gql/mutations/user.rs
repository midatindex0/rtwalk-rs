use argon2::{Argon2, PasswordHash, PasswordVerifier};
use diesel::prelude::*;
use diesel::{insert_into, RunQueryDsl};
use log;

use crate::db::models::user::NewUser;
use crate::db::models::user::User;
use crate::error::{UserAuthError, UserCreationError};
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

pub fn verify_user<'a>(
    _username: &str,
    _password: &str,
    conn: &mut Conn,
    hasher: &Argon2,
) -> Result<(bool, i32), UserAuthError<'a>> {
    let _user = users
        .filter(username.eq(_username))
        .select(User::as_select())
        .get_result(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => UserAuthError::UserNotFound(""),
            _ => {
                log::error!("{:?}", e);
                UserAuthError::InternalError("Some error occured, try again later.")
            }
        })?;

    let parsed_hash = PasswordHash::new(&_user.password)
        .map_err(|_| UserAuthError::InternalError("Some error occured, try again later."))?;

    Ok((
        hasher
            .verify_password(_password.as_bytes(), &parsed_hash)
            .is_ok(),
        _user.v,
    ))
}
