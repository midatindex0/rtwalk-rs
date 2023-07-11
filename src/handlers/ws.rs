use actix::Addr;
use actix_session::Session;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::time::Instant;

use crate::core::{session::RtSession, RtServer};

pub async fn connect(
    req: HttpRequest,
    session: Session,
    stream: web::Payload,
    rt_server: web::Data<Addr<RtServer>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        RtSession {
            id: todo!(),
            hb: Instant::now(),
            post_id: todo!(),
            user: todo!(),
            addr: rt_server.get_ref().clone(),
        },
        &req,
        stream,
    )
}
