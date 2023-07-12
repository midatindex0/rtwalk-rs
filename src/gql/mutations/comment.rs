use diesel::{insert_into, RunQueryDsl};

use crate::db::models::comment::Comment;
use crate::db::models::{comment::NewComment, File};
use crate::schema::comments::dsl::*;

pub fn create_comment(
    _user_id: i32,
    _post_id: i32,
    _parent_id: Option<i32>,
    _content: String,
    _media: Option<Vec<Option<File>>>,
    conn: &mut crate::Conn,
) -> anyhow::Result<Comment> {
    let new_comment = NewComment {
        user_id: _user_id,
        post_id: _post_id,
        parent_id: _parent_id,
        content: _content,
        media: _media,
    };
    let x = insert_into(comments)
        .values(&new_comment)
        .get_result::<Comment>(conn)?;
    Ok(x)
}
