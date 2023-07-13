use actix::Addr;
use actix_session::Session;
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::time::Instant;

use crate::{
    constants::UNAUTHEMTICATED_MESSAGE,
    core::{packet::ActiveUser, session::RtSession, RtServer},
    db::{models::post::Post, pool::PostgresPool},
    gql::query::{post::get_post_by_id, user::get_user_by_username},
    spawn_blocking,
};

#[get("/connect/{post_id}")]
pub async fn connect(
    req: HttpRequest,
    session: Session,
    stream: web::Payload,
    path: web::Path<(i32,)>,
    pool: web::Data<PostgresPool>,
    rt_server: web::Data<Addr<RtServer>>,
) -> Result<HttpResponse, Error> {
    let (post_id,) = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    let mut conn1 = pool
        .get()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    let id = uuid::Uuid::new_v4().to_string();

    if let Some(username) = session.get::<String>("username")? {
        let user = spawn_blocking!(get_user_by_username(&username, &mut conn1))
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

        let post: Post = spawn_blocking!(get_post_by_id(&post_id, &mut conn))
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

        ws::start(
            RtSession {
                id,
                hb: Instant::now(),
                post_id,
                forum_id: post.forum_id,
                user: ActiveUser {
                    id: user.id,
                    username: user.username,
                    display_name: user.display_name,
                    bio: user.bio,
                    pfp: user.pfp,
                    banner: user.banner,
                },
                addr: rt_server.get_ref().clone(),
            },
            &req,
            stream,
        )
    } else {
        Err(actix_web::error::ErrorUnauthorized(UNAUTHEMTICATED_MESSAGE))
    }
}
