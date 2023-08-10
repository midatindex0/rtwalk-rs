use async_graphql::{InputObject, OneofObject, SimpleObject};
use futures::StreamExt;
use sqlx::postgres::PgRow;
use sqlx::Row;

use super::{Page, PageOrder};
use crate::db::models::user::User;
use crate::search::SearchIndex;

#[derive(InputObject, Default)]
pub struct UserFilter {
    page: Option<Page>,
}

struct RawUserFilter {
    page: Page,
}

impl From<Option<UserFilter>> for RawUserFilter {
    fn from(value: Option<UserFilter>) -> Self {
        let value = value.unwrap_or_default();
        Self {
            page: value.page.unwrap_or_default(),
        }
    }
}

#[derive(OneofObject)]
pub enum UserCriteria {
    Search(String),
    ByUsernames(Vec<String>),
    ByIds(Vec<i32>),
}

#[derive(SimpleObject, Debug)]
pub struct UserResponse {
    pub user: User,
    pub owned_forums_count: i64,
    pub posts_count: i64,
    pub comments_count: i64,
    pub score: Option<i32>,
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
    index: &SearchIndex,
    pool: &crate::Pool,
) -> anyhow::Result<Vec<UserResponse>> {
    let filter: RawUserFilter = filter.into();
    let x: Vec<UserResponse> = match criteria {
        UserCriteria::Search(query) => {
            let results = index.user.search(&query)?;
            let ids = results.ids();

            let query_str = &format!("
                SELECT u.*, COUNT(f.id) AS owned_forums_count, COUNT(p.id) AS posts_count, COUNT(c.id) AS comments_count
                FROM users u
                LEFT JOIN forums f ON u.id = f.owner_id
                LEFT JOIN posts p ON u.id = p.poster_id
                LEFT JOIN comments c ON u.id = c.user_id
                WHERE u.id  = ANY($1) AND u.id {}= $2
                GROUP BY u.id
                ORDER BY u.id {}
                LIMIT $3;
            ", match filter.page.order {
                PageOrder::ASC => ">",
                PageOrder::DESC => "<"
            }, filter.page.order.as_str());

            let users = sqlx::query(&query_str)
                .bind(ids)
                .bind(filter.page.next_from)
                .bind(filter.page.per)
                .map(|row: PgRow| UserResponse {
                    user: User {
                        id: row.get("id"),
                        username: row.get("username"),
                        password: String::from("NA"),
                        display_name: row.get("display_name"),
                        bio: row.get("bio"),
                        pfp: row.get::<Option<String>, _>("pfp").into(),
                        banner: row.get::<Option<String>, _>("banner").into(),
                        created_at: row.get("created_at"),
                        v: row.get("v"),
                        admin: row.get("admin"),
                    },
                    owned_forums_count: row.get("owned_forums_count"),
                    posts_count: row.get("posts_count"),
                    comments_count: row.get("comments_count"),
                    score: None,
                })
                .fetch(pool)
                .filter_map(|x| async move { x.ok() })
                .collect::<Vec<_>>()
                .await;
            users
        }
        UserCriteria::ByUsernames(usernames) => todo!(),
        UserCriteria::ByIds(ids) => todo!(),
    };
    Ok(x)
}

pub async fn get_user_by_id(id: i32, pool: &crate::Pool) -> anyhow::Result<UserResponse> {
    let user = sqlx::query(
        "
        SELECT u.*, COUNT(f.id) AS owned_forums_count, COUNT(p.id) AS posts_count, COUNT(c.id) AS comments_count
        FROM users u
        LEFT JOIN forums f ON u.id = f.owner_id
        LEFT JOIN posts p ON u.id = p.poster_id
        LEFT JOIN comments c ON u.id = c.user_id
        WHERE u.id = $1
        GROUP BY u.id;"
    ).bind(id).fetch_one(pool).await.map(|row| UserResponse {
        user: User {
            id: row.get("id"),
            username: row.get("username"),
            password: String::from("NA"),
            display_name: row.get("display_name"),
            bio: row.get("bio"),
            pfp: row.get::<Option<String>, _>("pfp").into(),
            banner: row.get::<Option<String>, _>("banner").into(),
            created_at: row.get("created_at"),
            v: row.get("v"),
            admin: row.get("admin"),
        },
        owned_forums_count: row.get("owned_forums_count"),
        posts_count: row.get("posts_count"),
        comments_count: row.get("comments_count"),
        score: None,
    })?;
    Ok(user)
}
