use async_graphql::{InputObject, OneofObject, SimpleObject};
use diesel::prelude::*;

use super::Page;
use crate::db::models::forum::Forum;
use crate::db::models::post::{Post, RawPost};
use crate::db::models::user::User;
use crate::schema::posts::dsl::*;
use crate::schema::{forums, users};
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
    page: Page,
    star: StarFilter,
}

impl From<Option<PostFilter>> for RawPostFilter {
    fn from(value: Option<PostFilter>) -> Self {
        let value = value.unwrap_or_default();
        Self {
            page: value.page.unwrap_or_default(),
            star: value.star_count.unwrap_or_default(),
        }
    }
}

#[derive(OneofObject)]
pub enum PostCriteria {
    Search(String),
    BySlugs(Vec<String>),
    ByIds(Vec<i32>),
    ByForumId(i32),
}

#[derive(SimpleObject)]
pub struct MultiPostReturn {
    pub post: RawPost,
    pub poster: User,
    pub forum: Forum,
    // TODO:
    // pub participants: Vec<User>,
    pub score: Option<f32>,
}

pub fn get_posts(
    filter: Option<PostFilter>,
    criteria: PostCriteria,
    index: &SearchIndex,
    conn: &mut crate::Conn,
) -> anyhow::Result<Vec<MultiPostReturn>> {
    let filter: RawPostFilter = filter.into();
    let _posts: Vec<MultiPostReturn> = match criteria {
        PostCriteria::Search(query) => {
            let result =
                index
                    .post
                    .search(&query, filter.page.offset(), filter.page.per as usize)?;

            let ids = result.ids();

            let _posts: Vec<(Post, User, Forum)> = posts
                .inner_join(users::table)
                .inner_join(forums::table)
                .filter(id.eq_any(ids))
                .filter(stars.gt(filter.star.gt))
                .offset(filter.page.offset() as i64)
                .limit(filter.page.per as i64)
                .select((Post::as_select(), User::as_select(), Forum::as_select()))
                .load(conn)?;

            let _posts = _posts
                .into_iter()
                .map(|x| MultiPostReturn {
                    score: result.map_id_score(x.0.id),
                    post: RawPost::from(x.0),
                    poster: x.1,
                    forum: x.2,
                })
                .collect();
            _posts
        }
        PostCriteria::BySlugs(slugs) => {
            let _posts: Vec<(Post, User, Forum)> = posts
                .inner_join(users::table)
                .inner_join(forums::table)
                .filter(slug.eq_any(slugs))
                .filter(stars.gt(filter.star.gt))
                .offset(filter.page.offset() as i64)
                .limit(filter.page.per as i64)
                .select((Post::as_select(), User::as_select(), Forum::as_select()))
                .load(conn)?;

            let _posts = _posts
                .into_iter()
                .map(|x| MultiPostReturn {
                    score: None,
                    post: RawPost::from(x.0),
                    poster: x.1,
                    forum: x.2,
                })
                .collect();
            _posts
        }
        PostCriteria::ByIds(ids) => {
            let _posts: Vec<(Post, User, Forum)> = posts
                .inner_join(users::table)
                .inner_join(forums::table)
                .filter(id.eq_any(ids))
                .filter(stars.gt(filter.star.gt))
                .offset(filter.page.offset() as i64)
                .limit(filter.page.per as i64)
                .select((Post::as_select(), User::as_select(), Forum::as_select()))
                .load(conn)?;

            let _posts = _posts
                .into_iter()
                .map(|x| MultiPostReturn {
                    score: None,
                    post: RawPost::from(x.0),
                    poster: x.1,
                    forum: x.2,
                })
                .collect();
            _posts
        }
        PostCriteria::ByForumId(fid) => {
            let _posts: Vec<(Post, User, Forum)> = posts
                .inner_join(users::table)
                .inner_join(forums::table)
                .filter(forum_id.eq(fid))
                .filter(stars.gt(filter.star.gt))
                .offset(filter.page.offset() as i64)
                .limit(filter.page.per as i64)
                .select((Post::as_select(), User::as_select(), Forum::as_select()))
                .load(conn)?;

            let _posts = _posts
                .into_iter()
                .map(|x| MultiPostReturn {
                    score: None,
                    post: RawPost::from(x.0),
                    poster: x.1,
                    forum: x.2,
                })
                .collect();
            _posts
        }
    };
    Ok(_posts)
}
