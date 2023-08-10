use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler};

use super::event::{Com, CommentEvent, EventManager, ForumEvent, PostEvent, UserEvent};

pub struct UserEventSession {
    pub sender: futures::channel::mpsc::Sender<UserEvent>,
    pub manager: Addr<EventManager>,
}

impl Actor for UserEventSession {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address().recipient();
        self.manager.do_send(Com::SubUser(addr));
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> actix::Running {
        let addr = ctx.address().recipient();
        self.manager.do_send(Com::UnsubUser(addr));
        actix::Running::Stop
    }
}

impl Handler<UserEvent> for UserEventSession {
    type Result = ();

    fn handle(&mut self, msg: UserEvent, ctx: &mut Self::Context) -> Self::Result {
        match self.sender.try_send(msg) {
            Ok(_) => {}
            Err(_) => ctx.stop(),
        }
    }
}

// -------------------

pub struct ForumEventSession {
    pub sender: futures::channel::mpsc::Sender<ForumEvent>,
    pub manager: Addr<EventManager>,
}

impl Actor for ForumEventSession {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address().recipient();
        self.manager.do_send(Com::SubForum(addr));
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> actix::Running {
        let addr = ctx.address().recipient();
        self.manager.do_send(Com::UnsubForum(addr));
        actix::Running::Stop
    }
}

impl Handler<ForumEvent> for ForumEventSession {
    type Result = ();

    fn handle(&mut self, msg: ForumEvent, ctx: &mut Self::Context) -> Self::Result {
        match self.sender.try_send(msg) {
            Ok(_) => {}
            Err(_) => ctx.stop(),
        }
    }
}

// -------------------------

pub struct PostEventSession {
    pub sender: futures::channel::mpsc::Sender<PostEvent>,
    pub manager: Addr<EventManager>,
}

impl Actor for PostEventSession {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address().recipient();
        self.manager.do_send(Com::SubPost(addr));
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> actix::Running {
        let addr = ctx.address().recipient();
        self.manager.do_send(Com::UnsubPost(addr));
        actix::Running::Stop
    }
}

impl Handler<PostEvent> for PostEventSession {
    type Result = ();

    fn handle(&mut self, msg: PostEvent, ctx: &mut Self::Context) -> Self::Result {
        match self.sender.try_send(msg) {
            Ok(_) => {}
            Err(_) => ctx.stop(),
        }
    }
}
// ------------------------------

pub struct CommentEventSession {
    pub sender: futures::channel::mpsc::Sender<CommentEvent>,
    pub forum_ids: Vec<i32>,
    pub manager: Addr<EventManager>,
}

impl Actor for CommentEventSession {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address().recipient();
        self.manager.do_send(Com::SubComment(addr));
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> actix::Running {
        let addr = ctx.address().recipient();
        self.manager.do_send(Com::UnsubComment(addr));
        actix::Running::Stop
    }
}

impl Handler<CommentEvent> for CommentEventSession {
    type Result = ();

    fn handle(&mut self, msg: CommentEvent, ctx: &mut Self::Context) -> Self::Result {
        if self.forum_ids.contains(&msg.comment.forum_id) {
            match self.sender.try_send(msg) {
                Ok(_) => {}
                Err(_) => ctx.stop(),
            }
        }
    }
}
