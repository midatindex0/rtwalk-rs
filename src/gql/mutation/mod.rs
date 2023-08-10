pub mod comment;
mod forum;
mod post;
mod user;

use actix::Addr;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use async_graphql::{Context, ErrorExtensions, Object, Result, Upload};
use opendal::Operator;
use rand::Rng;

use crate::db::models::user::User;
use crate::error::UserCreationError;
use crate::{
    auth::SharedSession,
    constants,
    core::event::{
        CommentEvent, CommentEventTy, EventManager, ForumEvent, ForumEventTy, PostEvent,
        PostEventTy, UserEvent, UserEventTy,
    },
    db::models::{
        comment::{Comment, UpdateComment},
        forum::{Forum, SearchForum, UpdateForum},
        post::{InputPost, Post, SearchPost, UpdatePost},
        user::{SearchUser, UpdateUser},
    },
    error::UserAuthError,
    helpers::check_valid_uservane,
    search::SearchIndex,
};
use crate::{
    db::models::MaybeEmptyFile,
    helpers::{calculate_password_strength, check_reserved_username},
};

pub struct Mutation;

#[Object]
impl Mutation {
    async fn create_user<'c>(
        &self,
        ctx: &Context<'c>,
        username: String,
        password: String,
    ) -> Result<User> {
        if check_reserved_username(&username) {
            return Err(UserCreationError::ReservedUsername("This username is reserved").into());
        }

        if username.len() > 20 {
            return Err(UserCreationError::InvalidUsername(
                "Usernames can be atmost 20 characters",
            )
            .into());
        }

        if !check_valid_uservane(&username) {
            return Err(UserCreationError::InvalidUsername(
                "Username can only be lowercase, alphanumeric and seperated by _",
            )
            .into());
        }

        let pass_score = calculate_password_strength(&password, &username)?;
        if pass_score < 3 {
            return Err(UserCreationError::LowPasswordStrength(&format!(
                "Password is too weak [score {}/4]",
                pass_score
            ))
            .into());
        }

        if password.len() > 32 {
            return Err(
                UserCreationError::PasswordTooLong("Password can be atmost 32 characters").into(),
            );
        }

        let pool = ctx.data::<crate::Pool>()?;
        let hasher = ctx.data::<Argon2>()?;

        let salt = SaltString::generate(&mut OsRng);
        let hashed_pass = hasher
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| e.extend_with(|_, e| e.set("code", "500")))?
            .to_string();

        let created_user = user::create_user(username, hashed_pass, &pool).await?;

        let index = ctx.data::<SearchIndex>()?;
        let search_user: SearchUser = created_user.clone().into();
        index.user.add(search_user)?;

        let event_manager = ctx.data::<Addr<EventManager>>()?;

        event_manager.do_send(UserEvent {
            ty: UserEventTy::UserCreation,
            user: created_user.clone(),
        });

        Ok(created_user)
    }

    async fn update_user_basic<'c>(
        &self,
        ctx: &Context<'c>,
        changes: user::BasicUserUpdate,
    ) -> Result<User> {
        let session = ctx.data::<SharedSession>()?;
        let id = session.get::<i32>("id")?;

        if let Some(id) = id {
            let pool = ctx.data::<crate::Pool>()?;
            let index = ctx.data::<SearchIndex>()?;

            let mut changes: UpdateUser = changes.into();
            changes.id = id;
            let user = user::update_user(&changes, &pool).await?;

            let index_update: SearchUser = user.clone().into();
            index.user.update(index_update)?;

            let event_manager = ctx.data::<Addr<EventManager>>()?;

            event_manager.do_send(UserEvent {
                ty: UserEventTy::UserBasicUpdate,
                user: user.clone(),
            });

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
        let pool = ctx.data::<crate::Pool>()?;
        let hasher = ctx.data::<Argon2>()?.clone();
        let session = ctx.data::<SharedSession>()?;

        if session.get::<String>("username")?.is_some() {
            return Ok(true);
        }

        let x = user::verify_user(&username, &password, pool, &hasher).await?;

        match x {
            (true, user) => {
                session.insert("id", user.id)?;
                session.insert("admin", user.admin)?;
                session.insert("username", username)?;
                session.insert("pfp", user.pfp.id)?;
                Ok(true)
            }
            (false, _) => Err(UserAuthError::InvalidUsernameOrPassword(
                "Username or password is invalid",
            )
            .extend_with(|_, e| e.set("code", "401"))),
        }
    }

    async fn logout<'c>(&self, ctx: &Context<'c>) -> Result<bool> {
        let session = ctx.data::<SharedSession>()?;
        session.purge();
        Ok(true)
    }

    async fn upload<'c>(
        &self,
        ctx: &Context<'c>,
        uploads: Vec<Upload>,
    ) -> Result<Vec<MaybeEmptyFile>> {
        let session = ctx.data::<SharedSession>()?;
        let username = session.get::<String>("username")?;
        let mut files = Vec::with_capacity(uploads.len());
        if let Some(username) = username {
            let operator = ctx.data::<Operator>()?;
            for upload in uploads {
                let mut upload = upload.value(ctx)?;
                let file = MaybeEmptyFile::new(format!(
                    "media/{}/{}.{}",
                    username,
                    uuid::Uuid::new_v4(),
                    upload.filename.replace("/", "-")
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
            let pool = ctx.data::<crate::Pool>()?;
            let x = forum::create_forum(owner_id, name, display_name, description, &pool).await?;

            let index = ctx.data::<SearchIndex>()?;
            let search_forum: SearchForum = x.clone().into();
            index.forum.add(search_forum)?;

            let event_manager = ctx.data::<Addr<EventManager>>()?;

            event_manager.do_send(ForumEvent {
                ty: ForumEventTy::ForumCreation,
                forum: x.clone(),
            });

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
            let pool = ctx.data::<crate::Pool>()?;
            let index = ctx.data::<SearchIndex>()?;

            let changes: UpdateForum = changes.into();
            let forum = forum::update_forum(id, &changes, &pool).await?;

            let index_update: SearchForum = forum.clone().into();
            index.forum.update(index_update)?;

            let event_manager = ctx.data::<Addr<EventManager>>()?;

            event_manager.do_send(ForumEvent {
                ty: ForumEventTy::ForumBasicUpdate,
                forum: forum.clone(),
            });

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
            let pool = ctx.data::<crate::Pool>()?;
            let x = post::create_post(
                input_post.tags,
                input_post.title,
                slug,
                input_post.content,
                input_post.media,
                input_post.forum,
                poster_id,
                &pool,
            )
            .await?;

            let index = ctx.data::<SearchIndex>()?;
            let search_post: SearchPost = x.clone().into();
            index.post.add(search_post)?;

            let event_manager = ctx.data::<Addr<EventManager>>()?;

            event_manager.do_send(PostEvent {
                ty: PostEventTy::PostCreation,
                post: x.clone(),
            });

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
            let mut conn = ctx.data::<crate::Pool>()?;
            let index = ctx.data::<SearchIndex>()?;

            let changes: UpdatePost = changes.into();
            let post = post::update_post(id, &changes, &mut conn).await?;

            let index_update: SearchPost = post.clone().into();
            index.post.update(index_update)?;

            let event_manager = ctx.data::<Addr<EventManager>>()?;

            event_manager.do_send(PostEvent {
                ty: PostEventTy::PostBasicUpdate,
                post: post.clone(),
            });

            return Ok(post);
        }
        Err(
            async_graphql::Error::new(constants::UNAUTHEMTICATED_MESSAGE)
                .extend_with(|_, e| e.set("code", "401")),
        )
    }

    async fn update_comment_basic<'c>(
        &self,
        ctx: &Context<'c>,
        changes: comment::BasicCommentUpdate,
    ) -> Result<Comment> {
        let session = ctx.data::<SharedSession>()?;
        let id = session.get::<i32>("id")?;

        if let Some(id) = id {
            let pool = ctx.data::<crate::Pool>()?;

            let changes: UpdateComment = changes.into();
            let comment = comment::update_comment(id, &changes, &pool).await?;

            let event_manager = ctx.data::<Addr<EventManager>>()?;

            event_manager.do_send(CommentEvent {
                ty: CommentEventTy::CommentCreation,
                comment: comment.clone(),
            });

            return Ok(comment);
        }
        Err(
            async_graphql::Error::new(constants::UNAUTHEMTICATED_MESSAGE)
                .extend_with(|_, e| e.set("code", "401")),
        )
    }
}
