use actix_session::Session;
use actix_web::{get, route, web, Responder};
use actix_web_lab::respond::Html;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

use crate::{
    auth::{SharedSession},
    gql::root::Schema,
};

#[route("/gql", method = "GET", method = "POST")]
pub async fn gql_handler(
    schema: web::Data<Schema>,
    req: GraphQLRequest,
    session: Session,
) -> GraphQLResponse {
    let shared_sesiion = SharedSession::new(session);
    let req = req.into_inner().data(shared_sesiion);

    schema.execute(req).await.into()
}

#[get("/graphiql")]
async fn gql_playground_handler() -> impl Responder {
    Html(playground_source(
        GraphQLPlaygroundConfig::new("/gql").with_setting("request.credentials", "same-origin"),
    ))
}
