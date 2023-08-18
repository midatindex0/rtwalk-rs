use anyhow::Ok;
use async_graphql::{Enum, InputObject, OneofObject, SimpleObject};
use futures::StreamExt;
use sqlx::postgres::PgRow;
use sqlx::{FromRow, Row};

use super::{Page, RawPage};
use crate::db::models::forum::Forum;
use crate::db::models::user::User;
use crate::gql::query::PageOrder;
use crate::search::SearchIndex;

#[derive(InputObject, Default)]
pub struct ForumFilter {
    page: Option<Page>,
}

// TODO: Implement
#[derive(InputObject, Default)]
struct PostForumFilter {
    gt: i64,
}

struct RawForumFilter {
    page: RawPage,
}

impl From<Option<ForumFilter>> for RawForumFilter {
    fn from(value: Option<ForumFilter>) -> Self {
        let value = value.unwrap_or_default();
        Self {
            page: value.page.into(),
        }
    }
}

#[derive(OneofObject)]
pub enum ForumCriteria {
    Search(String),
    ByNames(Vec<String>),
    ByIds(Vec<i32>),
}

#[derive(Enum, Clone, Copy, Eq, PartialEq)]
pub enum ForumOrderBy {
    Posts,
    Comments,
    Newest,
    Oldest,
    RecentPost,
    None,
}

impl Default for ForumOrderBy {
    fn default() -> Self {
        Self::None
    }
}

#[derive(SimpleObject)]
pub struct ForumResponse {
    pub forum: Forum,
    // pub stars: i64,
    pub post_count: i64,
    pub comment_count: i64,
    pub post_participant_count: i64,
    pub comment_participant_count: i64,
    pub score: Option<f32>,
}

pub async fn get_forums(
    criteria: ForumCriteria,
    filter: Option<ForumFilter>,
    order: Option<ForumOrderBy>,
    index: &SearchIndex,
    pool: &crate::Pool,
) -> anyhow::Result<Vec<ForumResponse>> {
    let filter: RawForumFilter = filter.into();
    let query_str = format!(
        "
        SELECT f.*, COUNT(p.id) AS post_count, COUNT(c.id) AS comment_count, COUNT(DISTINCT pu.id) as post_participant_count, COUNT(DISTINCT cu.id) as comment_participant_count
        FROM forums f
        LEFT JOIN posts p ON f.id = p.forum_id
        LEFT JOIN users pu ON p.poster_id = pu.id
        LEFT JOIN comments c ON f.id = c.forum_id
        LEFT JOIN users cu ON c.user_id = cu.id
        WHERE f.{} = ANY($3) AND f.id {}= $1
        GROUP BY f.id
        ORDER BY f.id {}
        LIMIT $2;
    ",
        match &criteria {
            ForumCriteria::Search(_) | ForumCriteria::ByIds(_) => "id",
            ForumCriteria::ByNames(_) => "name",
        },
        match filter.page.order {
            PageOrder::ASC => ">",
            PageOrder::DESC => "<",
        },
        filter.page.order.as_str()
    );

    let sql_query = sqlx::query(query_str.as_str())
        .bind(filter.page.next_from)
        .bind(filter.page.per);
    let x = match criteria {
        ForumCriteria::Search(query) => {
            let results = index.forum.search(&query)?;
            let ids = results.ids();

            let forums = sql_query
                .bind(ids)
                .map(|row: PgRow| ForumResponse {
                    // cant fail because we know the fields
                    forum: Forum::from_row(&row).unwrap(),
                    post_count: row.get("post_count"),
                    comment_count: row.get("comment_count"),
                    post_participant_count: row.get("post_participant_count"),
                    comment_participant_count: row.get("comment_participant_count"),
                    score: results.map_id_score(row.get("id")),
                })
                .fetch(pool)
                .filter_map(|x| async move { x.ok() })
                .collect::<Vec<_>>()
                .await;
            forums
        }
        ForumCriteria::ByIds(ids) => {
            let forums = sql_query
                .bind(ids)
                .map(|row: PgRow| ForumResponse {
                    // cant fail because we know the fields
                    forum: Forum::from_row(&row).unwrap(),
                    post_count: row.get("post_count"),
                    comment_count: row.get("comment_count"),
                    post_participant_count: row.get("post_participant_count"),
                    comment_participant_count: row.get("comment_participant_count"),
                    score: None,
                })
                .fetch(pool)
                .filter_map(|x| async move { x.ok() })
                .collect::<Vec<_>>()
                .await;
            forums
        }
        ForumCriteria::ByNames(names) => {
            let forums = sql_query
                .bind(names)
                .map(|row: PgRow| ForumResponse {
                    // cant fail because we know the fields
                    forum: Forum::from_row(&row).unwrap(),
                    post_count: row.get("post_count"),
                    comment_count: row.get("comment_count"),
                    post_participant_count: row.get("post_participant_count"),
                    comment_participant_count: row.get("comment_participant_count"),
                    score: None,
                })
                .fetch(pool)
                .filter_map(|x| async move { x.ok() })
                .collect::<Vec<_>>()
                .await;
            forums
        }
    };

    Ok(x)
}
