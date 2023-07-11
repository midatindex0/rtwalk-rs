use async_graphql::{InputObject, OneofObject};
use diesel::prelude::*;

use crate::db::models::comment::{Comment, CommentHierarchy};
use crate::db::models::user::User;
use crate::schema::comments::dsl::*;
use crate::schema::users;
use crate::search::SearchIndex;

use super::Page;

#[derive(InputObject, Default)]
pub struct CommentFilter {
    page: Option<Page>,
    _parent_id: Option<i32>,
}

struct RawCommentFilter {
    page: Page,
    _parent_id: Option<i32>,
}

impl From<Option<CommentFilter>> for RawCommentFilter {
    fn from(value: Option<CommentFilter>) -> Self {
        let value = value.unwrap_or_default();
        Self {
            page: value.page.unwrap_or_default(),
            _parent_id: value._parent_id,
        }
    }
}

#[derive(OneofObject)]
pub enum CommentCriteria {
    Search(String), // TODO
    ByPostId(i32),
}

pub fn get_comments(
    filter: Option<CommentFilter>,
    criteria: CommentCriteria,
    _index: &SearchIndex,
    conn: &mut crate::Conn,
) -> anyhow::Result<Vec<CommentHierarchy>> {
    let filter: RawCommentFilter = filter.into();
    let _comments: Vec<CommentHierarchy> = match criteria {
        CommentCriteria::Search(_) => {
            return Err(anyhow::Error::msg("Not supported yet"));
        }
        CommentCriteria::ByPostId(_post_id) => {
            let _comments: Vec<(Comment, User)> = comments
                .inner_join(users::table)
                .filter(post_id.eq(_post_id))
                .offset(filter.page.offset())
                .limit(filter.page.per)
                .filter(parent_id.eq(filter._parent_id))
                .load::<(Comment, User)>(conn)?;

            CommentHierarchy::load_hierarchy(&_comments, filter._parent_id)
        }
    };

    Ok(_comments)
}
