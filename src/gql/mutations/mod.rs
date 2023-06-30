pub mod forum;
pub mod user;

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};

use async_graphql::{Context, ErrorExtensions, Object, Result};

use crate::helpers::{calculate_password_strength, check_reserved_username};
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
        _username: String,
        _password: String,
    ) -> Result<bool> {
        let _conn = ctx.data::<PostgresPool>()?.get()?;
        let _hasher = ctx.data::<Argon2>()?;

        Ok(true)
    }
}
