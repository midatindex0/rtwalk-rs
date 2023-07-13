use async_graphql::InputObject;
use diesel::{insert_into, ExpressionMethods, RunQueryDsl};

use crate::db::models::comment::{Comment, UpdateComment};
use crate::db::models::{comment::NewComment, File};
use crate::schema::comments::dsl::*;

#[derive(InputObject)]
pub struct BasicCommentUpdate {
    _id: i32,
    _content: Option<String>,
    _media: Option<Vec<String>>,
}

impl Into<UpdateComment> for BasicCommentUpdate {
    fn into(self) -> UpdateComment {
        UpdateComment {
            id: self._id,
            user_id: None,
            content: self._content.map(|x| {
                if x.is_empty() {
                    "User cleared this comment".to_string()
                } else {
                    x
                }
            }),
            media: self._media.map(|x| {
                if x.is_empty() {
                    None
                } else {
                    Some(x.into_iter().map(|x| Some(File::new(x))).collect())
                }
            }),
            edited: true,
            edited_at: Some(chrono::Utc::now().naive_utc()),
        }
    }
}

pub fn create_comment(
    _user_id: i32,
    _post_id: i32,
    _forum_id: i32,
    _parent_id: Option<i32>,
    _content: String,
    _media: Option<Vec<Option<File>>>,
    conn: &mut crate::Conn,
) -> anyhow::Result<Comment> {
    let new_comment = NewComment {
        user_id: _user_id,
        post_id: _post_id,
        forum_id: _forum_id,
        parent_id: _parent_id,
        content: _content,
        media: _media,
    };

    let x = insert_into(comments)
        .values(&new_comment)
        .get_result::<Comment>(conn)?;

    Ok(x)
}

pub fn update_comment(
    _user_id: i32,
    changes: &UpdateComment,
    conn: &mut crate::Conn,
) -> anyhow::Result<Comment> {
    let x = diesel::update(comments)
        .set(changes)
        .filter(user_id.eq(_user_id))
        .get_result::<Comment>(conn)?;
    Ok(x)
}
