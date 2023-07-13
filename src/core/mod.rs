use actix::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use crate::{db::pool::PostgresPool, gql::mutation::comment::create_comment};

use self::packet::{
    ActiveUser, Connect, ConnectNotification, Disconnect, DisconnectNotification, InComment,
    ListActiveUsers, OutComment, OutPacket,
};

pub mod packet;
pub mod session;
pub mod event;
pub mod event_session;

pub struct RtServer {
    active_broadcasts: HashMap<String, Recipient<OutPacket>>,
    broadcasting_posts: HashMap<i32, HashSet<String>>,
    pool: PostgresPool,
}

impl RtServer {
    pub fn new(pool: PostgresPool) -> Self {
        Self {
            active_broadcasts: HashMap::new(),
            broadcasting_posts: HashMap::new(),
            pool,
        }
    }
}

impl RtServer {
    fn realy(&self, post_id: i32, inc: InComment) {
        let conn = self.pool.get();
        if let Ok(mut conn) = conn {
            match create_comment(
                inc.user.id,
                inc.post_id,
                inc.parent_id,
                inc.content.clone(),
                inc.media.clone(),
                &mut conn,
            ) {
                Ok(comment) => {
                    if let Some(listners) = self.broadcasting_posts.get(&post_id) {
                        for listner in listners {
                            if let Some(addr) = self.active_broadcasts.get(listner) {
                                addr.do_send(OutPacket::OutComment(OutComment {
                                    id: comment.id,
                                    created_at: comment.created_at,
                                    user: inc.user.clone(),
                                    post_id: inc.post_id,
                                    parent_id: inc.parent_id,
                                    content: inc.content.clone(),
                                    media: inc.media.clone(),
                                }));
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("{e:?}");
                    return;
                }
            }
        }
    }

    fn notify_connect(&self, post_id: i32, notif: ConnectNotification) {
        if let Some(listners) = self.broadcasting_posts.get(&post_id) {
            for listner in listners {
                if let Some(addr) = self.active_broadcasts.get(listner) {
                    addr.do_send(OutPacket::ConnectNotification(notif.clone()));
                }
            }
        }
    }

    fn notify_disconnect(&self, post_id: i32, notif: DisconnectNotification) {
        if let Some(listners) = self.broadcasting_posts.get(&post_id) {
            for listner in listners {
                if let Some(addr) = self.active_broadcasts.get(listner) {
                    addr.do_send(OutPacket::DisconnectNotification(notif.clone()));
                }
            }
        }
    }
}

impl Actor for RtServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for RtServer {
    type Result = ();
    fn handle(&mut self, event: Connect, _: &mut Self::Context) -> Self::Result {
        self.active_broadcasts.insert(event.id.clone(), event.addr);
        self.broadcasting_posts
            .entry(event.post_id)
            .or_insert_with(HashSet::new)
            .insert(event.id);
        self.notify_connect(event.post_id, event.notif);
    }
}

impl Handler<Disconnect> for RtServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
        self.active_broadcasts.remove(&msg.id);
        self.broadcasting_posts.entry(msg.post_id).and_modify(|x| {
            x.remove(&msg.id);
        });
        self.notify_disconnect(msg.post_id, msg.notif);
    }
}

impl Handler<InComment> for RtServer {
    type Result = ();

    fn handle(&mut self, msg: InComment, _: &mut Self::Context) {
        self.realy(msg.post_id, msg);
    }
}

impl Handler<ListActiveUsers> for RtServer {
    type Result = Vec<ActiveUser>;

    fn handle(&mut self, msg: ListActiveUsers, ctx: &mut Self::Context) -> Self::Result {
        if let Some(post) = self.broadcasting_posts.get(&msg.post_id) {
            let identified_users = Arc::new(Mutex::new(vec![]));
            for session in post {
                if let Some(user) = self.active_broadcasts.get(session) {
                    let x = identified_users.clone();
                    user.send(OutPacket::Identify)
                        .into_actor(self)
                        .then(move |res, _, _| {
                            match res {
                                Ok(Some(s)) => {
                                    x.lock().unwrap().push(s);
                                }
                                Ok(None) => {}
                                _ => {
                                    log::error!("{res:?}");
                                }
                            };
                            fut::ready(())
                        })
                        .wait(ctx);
                }
            }
            return Arc::try_unwrap(identified_users)
                .unwrap_or_default()
                .into_inner()
                .unwrap_or_default();
        }
        vec![]
    }
}
