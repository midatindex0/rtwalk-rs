use async_graphql::{InputObject, OneofObject};
use diesel::prelude::*;

use super::Page;
use crate::db::models::forum::Forum;
use crate::schema::forums::dsl::*;
use crate::search::SearchIndex;

#[derive(InputObject, Default)]
pub struct ForumFilter {
    page: Option<Page>,
    post_count: Option<PostForumFilter>,
    participant_count: Option<ParticipantForumFilter>,
}

// TODO: Implement
#[derive(InputObject)]
struct PostForumFilter {
    gt: usize,
}
//TODO: Implement
#[derive(InputObject)]
struct ParticipantForumFilter {
    gt: usize,
}

struct RawForumFilter {
    page: Page,
}

impl From<Option<ForumFilter>> for RawForumFilter {
    fn from(value: Option<ForumFilter>) -> Self {
        let value = value.unwrap_or_default();
        Self {
            page: value.page.unwrap_or_default(),
        }
    }
}

#[derive(OneofObject)]
pub enum ForumCriteria {
    Search(String),
    ByNames(Vec<String>),
    ByIds(Vec<i32>),
}

// TODO: Add order by [struct Order]

pub fn get_forums(
    criteria: ForumCriteria,
    filter: Option<ForumFilter>,
    index: &SearchIndex,
    conn: &mut crate::Conn,
) -> anyhow::Result<Vec<Forum>> {
    let filter: RawForumFilter = filter.into();
    let x: Vec<Forum> = match criteria {
        ForumCriteria::Search(query) => {
            let result =
                index
                    .forum
                    .search(&query, filter.page.offset(), filter.page.per as usize)?;
            let ids = result.ids();

            forums.filter(id.eq_any(ids)).load::<Forum>(conn)?
        }
        ForumCriteria::ByNames(names) => forums
            .filter(name.eq_any(names))
            .offset(filter.page.offset() as i64)
            .limit(filter.page.per as i64)
            .load::<Forum>(conn)?,
        ForumCriteria::ByIds(ids) => forums
            .filter(id.eq_any(ids))
            .offset(filter.page.offset() as i64)
            .limit(filter.page.per as i64)
            .load::<Forum>(conn)?,
    };
    Ok(x)
}
