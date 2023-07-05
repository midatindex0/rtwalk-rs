use async_graphql::{InputObject, OneofObject, SimpleObject};
use diesel::prelude::*;

use crate::db::models::post::{Post, PostWithoutUser};
use crate::db::models::user::User;
use crate::schema::posts::dsl::*;
use crate::schema::users;

#[derive(OneofObject)]
pub enum PostCriteria {
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

#[derive(SimpleObject)]
pub struct PostWithUser {
    pub post: PostWithoutUser,
    pub poster: User,
}

pub fn get_posts(
    _forum_id: i32,
    order: &PostCriteria,
    conn: &mut crate::Conn,
) -> anyhow::Result<Vec<(Post, User)>> {
    let _posts: Vec<(Post, User)> = match *order {
        PostCriteria::Limit(x) => posts
            .inner_join(users::table)
            .filter(forum_id.eq(forum_id))
            .order(created_at.desc())
            .limit(x)
            .select((Post::as_select(), User::as_select()))
            .load(conn)?,
        PostCriteria::IdFrom(x) => posts
            .inner_join(users::table)
            .filter(forum_id.eq(forum_id))
            .filter(id.ge(x))
            .order(created_at.desc())
            .limit(100)
            .select((Post::as_select(), User::as_select()))
            .load(conn)?,
        PostCriteria::IdTill(x) => posts
            .inner_join(users::table)
            .filter(forum_id.eq(forum_id))
            .filter(id.le(x))
            .order(created_at.desc())
            .limit(100)
            .select((Post::as_select(), User::as_select()))
            .load(conn)?,
        PostCriteria::IdRange(IdRange { start, end }) => posts
            .inner_join(users::table)
            .filter(forum_id.eq(forum_id))
            .filter(id.ge(start))
            .filter(id.le(end))
            .order(created_at.desc())
            .limit(100)
            .select((Post::as_select(), User::as_select()))
            .load(conn)?,
        PostCriteria::Before(x) => posts
            .inner_join(users::table)
            .filter(forum_id.eq(forum_id))
            .filter(created_at.le(x))
            .order(created_at.desc())
            .limit(100)
            .select((Post::as_select(), User::as_select()))
            .load(conn)?,
        PostCriteria::After(x) => posts
            .inner_join(users::table)
            .filter(forum_id.eq(forum_id))
            .filter(created_at.ge(x))
            .order(created_at.desc())
            .limit(100)
            .select((Post::as_select(), User::as_select()))
            .load(conn)?,
    };
    Ok(_posts)
}
