mod forum;
mod post;
mod user;

use forum::{ForumCriteria, ForumFilter};
use user::{UserCriteria, UserFilter};

use async_graphql::{Context, InputObject, Object, Result};

use crate::{
    db::{
        models::{forum::Forum, user::User},
        pool::PostgresPool,
    },
    info::VersionInfo,
    search::SearchIndex,
};

#[derive(InputObject)]
pub struct Page {
    #[graphql(validator(minimum = 1))]
    num: u32,
    #[graphql(validator(minimum = 1, maximum = 50))]
    per: u32,
}

impl Default for Page {
    fn default() -> Self {
        Self { num: 1, per: 20 }
    }
}

impl Page {
    fn offset(&self) -> usize {
        return ((self.num - 1) * self.per) as usize;
    }
}

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

    async fn users<'c>(
        &self,
        ctx: &Context<'c>,
        filter: Option<UserFilter>,
        criteria: UserCriteria,
    ) -> Result<Vec<User>> {
        let mut conn = ctx.data::<PostgresPool>()?.get()?;
        let index = ctx.data::<SearchIndex>()?.clone();

        let users = actix_rt::task::spawn_blocking(move || {
            user::get_users(criteria, filter, &index, &mut conn)
        })
        .await??;

        Ok(users)
    }

    async fn forums<'c>(
        &self,
        ctx: &Context<'c>,
        filter: Option<ForumFilter>,
        criteria: ForumCriteria,
    ) -> Result<Vec<Forum>> {
        let mut conn = ctx.data::<PostgresPool>()?.get()?;
        let index = ctx.data::<SearchIndex>()?.clone();

        let forums = actix_rt::task::spawn_blocking(move || {
            forum::get_forums(criteria, filter, &index, &mut conn)
        })
        .await??;

        Ok(forums)
    }

    async fn posts<'c>(
        &self,
        ctx: &Context<'c>,
        filter: Option<post::PostFilter>,
        criteria: post::PostCriteria,
    ) -> Result<Vec<post::MultiPostReturn>> {
        let mut conn = ctx.data::<PostgresPool>()?.get()?;
        let index = ctx.data::<SearchIndex>()?.clone();

        let posts = actix_rt::task::spawn_blocking(move || {
            post::get_posts(filter, criteria, &index, &mut conn)
        })
        .await??;

        Ok(posts)
    }
}
