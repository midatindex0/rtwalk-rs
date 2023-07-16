use async_graphql::{Enum, InputObject, OneofObject, SimpleObject};
use diesel::dsl::{count, max};
use diesel::prelude::*;

use super::Page;
use crate::db::models::forum::Forum;
use crate::schema::forums::dsl::{id, name};
use crate::schema::posts;
use crate::schema::{comments, forums};
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
}

impl Default for ForumOrderBy {
    fn default() -> Self {
        Self::RecentPost
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

    let mut common = forums::table
        .inner_join(posts::table)
        .inner_join(comments::table)
        .group_by(forums::id)
        .having(count(posts::id).gt(filter.post_count.gt))
        .offset(filter.page.offset() as i64)
        .limit(filter.page.per as i64)
        .select((Forum::as_select(), count(posts::id), count(comments::id)))
        .into_boxed();

    common = match order {
        ForumOrderBy::Posts => common.order(count(posts::id).desc()),
        ForumOrderBy::Comments => common.order(count(comments::id).desc()),
        ForumOrderBy::Newest => common.order(forums::created_at.desc()),
        ForumOrderBy::Oldest => common.order(forums::created_at.asc()),
        ForumOrderBy::RecentPost =>  common.order(max(posts::created_at).desc()),
    };


    let x: Vec<MultiForumReturn> = match criteria {
        ForumCriteria::Search(query) => {
            let result = index.forum.search(
                &query,
                filter.page.offset() as usize,
                filter.page.per as usize,
            )?;
            let ids = result.ids();

            let _forums: Vec<(Forum, i64, i64)> = common
                .filter(id.eq_any(ids))
                .load::<(Forum, i64, i64)>(conn)?;

            let _forums = _forums
                .into_iter()
                .map(|x| MultiForumReturn {
                    score: result.map_id_score(x.0.id),
                    forum: x.0,
                    num_posts: x.1,
                    num_comments: x.2,
                })
                .collect();

            _forums
        }
        ForumCriteria::ByNames(names) => {
            let _forums: Vec<(Forum, i64, i64)> =
                common
                    .filter(name.eq_any(names))
                    .load::<(Forum, i64, i64)>(conn)?;

            let _forums = _forums
                .into_iter()
                .map(|x| MultiForumReturn {
                    forum: x.0,
                    num_posts: x.1,
                    num_comments: x.2,
                    score: None,
                })
                .collect();

            _forums
        }
        ForumCriteria::ByIds(ids) => {
            let _forums: Vec<(Forum, i64, i64)> = common
                .filter(id.eq_any(ids))
                .load::<(Forum, i64, i64)>(conn)?;

            let _forums = _forums
                .into_iter()
                .map(|x| MultiForumReturn {
                    forum: x.0,
                    num_posts: x.1,
                    num_comments: x.2,
                    score: None,
                })
                .collect();

            _forums
        }
    };
    Ok(x)
}
