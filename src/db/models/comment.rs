use super::File;
use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use diesel::{prelude::*, pg::Pg};

use crate::schema::comments;

#[derive(Clone, Queryable, Selectable, Debug)]
#[diesel(belongs_to(User, foreign_key=user_id))]
#[diesel(belongs_to(Post, foreign_key=post_id))]
#[diesel(belongs_to(Comment, foreign_key=parent_id))]
#[diesel(check_for_backend(Pg))]
pub struct Comment {
    pub id: i32,
    pub user_id: i32,
    pub post_id: i32,
    pub parent_id: Option<i32>,
    pub content: String,
    pub media: Option<Vec<Option<File>>>,
    pub created_at: NaiveDateTime,
    pub edited: bool,
    pub edited_at: Option<NaiveDateTime>,
}

#[derive(SimpleObject)]
pub struct CommentHierarchy {
    pub id: i32,
    pub user_id: i32,
    pub post_id: i32,
    pub parent_id: Option<i32>,
    pub content: String,
    pub media: Option<Vec<Option<File>>>,
    pub created_at: NaiveDateTime,
    pub edited: bool,
    pub edited_at: Option<NaiveDateTime>,
    pub child_comments: Option<Vec<CommentHierarchy>>,
}

impl CommentHierarchy {
    pub fn load_hierarchy(comments: &Vec<Comment>, parent_id: Option<i32>) -> Vec<CommentHierarchy> {
        comments
            .iter()
            .filter(|comment| comment.parent_id == parent_id)
            .map(|comment| CommentHierarchy {
                id: comment.id,
                user_id: comment.user_id,
                post_id: comment.post_id,
                parent_id: comment.parent_id,
                content: comment.content.clone(),
                media: comment.media.clone(),
                created_at: comment.created_at,
                edited: comment.edited,
                edited_at: comment.edited_at,
                child_comments: Some(CommentHierarchy::load_hierarchy(comments, Some(comment.id))),
            })
            .collect()
    }
}
