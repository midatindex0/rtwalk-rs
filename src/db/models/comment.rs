use super::{user::User, File};
use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use diesel::{pg::Pg, prelude::*};

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
    pub user: User,
    pub child_comments: Option<Vec<CommentHierarchy>>,
}

impl CommentHierarchy {
    pub fn load_hierarchy(
        comments: &Vec<(Comment, User)>,
        parent_id: Option<i32>,
    ) -> Vec<CommentHierarchy> {
        comments
            .iter()
            .filter(|comment| comment.0.parent_id == parent_id)
            .map(|comment| CommentHierarchy {
                id: comment.0.id,
                user_id: comment.0.user_id,
                post_id: comment.0.post_id,
                parent_id: comment.0.parent_id,
                content: comment.0.content.clone(),
                media: comment.0.media.clone(),
                created_at: comment.0.created_at,
                edited: comment.0.edited,
                edited_at: comment.0.edited_at,
                user: comment.1.clone(),
                child_comments: Some(CommentHierarchy::load_hierarchy(comments, Some(comment.0.id))),
            })
            .collect()
    }
}
