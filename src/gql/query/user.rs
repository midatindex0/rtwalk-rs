use async_graphql::{Enum, InputObject, OneofObject, SimpleObject};
use futures::StreamExt;
use sqlx::postgres::PgRow;
use sqlx::{FromRow, Row};

use super::{Page, PageOrder, RawPage};
use crate::db::models::user::User;
use crate::search::SearchIndex;

#[derive(InputObject, Default)]
pub struct UserFilter {
    page: Option<Page>,
    forum_count: Option<UserForumFilter>,
    post_count: Option<UserPostFilter>,
    comment_count: Option<UserCommentFilter>,
    stars: Option<UserStarFilter>,
    admin: Option<bool>,
}

#[derive(InputObject)]
struct UserForumFilter {
    ge: i64,
    le: Option<i64>,
}

struct RawUserForumFilter {
    ge: i64,
    le: i64,
}

impl From<Option<UserForumFilter>> for RawUserForumFilter {
    fn from(value: Option<UserForumFilter>) -> Self {
        match value {
            Some(filter) => Self {
                ge: filter.ge,
                le: filter.le.unwrap_or(i64::MAX),
            },
            None => Self {
                ge: 0,
                le: i64::MAX,
            },
        }
    }
}

#[derive(InputObject)]
struct UserPostFilter {
    ge: i64,
    le: Option<i64>,
}

struct RawUserPostFilter {
    ge: i64,
    le: i64,
}

impl From<Option<UserPostFilter>> for RawUserPostFilter {
    fn from(value: Option<UserPostFilter>) -> Self {
        match value {
            Some(filter) => Self {
                ge: filter.ge,
                le: filter.le.unwrap_or(i64::MAX),
            },
            None => Self {
                ge: 0,
                le: i64::MAX,
            },
        }
    }
}

#[derive(InputObject)]
struct UserCommentFilter {
    ge: i64,
    le: Option<i64>,
}

struct RawUserCommentFilter {
    ge: i64,
    le: i64,
}

impl From<Option<UserCommentFilter>> for RawUserCommentFilter {
    fn from(value: Option<UserCommentFilter>) -> Self {
        match value {
            Some(filter) => Self {
                ge: filter.ge,
                le: filter.le.unwrap_or(i64::MAX),
            },
            None => Self {
                ge: 0,
                le: i64::MAX,
            },
        }
    }
}

#[derive(InputObject)]
struct UserStarFilter {
    ge: i64,
    le: Option<i64>,
}

struct RawUserStarFilter {
    ge: i64,
    le: i64,
}

impl From<Option<UserStarFilter>> for RawUserStarFilter {
    fn from(value: Option<UserStarFilter>) -> Self {
        match value {
            Some(filter) => Self {
                ge: filter.ge,
                le: filter.le.unwrap_or(i64::MAX),
            },
            None => Self {
                ge: 0,
                le: i64::MAX,
            },
        }
    }
}

struct RawUserFilter {
    page: RawPage,
    forum_count: RawUserForumFilter,
    post_coubt: RawUserPostFilter,
    comment_count: RawUserCommentFilter,
    stars: RawUserStarFilter,
    admin_str: &'static str,
}

impl From<Option<UserFilter>> for RawUserFilter {
    fn from(value: Option<UserFilter>) -> Self {
        let value = value.unwrap_or_default();
        Self {
            page: value.page.into(),
            forum_count: value.forum_count.into(),
            post_coubt: value.post_count.into(),
            comment_count: value.comment_count.into(),
            stars: value.stars.into(),
            admin_str: match value.admin {
                None => "IS NOT NULL",
                Some(true) => "= true",
                Some(false) => "= false",
            },
        }
    }
}

#[derive(OneofObject)]
pub enum UserCriteria {
    Search(String),
    ByUsernames(Vec<String>),
    ByIds(Vec<i32>),
}

#[derive(InputObject)]
pub struct UserOrder {
    ty: UserOrderType,
    last_id: Option<i32>,
}

#[derive(Enum, PartialEq, Eq, Clone, Copy)]
pub enum UserOrderType {
    None,
    OwnedForumCountAsc,
    OwnedForumCountDesc,
    PostCountAsc,
    PostCountDesc,
    CommentCountAsc,
    CommentCountDesc,
}

struct RawUserOrder {
    ty: UserOrderType,
    last_id: i32,
}

impl From<Option<UserOrder>> for RawUserOrder {
    fn from(value: Option<UserOrder>) -> Self {
        match value {
            None => Self {
                ty: UserOrderType::None,
                last_id: 0,
            },
            Some(order) => Self {
                ty: order.ty,
                last_id: order.last_id.unwrap_or(match order.ty {
                    UserOrderType::None
                    | UserOrderType::CommentCountAsc
                    | UserOrderType::OwnedForumCountAsc
                    | UserOrderType::PostCountAsc => 0,
                    UserOrderType::CommentCountDesc
                    | UserOrderType::OwnedForumCountDesc
                    | UserOrderType::PostCountDesc => i32::MAX,
                }),
            },
        }
    }
}

#[derive(SimpleObject, Debug)]
pub struct UserResponse {
    pub user: User,
    pub owned_forum_count: i64,
    pub post_count: i64,
    pub comment_count: i64,
    pub stars: i64,
    pub score: Option<f32>,
}

#[derive(SimpleObject, Debug)]
pub struct MultiUserResponse {
    pub data: Vec<UserResponse>,
    pub next_from: i32,
    pub total: i64,
}

pub async fn get_users(
    criteria: UserCriteria,
    filter: Option<UserFilter>,
    order: Option<UserOrder>,
    index: &SearchIndex,
    pool: &crate::Pool,
) -> anyhow::Result<Vec<UserResponse>> {
    let filter: RawUserFilter = filter.into();
    let order: RawUserOrder = order.into();

    let query_str = format!("
        SELECT u.*, SUM(p.stars) as stars, COUNT(f.id) AS owned_forum_count, COUNT(p.id) AS post_count, COUNT(c.id) AS comment_count
        FROM users u
        LEFT JOIN forums f ON u.id = f.owner_id
        LEFT JOIN posts p ON u.id = p.poster_id
        LEFT JOIN comments c ON u.id = c.user_id
        WHERE u.{} = ANY($11) AND u.id {}= $1 AND admin {}
        GROUP BY u.id
        -- Filters
        HAVING COUNT(f.id) >= $2 AND COUNT(f.id) <= $3 AND COUNT(p.id) >= $4 AND COUNT(p.id) <= $5 AND COUNT(c.id) >= $6 AND COUNT(c.id) <= $7 AND SUM(p.stars) >= $8 AND SUM(p.stars) <= $9
        -- Filters end
        ORDER BY u.id {}
        LIMIT $10;
    ", match &criteria {
        UserCriteria::Search(_) | UserCriteria::ByIds(_) => {"id"},
        UserCriteria::ByUsernames(_) => "username"
    }, match filter.page.order {
        PageOrder::ASC => ">",
        PageOrder::DESC => "<"
    }, filter.admin_str, filter.page.order.as_str());


    let sql_query = sqlx::query(query_str.as_str())
        .bind(filter.page.next_from)
        .bind(filter.forum_count.ge)
        .bind(filter.forum_count.le)
        .bind(filter.post_coubt.ge)
        .bind(filter.post_coubt.le)
        .bind(filter.comment_count.ge)
        .bind(filter.comment_count.le)
        .bind(filter.stars.ge)
        .bind(filter.stars.le)
        .bind(filter.page.per);
    let x: Vec<UserResponse> = match criteria {
        UserCriteria::Search(query) => {
            let results = index.user.search(&query)?;
            let ids = results.ids();

            let users = sql_query
                .bind(ids)
                .map(|row: PgRow| UserResponse {
                    // cant fail because we know the fields
                    user: User::from_row(&row).unwrap(),
                    owned_forum_count: row.get("owned_forum_count"),
                    post_count: row.get("post_count"),
                    comment_count: row.get("comment_count"),
                    stars: row.get("stars"),
                    score: results.map_id_score(row.get("id")),
                })
                .fetch(pool)
                .filter_map(|x| async move { x.ok() })
                .collect::<Vec<_>>()
                .await;
            users
        }
        UserCriteria::ByUsernames(usernames) => {
            let users = sql_query
                .bind(usernames)
                .map(|row: PgRow| UserResponse {
                    // cant fail because we know the fields
                    user: User::from_row(&row).unwrap(),
                    owned_forum_count: row.get("owned_forum_count"),
                    post_count: row.get("post_count"),
                    comment_count: row.get("comment_count"),
                    stars: row.get("stars"),
                    score: None,
                })
                .fetch(pool)
                .filter_map(|x| async move { x.ok() })
                .collect::<Vec<_>>()
                .await;
            users
        }
        UserCriteria::ByIds(ids) => {
            let users = sql_query
                .bind(ids)
                .map(|row: PgRow| UserResponse {
                    // cant fail because we know the fields
                    user: User::from_row(&row).unwrap(),
                    owned_forum_count: row.get("owned_forum_count"),
                    post_count: row.get("post_count"),
                    comment_count: row.get("comment_count"),
                    stars: row.get("stars"),
                    score: None,
                })
                .fetch(pool)
                .filter_map(|x| async move { x.ok() })
                .collect::<Vec<_>>()
                .await;
            users
        }
    };
    Ok(x)
}

pub async fn top(pool: &crate::Pool) -> anyhow::Result<Vec<UserResponse>> {
    let users = sqlx::query("
        SELECT u.*, SUM(p.stars) as stars, COUNT(f.id) AS owned_forum_count, COUNT(p.id) AS post_count, COUNT(c.id) AS comment_count
        FROM users u
        LEFT JOIN forums f ON u.id = f.owner_id
        LEFT JOIN posts p ON u.id = p.poster_id
        LEFT JOIN comments c ON u.id = c.user_id
        GROUP BY u.id
        HAVING SUM(p.stars) >= 0
        ORDER BY SUM(p.stars)
        LIMIT 50;
    ").map(|row: PgRow| UserResponse {
        // cant fail because we know the fields
        user: User::from_row(&row).unwrap(),
        owned_forum_count: row.get("owned_forum_count"),
        post_count: row.get("post_count"),
        comment_count: row.get("comment_count"),
        stars: row.get("stars"),
        score: None,
    })
    .fetch(pool)
    .filter_map(|x| async move { x.ok() })
    .collect::<Vec<_>>()
    .await;
    Ok(users)
}

pub async fn get_user_by_id(id: i32, pool: &crate::Pool) -> anyhow::Result<UserResponse> {
    let user = sqlx::query(
        "
        SELECT u.*, SUM(p.stars) as stars, COUNT(f.id) AS owned_forum_count, COUNT(p.id) AS post_count, COUNT(c.id) AS comment_count
        FROM users u
        LEFT JOIN forums f ON u.id = f.owner_id
        LEFT JOIN posts p ON u.id = p.poster_id
        LEFT JOIN comments c ON u.id = c.user_id
        WHERE u.id = $1
        GROUP BY u.id;"
    ).bind(id).fetch_one(pool).await.map(|row| UserResponse {
        // cant fail because we know the fields
        user: User::from_row(&row).unwrap(),
        owned_forum_count: row.get("owned_forum_count"),
        post_count: row.get("post_count"),
        comment_count: row.get("comment_count"),
        stars: row.get("stars"),
        score: None,
    })?;
    Ok(user)
}
