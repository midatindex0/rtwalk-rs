use std::time::{Duration, Instant};

use actix::prelude::*;
use actix::{Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Handler, WrapFuture};
use actix_web_actors::ws::{self, WebsocketContext};

use super::packet::{InComment, InPacket, ListActiveUsers};
use super::{
    packet::{
        ActiveUser, Connect, ConnectNotification, Disconnect, DisconnectNotification, OutPacket,
    },
    RtServer,
};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const SESSION_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct RtSession {
    pub id: String,
    pub hb: Instant,
    pub post_id: i32,
    pub user: ActiveUser,
    pub addr: Addr<RtServer>,
}

impl RtSession {
    fn hb(&self, ctx: &mut WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > SESSION_TIMEOUT {
                act.addr.do_send(Disconnect {
                    id: act.id.clone(),
                    post_id: act.post_id,
                    notif: DisconnectNotification { id: act.user.id },
                });
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for RtSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let addr = ctx.address();

        self.addr
            .send(Connect {
                id: self.id.clone(),
                post_id: self.post_id,
                addr: addr.recipient(),
                notif: ConnectNotification {
                    user: self.user.clone(),
                },
            })
            .into_actor(self)
            .then(|res, _, ctx| {
                match res {
                    Ok(_) => {}
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.addr.do_send(Disconnect {
            id: self.id.clone(),
            post_id: self.post_id,
            notif: DisconnectNotification { id: self.user.id },
        });
        Running::Stop
    }
}

impl Handler<OutPacket> for RtSession {
    type Result = Option<ActiveUser>;

    fn handle(&mut self, msg: OutPacket, ctx: &mut Self::Context) -> Self::Result {
        let msg = match msg {
            OutPacket::Identify => {
                return Some(self.user.clone());
            }
            x => x,
        };
        let msg = serde_json::to_string(&msg);
        match msg {
            Ok(s) => ctx.text(s),
            _ => {
                log::error!("Failed to serialize: {:?}", msg);
                ctx.stop();
            }
        };
        None
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for RtSession {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let item = match item {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(item) => item,
        };
        match item {
            ws::Message::Text(msg) => {
                let msg = msg.trim();

                let msg = serde_json::from_str::<InPacket>(msg);
                match msg {
                    Ok(s) => match s {
                        InPacket::Message {
                            parent_id,
                            content,
                            media,
                        } => {
                            self.addr.do_send(InComment {
                                user: self.user.clone(),
                                post_id: self.post_id,
                                parent_id,
                                content,
                                media,
                            });
                        }
                        InPacket::ListActiveUsers => self
                            .addr
                            .send(ListActiveUsers {
                                post_id: self.post_id,
                            })
                            .into_actor(self)
                            .then(|res, act, ctx| {
                                match res {
                                    Ok(users) => actix::Handler::handle(
                                        act,
                                        OutPacket::ActiveUserList(users),
                                        ctx,
                                    ),
                                    Err(e) => {
                                        log::error!("{e:?}");
                                        None
                                    }
                                };
                                fut::ready(())
                            })
                            .wait(ctx),
                    },
                    Err(_) => {
                        ctx.stop();
                        return;
                    }
                }
            }
            ws::Message::Ping(x) => {
                self.hb = Instant::now();
                ctx.pong(&x);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}
