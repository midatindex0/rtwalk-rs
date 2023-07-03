pub mod forum;
pub mod user;

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use async_graphql::{Context, ErrorExtensions, Object, Result, Upload};
use opendal::Operator;

use crate::{
    auth::{SharedSession},
    constants,
    error::UserAuthError,
};
use crate::{
    db::models::File,
    helpers::{calculate_password_strength, check_reserved_username},
};
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
        let session = ctx.data::<SharedSession>()?;
        let to_be_moved_username = username.clone();

        if session.get::<String>("username")?.is_some() {
            return Ok(true);
        }

        let x = actix_rt::task::spawn_blocking(move || {
            user::verify_user(&to_be_moved_username, &password, &mut conn, &hasher)
        })
        .await
        .map_err(|e| e.extend_with(|_, e| e.set("code", "500")))?;

        match x {
            Ok((true, v)) => {
                session.insert("username", username)?;
                session.insert("version", v)?;
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

    async fn logout<'c>(&self, ctx: &Context<'c>) -> Result<bool> {
        let session = ctx.data::<SharedSession>()?;
        session.purge();
        Ok(true)
    }

    async fn upload<'c>(&self, ctx: &Context<'c>, uploads: Vec<Upload>) -> Result<Vec<File>> {
        let session = ctx.data::<SharedSession>()?;
        let username = session.get::<String>("username")?;
        let mut files = Vec::with_capacity(uploads.len());
        if let Some(username) = username {
            let operator = ctx.data::<Operator>()?;
            for upload in uploads {
                let mut upload = upload.value(ctx)?;
                let file = File::new(format!(
                    "temp/{}/{}.{}",
                    username,
                    uuid::Uuid::new_v4(),
                    upload.filename
                ));
                file.save(&mut upload.content, operator).await?;
                files.push(file);
            }
            return Ok(files);
        }
        Err(
            async_graphql::Error::new(constants::UNAUTHEMTICATED_MESSAGE)
                .extend_with(|_, e| e.set("code", "401")),
        )
    }
}
