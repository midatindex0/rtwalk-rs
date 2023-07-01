pub mod forum;
pub mod user;

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};

use async_graphql::{Context, ErrorExtensions, Object, Result};
use rusty_paseto::prelude::{Local, PasetoSymmetricKey, V4};

use crate::helpers::{calculate_password_strength, check_reserved_username};
use crate::{auth::AuthUser, error::UserAuthError};
use crate::{db::pool::PostgresPool, error::UserCreationError};

pub struct Mutation;

#[Object]
impl Mutation {
    async fn create_user<'c>(
        &self,
        ctx: &Context<'c>,
        username: String,
        password: String,
    ) -> Result<bool> {
        let mut conn = ctx.data::<PostgresPool>()?.get()?;
        let hasher = ctx.data::<Argon2>()?;

        if check_reserved_username(&username) {
            return Err(UserCreationError::ReservedUsername("This username is reserved.").into());
        }

        if calculate_password_strength(&password) != 4 {
            return Err(UserCreationError::LowPasswordStrength(
                "Password must have 1 uppercase, lowercase, numeric and special char.",
            )
            .into());
        }

        if password.len() < 5 {
            return Err(UserCreationError::PasswordTooShort(
                "Password must be atleast 5 characters.",
            )
            .into());
        }

        if password.len() > 16 {
            return Err(UserCreationError::PasswordTooShort(
                "Password can be atmost 16 characters.",
            )
            .into());
        }

        let salt = SaltString::generate(&mut OsRng);
        let hashed_pass = hasher
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| e.extend_with(|_, e| e.set("code", "500")))?
            .to_string();

        actix_rt::task::spawn_blocking(move || {
            user::create_user(username, hashed_pass, &mut conn)
                .map(|_| true)
                .map_err(|e| {
                    e.extend_with(|err, e| match err {
                        UserCreationError::UsernameAlreadyExists(_) => e.set("code", "409"),
                        UserCreationError::InternalError(_) => e.set("code", "500"),
                        _ => unreachable!(),
                    })
                })
        })
        .await
        .map_err(|e| e.extend_with(|_, e| e.set("code", "500")))?
    }

    async fn login<'c>(
        &self,
        ctx: &Context<'c>,
        username: String,
        password: String,
    ) -> Result<bool> {
        let mut conn = ctx.data::<PostgresPool>()?.get()?;
        let hasher = ctx.data::<Argon2>()?.clone();
        let paseto_key = ctx.data::<PasetoSymmetricKey<V4, Local>>()?;
        let to_be_moved_username = username.clone();

        let x = actix_rt::task::spawn_blocking(move || {
            user::verify_user(&to_be_moved_username, &password, &mut conn, &hasher)
        })
        .await
        .map_err(|e| e.extend_with(|_, e| e.set("code", "500")))?;

        match x {
            Ok((true, v)) => {
                let user = AuthUser {
                    username: Some(username),
                    version: v,
                }
                .to_token(paseto_key)?;
                ctx.insert_http_header("auth", user);
                Ok(true)
            }
            Ok((false, _)) => Err(UserAuthError::InvalidUsernameOrPassword(
                "Username or password is invalid",
            )
            .extend_with(|_, e| e.set("code", "401"))),
            Err(e) => match e {
                UserAuthError::UserNotFound(_) => Err(UserAuthError::InvalidUsernameOrPassword(
                    "Username or password is invalid",
                )
                .extend_with(|_, e| e.set("code", "401"))),
                UserAuthError::InternalError(_) => Err(UserAuthError::InternalError(
                    "Some internal error occured. Try again",
                )
                .extend_with(|_, e| e.set("code", "500"))),
                UserAuthError::InvalidUsernameOrPassword(_) => unreachable!(),
            },
        }
    }
}
