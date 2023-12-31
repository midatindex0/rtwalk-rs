use actix_session::Session;
use actix_web::{get, route, web, HttpRequest, HttpResponse, Responder};
use actix_web_lab::respond::Html;
use async_graphql::http::{Credentials, GraphiQLSource};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};

use crate::{auth::SharedSession, gql::root::Schema};

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

pub async fn gql_ws_handler(
    schema: web::Data<Schema>,
    req: HttpRequest,
    payload: web::Payload,
) -> actix_web::Result<HttpResponse> {
    GraphQLSubscription::new(Schema::clone(&*schema)).start(&req, payload)
}

#[get("/graphiql")]
async fn gql_playground_handler() -> impl Responder {
    Html(
        GraphiQLSource::build()
            .endpoint("/gql")
            .subscription_endpoint("/gqlws")
            .credentials(Credentials::Include)
            .finish(),
    )
}
