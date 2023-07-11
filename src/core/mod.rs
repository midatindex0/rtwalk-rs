use actix::prelude::*;
use std::{
    collections::{HashMap, HashSet},
};

use self::packet::{
    Connect, ConnectNotification, Disconnect, DisconnectNotification, InComment, OutComment,
    OutPacket,
};

pub mod packet;
pub mod session;

pub struct RtServer {
    active_broadcasts: HashMap<String, Recipient<OutPacket>>,
    broadcasting_posts: HashMap<i32, HashSet<String>>,
}

impl Default for RtServer {
    fn default() -> Self {
        Self {
            active_broadcasts: HashMap::new(),
            broadcasting_posts: HashMap::new(),
        }
    }
}

impl RtServer {
    fn realy(&self, post_id: i32, inc: InComment) {
        if let Some(listners) = self.broadcasting_posts.get(&post_id) {
            for listner in listners {
                if let Some(addr) = self.active_broadcasts.get(listner) {
                    addr.do_send(OutPacket::OutComment(OutComment {
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
