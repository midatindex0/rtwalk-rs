mod post;
mod user;

use async_graphql::{Context, Object, Result};

use crate::{
    db::{
        models::{post::PostWithoutUser, user::User},
        pool::PostgresPool,
    },
    info::VersionInfo,
};

pub struct Query;

#[Object]
impl Query {
    async fn version<'c>(&self, ctx: &Context<'c>) -> Result<&'c VersionInfo> {
        ctx.data::<VersionInfo>()
    }

    /// TODO: Error Handling (in responses)
    async fn user<'c>(&self, ctx: &Context<'c>, username: String) -> Result<User> {
        let mut conn = ctx.data::<PostgresPool>()?.get()?;

        let user = actix_rt::task::spawn_blocking(move || {
            user::get_user_by_username(&username, &mut conn)
        })
        .await??;
        Ok(user)
    }

    async fn posts<'c>(
        &self,
        ctx: &Context<'c>,
        forum: i32,
        criteria: post::PostCriteria,
    ) -> Result<Vec<post::PostWithUser>> {
        let mut conn = ctx.data::<PostgresPool>()?.get()?;

        let posts =
            actix_rt::task::spawn_blocking(move || post::get_posts(forum, &criteria, &mut conn))
                .await??;

        Ok(posts
            .into_iter()
            .map(|(post, poster)| post::PostWithUser {
                post: PostWithoutUser::from(post),
                poster,
            })
            .collect::<Vec<_>>())
    }
}
