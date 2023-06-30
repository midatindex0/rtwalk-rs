use actix_web::{get, route, web, Responder};
use actix_web_lab::respond::Html;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

use crate::gql::root::Schema;

#[route("/gql", method = "GET", method = "POST")]
pub async fn gql_handler(
    schema: web::Data<Schema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let req = req.into_inner();
    schema.execute(req).await.into()
}

#[get("/graphiql")]
async fn gql_playground_handler() -> impl Responder {
    Html(playground_source(
        GraphQLPlaygroundConfig::new("/gql").subscription_endpoint("/gql"),
    ))
}
