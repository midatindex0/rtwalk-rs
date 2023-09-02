use async_graphql::{Enum, InputObject, OneofObject, SimpleObject};
use futures::StreamExt;
use sqlx::postgres::PgRow;
use sqlx::{FromRow, Row};

use super::{Page, PageOrder, RawPage};
use crate::db::models::forum::Forum;
use crate::db::models::post::Post;
use crate::db::models::user::User;
use crate::search::SearchIndex;

#[derive(InputObject, Default)]
struct StarFilter {
    gt: i32,
}

#[derive(InputObject, Default)]
pub struct PostFilter {
    page: Option<Page>,
    star_count: Option<StarFilter>,
}

struct RawPostFilter {
    page: RawPage,
    star: StarFilter,
}

impl From<Option<PostFilter>> for RawPostFilter {
    fn from(value: Option<PostFilter>) -> Self {
        let value = value.unwrap_or_default();
        Self {
            page: value.page.into(),
            star: value.star_count.unwrap_or_default(),
        }
    }
}

#[derive(OneofObject)]
pub enum PostCriteria {
    Search(String),
    BySlugs(Vec<String>),
    ByIds(Vec<i32>),
}

#[derive(SimpleObject)]
pub struct PostResponse {
    pub post: Post,
    // pub poster: User,
    // pub forum: Forum,
    // pub comment_count: i64,
    // pub participant_count: i64,
    pub score: Option<f32>,
}

pub async fn get_posts(
    criteria: PostCriteria,
    filter: Option<PostFilter>,
    index: &SearchIndex,
    pool: &crate::Pool,
) -> anyhow::Result<Vec<PostResponse>> {
    let filter: RawPostFilter = filter.into();

    let query_str = format!(
        "
        SELECT p.*
        FROM posts p
        LEFT JOIN forums f ON f.id = p.forum_id
        LEFT JOIN users poster ON poster.id = p.poster_id
        WHERE p.{} = ANY($3) AND p.id {}= $1
        GROUP BY p.id
        ORDER BY p.id {}
        LIMIT $2;
    ",
        match &criteria {
            PostCriteria::Search(_) | PostCriteria::ByIds(_) => "id",
            PostCriteria::BySlugs(_) => "slug",
        },
        match filter.page.order {
            PageOrder::ASC => ">",
            PageOrder::DESC => "<",
        },
        filter.page.order.as_str()
    );

    println!("{}", &query_str);

    let sql_query = sqlx::query(query_str.as_str())
        .bind(filter.page.next_from)
        .bind(filter.page.per);

    let x: Vec<PostResponse> = match criteria {
        PostCriteria::Search(query) => {
            let results = index.post.search(&query)?;
            let ids = results.ids();

            let posts = sql_query
                .bind(ids)
                .map(|row: PgRow| PostResponse {
                    // cant fail because we know the fields
                    post: Post::from_row(&row).unwrap(),
                    score: results.map_id_score(row.get("id")),
                })
                .fetch(pool)
                .filter_map(|x| async move { x.ok() })
                .collect::<Vec<_>>()
                .await;
            posts
        }
        PostCriteria::BySlugs(slugs) => {
            let posts = sql_query
                .bind(slugs)
                .map(|row: PgRow| PostResponse {
                    // cant fail because we know the fields
                    post: Post::from_row(&row).unwrap(),
                    score: None,
                })
                .fetch(pool)
                .filter_map(|x| async move { x.ok() })
                .collect::<Vec<_>>()
                .await;
            posts
        }
        PostCriteria::ByIds(ids) => {
            let posts = sql_query
                .bind(ids)
                .map(|row: PgRow| PostResponse {
                    // cant fail because we know the fields
                    post: Post::from_row(&row).unwrap(),
                    score: None,
                })
                .fetch(pool)
                .filter_map(|x| async move { x.ok() })
                .collect::<Vec<_>>()
                .await;
            posts
        }
    };

    Ok(x)
}

pub async fn get_post_by_slug(slug: &String, pool: &crate::Pool) -> anyhow::Result<PostResponse> {
    let post = sqlx::query(
        "
            SELECT p.*
            FROM posts p
            WHERE p.slug = $1
            GROUP BY p.id;",
    )
    .bind(slug)
    .fetch_one(pool)
    .await
    .map(|row| PostResponse {
        // cant fail because we know the fields
        post: Post::from_row(&row).unwrap(),
        score: None,
    })?;
    Ok(post)
}
