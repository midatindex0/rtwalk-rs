use async_graphql::{InputObject, OneofObject};
use diesel::prelude::*;

use crate::db::models::post::Post;
use crate::schema::posts::dsl::*;

#[derive(OneofObject)]
pub enum PostOrder {
    Limit(#[graphql(validator(min = 0, max = 100))] i64),
    IdFrom(#[graphql(validator(min = 1))] i32),
    IdTill(#[graphql(validator(min = 1))] i32),
    IdRange(IdRange),
    Before(chrono::NaiveDateTime),
    After(chrono::NaiveDateTime),
}

#[derive(InputObject)]
pub struct IdRange {
    start: i32,
    end: i32,
}

pub fn get_posts(
    _forum_id: i32,
    order: &PostOrder,
    conn: &mut crate::Conn,
) -> anyhow::Result<Vec<Post>> {
    let _posts: Vec<Post> = match *order {
        PostOrder::Limit(x) => posts
            .filter(forum_id.eq(forum_id))
            .limit(x)
            .select(Post::as_select())
            .load(conn)?,
        PostOrder::IdFrom(x) => posts
            .filter(forum_id.eq(forum_id))
            .filter(id.ge(x))
            .limit(100)
            .select(Post::as_select())
            .load(conn)?,
        PostOrder::IdTill(x) => posts
            .filter(forum_id.eq(forum_id))
            .filter(id.le(x))
            .limit(100)
            .select(Post::as_select())
            .load(conn)?,
        PostOrder::IdRange(IdRange { start, end }) => posts
            .filter(forum_id.eq(forum_id))
            .filter(id.ge(start))
            .filter(id.le(end))
            .limit(100)
            .select(Post::as_select())
            .load(conn)?,
        PostOrder::Before(x) => posts
            .filter(forum_id.eq(forum_id))
            .filter(created_at.le(x))
            .limit(100)
            .select(Post::as_select())
            .load(conn)?,
        PostOrder::After(x) => posts
            .filter(forum_id.eq(forum_id))
            .filter(created_at.ge(x))
            .limit(100)
            .select(Post::as_select())
            .load(conn)?,
    };
    Ok(_posts)
}
