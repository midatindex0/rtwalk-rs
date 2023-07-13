use argon2::{Argon2, PasswordHash, PasswordVerifier};
use async_graphql::InputObject;
use diesel::prelude::*;
use diesel::{insert_into, RunQueryDsl};
use log;

use crate::db::models::user::User;
use crate::db::models::user::{NewUser, UpdateUser};
use crate::db::models::File;
use crate::error::{UserAuthError, UserCreationError};
use crate::schema::users::dsl::*;

#[derive(InputObject)]
pub struct BasicUserUpdate {
    pub _display_name: Option<String>,
    pub _bio: Option<String>,
    pub _pfp: Option<String>,
    pub _banner: Option<String>,
}

impl Into<UpdateUser> for BasicUserUpdate {
    fn into(self) -> UpdateUser {
        UpdateUser {
            id: 0,
            username: None,
            password: None,
            display_name: self._display_name,
            bio: self._bio.map(|x| if x.is_empty() { None } else { Some(x) }),
            pfp: self._pfp.map(|x| {
                if x.is_empty() {
                    None
                } else {
                    Some(File::new(x))
                }
            }),
            banner: self._banner.map(|x| {
                if x.is_empty() {
                    None
                } else {
                    Some(File::new(x))
                }
            }),
            admin: None,
        }
    }
}

pub fn create_user(
    _username: String,
    _password: String,
    conn: &mut crate::Conn,
) -> Result<User, UserCreationError> {
    let new_user = NewUser {
        username: &_username,
        password: &_password,
        display_name: &_username,
        bio: None,
    };
    match insert_into(users)
        .values(&new_user)
        .get_result::<User>(conn)
    {
        Ok(_user) => Ok(_user),
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

pub fn update_user(changes: &UpdateUser, conn: &mut crate::Conn) -> anyhow::Result<User> {
    let x = diesel::update(users)
        .set(changes)
        .get_result::<User>(conn)?;
    Ok(x)
}

pub fn verify_user<'a>(
    _username: &str,
    _password: &str,
    conn: &mut crate::Conn,
    hasher: &Argon2,
) -> Result<(bool, User), UserAuthError<'a>> {
    let _user: User = users
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
        _user,
    ))
}
