pub mod comment;
mod forum;
mod post;
mod user;

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use async_graphql::{Context, ErrorExtensions, Object, Result, Upload};
use opendal::Operator;
use rand::Rng;

use crate::{
    auth::SharedSession,
    constants,
    db::models::{
        forum::{Forum, SearchForum, UpdateForum},
        post::{InputPost, Post, SearchPost, UpdatePost},
        user::{SearchUser, UpdateUser},
    },
    error::UserAuthError,
    helpers::check_valid_uservane,
    search::SearchIndex,
};
use crate::{db::models::user::User, spawn_blocking};
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

        if !check_valid_uservane(&username) {
            return Err(UserCreationError::InvalidUsername(
                "Username can only be alphanuremic and seperated by _.",
            )
            .into());
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

        let created_user = actix_rt::task::spawn_blocking(move || {
            user::create_user(username, hashed_pass, &mut conn).map_err(|e| {
                e.extend_with(|err, e| match err {
                    UserCreationError::UsernameAlreadyExists(_) => e.set("code", "409"),
                    UserCreationError::InternalError(_) => e.set("code", "500"),
                    _ => unreachable!(),
                })
            })
        })
        .await
        .map_err(|e| e.extend_with(|_, e| e.set("code", "500")))??;

        let index = ctx.data::<SearchIndex>()?;
        let search_user: SearchUser = created_user.into();
        index.user.add(search_user)?;

        Ok(true)
    }

    async fn update_user_basic<'c>(
        &self,
        ctx: &Context<'c>,
        changes: user::BasicUserUpdate,
    ) -> Result<User> {
        let session = ctx.data::<SharedSession>()?;
        let id = session.get::<i32>("id")?;

        if let Some(id) = id {
            let mut conn = ctx.data::<PostgresPool>()?.get()?;
            let index = ctx.data::<SearchIndex>()?;

            let mut changes: UpdateUser = changes.into();
            changes.id = id;
            let user = spawn_blocking!(user::update_user(&changes, &mut conn))??;

            let index_update: SearchUser = user.clone().into();
            index.user.update(index_update)?;
            return Ok(user);
        }
        Err(
            async_graphql::Error::new(constants::UNAUTHEMTICATED_MESSAGE)
                .extend_with(|_, e| e.set("code", "401")),
        )
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
            Ok((true, id)) => {
                session.insert("id", id)?;
                session.insert("username", username)?;
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
                    "media/{}/{}.{}",
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

    async fn create_forum<'c>(
        &self,
        ctx: &Context<'c>,
        display_name: String,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<Forum> {
        let session = ctx.data::<SharedSession>()?;

        if session.get::<String>("username")?.is_some() {
            let name = name.unwrap_or_else(|| slug::slugify(display_name.clone()));
            let owner_id = session.get::<i32>("id")?.unwrap();
            let mut conn = ctx.data::<PostgresPool>()?.get()?;
            let x = spawn_blocking!(forum::create_forum(
                owner_id,
                name,
                display_name,
                description,
                &mut conn
            ))??;

            let index = ctx.data::<SearchIndex>()?;
            let search_forum: SearchForum = x.clone().into();
            index.forum.add(search_forum)?;

            return Ok(x);
        }
        Err(
            async_graphql::Error::new(constants::UNAUTHEMTICATED_MESSAGE)
                .extend_with(|_, e| e.set("code", "401")),
        )
    }

    async fn update_forum_basic<'c>(
        &self,
        ctx: &Context<'c>,
        changes: forum::BasicForumUpdate,
    ) -> Result<Forum> {
        let session = ctx.data::<SharedSession>()?;
        let id = session.get::<i32>("id")?;

        if let Some(id) = id {
            let mut conn = ctx.data::<PostgresPool>()?.get()?;
            let index = ctx.data::<SearchIndex>()?;

            let changes: UpdateForum = changes.into();
            let forum = spawn_blocking!(forum::update_forum(id, &changes, &mut conn))??;

            let index_update: SearchForum = forum.clone().into();
            index.forum.update(index_update)?;

            return Ok(forum);
        }
        Err(
            async_graphql::Error::new(constants::UNAUTHEMTICATED_MESSAGE)
                .extend_with(|_, e| e.set("code", "401")),
        )
    }

    async fn create_post<'c>(&self, ctx: &Context<'c>, input_post: InputPost) -> Result<Post> {
        let session = ctx.data::<SharedSession>()?;
        let random_suffix = rand::thread_rng().gen_range(1..=9999);
        let slug = format!(
            "{}-{}",
            slug::slugify(input_post.title.clone()),
            random_suffix
        );
        if session.get::<String>("username")?.is_some() {
            let poster_id = session.get::<i32>("id")?.unwrap();
            let mut conn = ctx.data::<PostgresPool>()?.get()?;
            let x = spawn_blocking!(post::create_post(
                input_post.tags,
                input_post.title,
                slug,
                input_post.content,
                input_post.media,
                input_post.forum,
                poster_id,
                &mut conn,
            ))??;

            let index = ctx.data::<SearchIndex>()?;
            let search_post: SearchPost = x.clone().into();
            index.post.add(search_post)?;

            return Ok(x);
        }
        Err(
            async_graphql::Error::new(constants::UNAUTHEMTICATED_MESSAGE)
                .extend_with(|_, e| e.set("code", "401")),
        )
    }

    async fn update_post_basic<'c>(
        &self,
        ctx: &Context<'c>,
        changes: post::BasicPostUpdate,
    ) -> Result<Post> {
        let session = ctx.data::<SharedSession>()?;
        let id = session.get::<i32>("id")?;

        if let Some(id) = id {
            let mut conn = ctx.data::<PostgresPool>()?.get()?;
            let index = ctx.data::<SearchIndex>()?;

            let changes: UpdatePost = changes.into();
            let post = spawn_blocking!(post::update_post(id, &changes, &mut conn))??;

            let index_update: SearchPost = post.clone().into();
            index.post.update(index_update)?;

            return Ok(post);
        }
        Err(
            async_graphql::Error::new(constants::UNAUTHEMTICATED_MESSAGE)
                .extend_with(|_, e| e.set("code", "401")),
        )
    }
}
