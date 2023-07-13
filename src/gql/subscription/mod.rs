use actix::{Actor, Addr};
use async_graphql::*;
use futures_core::Stream;

use crate::core::{
    event::{CommentEvent, EventManager, ForumEvent, PostEvent, UserEvent},
    event_session::{CommentEventSession, ForumEventSession, PostEventSession, UserEventSession},
};

pub struct Subscription;

#[Subscription]
impl Subscription {
    async fn user_events<'c>(&self, ctx: &Context<'c>) -> Result<impl Stream<Item = UserEvent>> {
        let event_manager = ctx.data::<Addr<EventManager>>()?;

        let (tx, rx) = futures::channel::mpsc::channel::<UserEvent>(100);

        UserEventSession {
            sender: tx,
            manager: event_manager.clone(),
        }
        .start();

        Ok(rx)
    }

    async fn forum_events<'c>(&self, ctx: &Context<'c>) -> Result<impl Stream<Item = ForumEvent>> {
        let event_manager = ctx.data::<Addr<EventManager>>()?;

        let (tx, rx) = futures::channel::mpsc::channel::<ForumEvent>(100);

        ForumEventSession {
            sender: tx,
            manager: event_manager.clone(),
        }
        .start();

        Ok(rx)
    }

    async fn post_events<'c>(&self, ctx: &Context<'c>) -> Result<impl Stream<Item = PostEvent>> {
        let event_manager = ctx.data::<Addr<EventManager>>()?;

        let (tx, rx) = futures::channel::mpsc::channel::<PostEvent>(100);

        PostEventSession {
            sender: tx,
            manager: event_manager.clone(),
        }
        .start();

        Ok(rx)
    }

    async fn comment_events<'c>(
        &self,
        ctx: &Context<'c>,
    ) -> Result<impl Stream<Item = CommentEvent>> {
        let event_manager = ctx.data::<Addr<EventManager>>()?;

        let (tx, rx) = futures::channel::mpsc::channel::<CommentEvent>(100);

        CommentEventSession {
            sender: tx,
            manager: event_manager.clone(),
        }
        .start();

        Ok(rx)
    }
}
