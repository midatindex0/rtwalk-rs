use super::File;
use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::schema::{messages};

#[derive(Clone, Queryable, Selectable, Debug)]
#[diesel(belongs_to(User, foreign_key=user_id))]
#[diesel(belongs_to(Post, foreign_key=post_id))]
#[diesel(belongs_to(Message, foreign_key=parent_id))]
struct Message {
    pub id: i32,
    pub user_id: i32,
    pub post_id: i32,
    pub parent_id: Option<i32>,
    pub content: String,
    pub media: Option<Vec<File>>,
    pub created_at: NaiveDateTime,
    pub edited: bool,
    pub edited_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Associations, SimpleObject)]
#[diesel(belongs_to(Message, foreign_key = parent_id))]
#[diesel(table_name = messages)]
pub struct MessageHierarchy {
    pub id: i32,
    pub user_id: i32,
    pub post_id: i32,
    pub parent_id: Option<i32>,
    pub content: String,
    pub media: Option<Vec<File>>,
    pub created_at: NaiveDateTime,
    pub edited: bool,
    pub edited_at: Option<NaiveDateTime>,
    pub child_messages: Option<Vec<MessageHierarchy>>,
}

impl MessageHierarchy {
    fn load_hierarchy(messages: &Vec<Message>, parent_id: Option<i32>) -> Vec<MessageHierarchy> {
        messages
            .iter()
            .filter(|message| message.parent_id == parent_id)
            .map(|message| MessageHierarchy {
                id: message.id,
                user_id: message.user_id,
                post_id: message.post_id,
                parent_id: message.parent_id,
                content: message.content.clone(),
                media: message.media.clone(),
                created_at: message.created_at,
                edited: message.edited,
                edited_at: message.edited_at,
                child_messages: Some(MessageHierarchy::load_hierarchy(messages, Some(message.id))),
            })
            .collect()
    }
}
