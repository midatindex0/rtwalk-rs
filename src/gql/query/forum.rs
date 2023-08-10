use async_graphql::{Enum, InputObject, OneofObject, SimpleObject};
use diesel::dsl::{count, max};
use diesel::{prelude::*, sql_query};
use serde::Serialize;

use super::Page;
use crate::db::models::comment::Comment;
use crate::db::models::forum::Forum;
use crate::db::models::post::Post;
use crate::db::models::user::User;
use crate::schema::forums::dsl::{id, name};
use crate::schema::{comments, forums};
use crate::schema::{posts, users};
use crate::search::SearchIndex;

#[derive(InputObject, Default)]
pub struct ForumFilter {
    page: Option<Page>,
    post_count: Option<PostForumFilter>,
}

// TODO: Implement
#[derive(InputObject, Default)]
struct PostForumFilter {
    gt: i64,
}

struct RawForumFilter {
    page: Page,
    post_count: PostForumFilter,
}

impl From<Option<ForumFilter>> for RawForumFilter {
    fn from(value: Option<ForumFilter>) -> Self {
        let value = value.unwrap_or_default();
        Self {
            page: value.page.unwrap_or_default(),
            post_count: value.post_count.unwrap_or_default(),
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
pub struct MultiForumReturn {
    pub forum: Forum,
    pub num_posts: i64,
    pub num_comments: i64,
    pub score: Option<f32>,
}

pub fn get_forums(
    criteria: ForumCriteria,
    filter: Option<ForumFilter>,
    order: Option<ForumOrderBy>,
    index: &SearchIndex,
    conn: &mut crate::Conn,
) -> anyhow::Result<Vec<MultiForumReturn>> {
    let filter: RawForumFilter = filter.into();

    let order = order.unwrap_or_default();

    // TODO

    let mut common = forums::table
        .left_join(posts::table)
        .left_join(comments::table)
        .group_by(forums::id)
        .offset(filter.page.offset() as i64)
        .limit(filter.page.per as i64)
        .select(Forum::as_select())
        .into_boxed();

    common = match order {
        ForumOrderBy::Posts => common.order(count(posts::id).desc()),
        ForumOrderBy::Comments => common.order(count(comments::id).desc()),
        ForumOrderBy::Newest => common.order(forums::created_at.desc()),
        ForumOrderBy::Oldest => common.order(forums::created_at.asc()),
        ForumOrderBy::RecentPost => common.order(max(posts::created_at).desc()),
        ForumOrderBy::None => common,
    };

    let x: Vec<MultiForumReturn> = match criteria {
        ForumCriteria::Search(query) => {
            let result = index.forum.search(&query, 0, 500)?;
            let ids: Vec<i32> = result.ids();

            let _forums: Vec<Forum> = common.filter(id.eq_any(ids)).load::<Forum>(conn)?;

            let _forums = _forums
                .into_iter()
                .map(|x| MultiForumReturn {
                    score: result.map_id_score(x.id),
                    forum: x,
                    num_posts: 0,
                    num_comments: 0,
                })
                .collect();

            _forums
        }
        // ForumCriteria::ByNames(names) => {
        //     let _forums: Vec<(Forum, i64, i64)> =
        //         common
        //             .filter(name.eq_any(names))
        //             .load::<(Forum, i64, i64)>(conn)?;

        //     let _forums = _forums
        //         .into_iter()
        //         .map(|x| MultiForumReturn {
        //             forum: x.0,
        //             num_posts: x.1,
        //             num_comments: x.2,
        //             score: None,
        //         })
        //         .collect();

        //     _forums
        // }
        // ForumCriteria::ByIds(ids) => {
        //     let _forums: Vec<(Forum, i64, i64)> = common
        //         .filter(id.eq_any(ids))
        //         .load::<(Forum, i64, i64)>(conn)?;

        //     let _forums = _forums
        //         .into_iter()
        //         .map(|x| MultiForumReturn {
        //             forum: x.0,
        //             num_posts: x.1,
        //             num_comments: x.2,
        //             score: None,
        //         })
        //         .collect();

        //     _forums
        // }
        _ => todo!(),
    };
    Ok(x)
}
