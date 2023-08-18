// mod comment;
mod forum;
// pub mod post;
pub mod user;

use forum::{ForumCriteria, ForumFilter};
use user::{UserCriteria, UserFilter};

use async_graphql::{Context, Enum, InputObject, Object, Result};
use futures::TryStreamExt;
use sqlx::Row;

use crate::{
    db::models::{comment::CommentHierarchy, user::User},
    info::VersionInfo,
    search::SearchIndex,
};

use self::user::{UserOrder, UserResponse};

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum PageOrder {
    ASC,
    DESC,
}

impl PageOrder {
    fn as_str(&self) -> &str {
        match self {
            Self::ASC => "ASC",
            Self::DESC => "DESC",
        }
    }
}

#[derive(InputObject)]
pub struct Page {
    order: Option<PageOrder>,
    #[graphql(validator(minimum = 1))]
    next_from: i32,
    #[graphql(validator(minimum = 1, maximum = 50))]
    limit: Option<i64>,
}

impl From<Option<Page>> for RawPage {
    fn from(value: Option<Page>) -> Self {
        match value {
            None => Self {
                order: PageOrder::DESC,
                next_from: i32::MAX,
                per: 10,
            },
            Some(filter) => Self {
                order: filter.order.unwrap_or(PageOrder::DESC),
                next_from: filter.next_from,
                per: filter.limit.unwrap_or(10).min(50),
            },
        }
    }
}

pub struct RawPage {
    order: PageOrder,
    next_from: i32,
    per: i64,
}

pub struct Query;

#[Object(cache_control(max_age = 60))]
impl Query {
    async fn version<'c>(&self, ctx: &Context<'c>) -> Result<&'c VersionInfo> {
        ctx.data::<VersionInfo>()
    }

    // TODO: Error Handling (in responses)
    async fn user<'c>(&self, ctx: &Context<'c>, id: i32) -> Result<UserResponse> {
        let pool = ctx.data::<crate::Pool>()?;

        let user = user::get_user_by_id(id, &pool).await?;
        Ok(user)
    }

    async fn users<'c>(
        &self,
        ctx: &Context<'c>,
        filter: Option<UserFilter>,
        order: Option<UserOrder>,
        criteria: UserCriteria,
    ) -> Result<Vec<UserResponse>> {
        let pool = ctx.data::<crate::Pool>()?;
        let index = ctx.data::<SearchIndex>()?.clone();

        let users = user::get_users(criteria, filter, order, &index, &pool).await?;

        Ok(users)
    }

    async fn top_users<'c>(&self, ctx: &Context<'c>) -> Result<Vec<UserResponse>> {
        let pool = ctx.data::<crate::Pool>()?;

        let users = user::top(&pool).await?;

        Ok(users)
    }

    async fn forums<'c>(
        &self,
        ctx: &Context<'c>,
        criteria: ForumCriteria,
        filter: Option<ForumFilter>,
        order: Option<forum::ForumOrderBy>,
    ) -> Result<Vec<forum::ForumResponse>> {
        let pool = ctx.data::<crate::Pool>()?;
        let index = ctx.data::<SearchIndex>()?.clone();

        let forums = forum::get_forums(criteria, filter, order, &index, &pool).await?;

        Ok(forums)
    }

    // async fn posts<'c>(
    //     &self,
    //     ctx: &Context<'c>,
    //     filter: Option<post::PostFilter>,
    //     criteria: post::PostCriteria,
    // ) -> Result<Vec<post::MultiPostReturn>> {
    //     let pool = ctx.data::<PostgresPool>()?.get()?;
    //     let index = ctx.data::<SearchIndex>()?.clone();
    //     let pool = ctx.data::<crate::SqlxPool>()?;

    //     let mut r = sqlx::query(
    //         "
    //     SELECT f.*,
    //         u.*,
    //         COUNT(DISTINCT up.id) AS number_of_users_posted,
    //         COUNT(p.id) AS number_of_posts,
    //         STRING_AGG(
    //             up.username,
    //             ', '
    //             ORDER BY up.username ASC
    //         ) AS users_who_posted
    //     FROM forums f
    //         JOIN users u ON f.owner_id = u.id
    //         LEFT JOIN posts p ON f.id = p.forum_id
    //         LEFT JOIN users up ON p.poster_id = up.id
    //     GROUP BY f.id,
    //         u.id;
    //     ",
    //     )
    //     .fetch(pool);

    //     while let Some(x) = r.try_next().await? {
    //         dbg!(x.columns());
    //     }

    //     let posts = actix_rt::task::spawn_blocking(move || {
    //         post::get_posts(filter, criteria, &index, &pool)
    //     })
    //     .await??;

    //     Ok(posts)
    // }

    // async fn comments<'c>(
    //     &self,
    //     ctx: &Context<'c>,
    //     filter: Option<comment::CommentFilter>,
    //     criteria: comment::CommentCriteria,
    // ) -> Result<Vec<CommentHierarchy>> {
    //     let pool = ctx.data::<PostgresPool>()?.get()?;
    //     let index = ctx.data::<SearchIndex>()?.clone();

    //     let comments =
    //         spawn_blocking!(comment::get_comments(filter, criteria, &index, &pool))??;

    //     Ok(comments)
    // }
}
