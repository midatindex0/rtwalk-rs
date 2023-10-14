use actix::Addr;
use actix_session::Session;
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::time::Instant;

use crate::{
    constants::UNAUTHEMTICATED_MESSAGE,
    core::{packet::ActiveUser, session::RtSession, RtServer},
    gql::query::{post::get_post_by_slug, user::get_user_by_id},
};

#[get("/connect/{post_slug}")]
pub async fn connect(
    req: HttpRequest,
    session: Session,
    stream: web::Payload,
    path: web::Path<(String,)>,
    pool: web::Data<crate::PgPool>,
    rt_server: web::Data<Addr<RtServer>>,
) -> Result<HttpResponse, Error> {
    let (post_slug,) = path.into_inner();
    log::info!("Connected to WS: {}", &post_slug);
    let id = uuid::Uuid::new_v4().to_string();

    if let Some(user_id) = session.get::<i32>("id")? {
        let user = get_user_by_id(user_id, &pool).await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

        let post = get_post_by_slug(&post_slug, &pool).await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

        ws::start(
            RtSession {
                id,
                hb: Instant::now(),
                post_id: post.post.id,
                forum_id: post.post.forum_id,
                user: ActiveUser {
                    id: user.user.id,
                    username: user.user.username,
                    display_name: user.user.display_name,
                    bio: user.user.bio,
                    pfp: user.user.pfp,
                    banner: user.user.banner,
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
