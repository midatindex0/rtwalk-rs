use std::collections::HashSet;

use actix::{Actor, Context, Handler, Message, Recipient};
use async_graphql::{Enum, SimpleObject};

use crate::db::models::{comment::Comment, forum::Forum, post::Post, user::User};

#[derive(Default)]
pub struct EventManager {
    user_event_listeners: HashSet<Recipient<UserEvent>>,
    forum_event_listeners: HashSet<Recipient<ForumEvent>>,
    post_event_listeners: HashSet<Recipient<PostEvent>>,
    comment_event_listeners: HashSet<Recipient<CommentEvent>>,
}

impl Actor for EventManager {
    type Context = Context<Self>;
}

impl Handler<Com> for EventManager {
    type Result = ();

    fn handle(&mut self, msg: Com, _: &mut Self::Context) -> Self::Result {
        match msg {
            Com::SubUser(r) => self.user_event_listeners.insert(r),
            Com::SubForum(r) => self.forum_event_listeners.insert(r),
            Com::SubPost(r) => self.post_event_listeners.insert(r),
            Com::SubComment(r) => self.comment_event_listeners.insert(r),
            Com::UnsubUser(r) => self.user_event_listeners.remove(&r),
            Com::UnsubForum(r) => self.forum_event_listeners.remove(&r),
            Com::UnsubPost(r) => self.post_event_listeners.remove(&r),
            Com::UnsubComment(r) => self.comment_event_listeners.remove(&r),
        };
    }
}

impl Handler<UserEvent> for EventManager {
    type Result = ();

    fn handle(&mut self, msg: UserEvent, _: &mut Self::Context) -> Self::Result {
        for listener in &self.user_event_listeners {
            listener.do_send(msg.clone());
        }
    }
}

impl Handler<ForumEvent> for EventManager {
    type Result = ();

    fn handle(&mut self, msg: ForumEvent, _: &mut Self::Context) -> Self::Result {
        for listener in &self.forum_event_listeners {
            listener.do_send(msg.clone());
        }
    }
}

impl Handler<PostEvent> for EventManager {
    type Result = ();

    fn handle(&mut self, msg: PostEvent, _: &mut Self::Context) -> Self::Result {
        for listener in &self.post_event_listeners {
            listener.do_send(msg.clone());
        }
    }
}

impl Handler<CommentEvent> for EventManager {
    type Result = ();

    fn handle(&mut self, msg: CommentEvent, _: &mut Self::Context) -> Self::Result {
        for listener in &self.comment_event_listeners {
            listener.do_send(msg.clone());
        }
    }
}

#[derive(Clone, SimpleObject, Debug, Message)]
#[rtype(result = "()")]
pub struct UserEvent {
    pub ty: UserEventTy,
    pub user: User,
}

#[derive(Clone, SimpleObject, Debug, Message)]
#[rtype(result = "()")]
pub struct ForumEvent {
    pub ty: ForumEventTy,
    pub forum: Forum,
}

#[derive(Clone, SimpleObject, Debug, Message)]
#[rtype(result = "()")]
pub struct PostEvent {
    pub ty: PostEventTy,
    pub post: Post,
}

#[derive(Clone, SimpleObject, Debug, Message)]
#[rtype(result = "()")]
pub struct CommentEvent {
    pub ty: CommentEventTy,
    pub comment: Comment,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum UserEventTy {
    UserCreation,
    UserBasicUpdate,
}
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum ForumEventTy {
    ForumCreation,
    ForumBasicUpdate,
}
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum PostEventTy {
    PostCreation,
    PostBasicUpdate,
}
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum CommentEventTy {
    CommentCreation,
    CommentBasicUpdate,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub enum Com {
    SubUser(Recipient<UserEvent>),
    SubForum(Recipient<ForumEvent>),
    SubPost(Recipient<PostEvent>),
    SubComment(Recipient<CommentEvent>),
    UnsubUser(Recipient<UserEvent>),
    UnsubForum(Recipient<ForumEvent>),
    UnsubPost(Recipient<PostEvent>),
    UnsubComment(Recipient<CommentEvent>),
}
