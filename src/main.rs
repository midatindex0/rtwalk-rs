pub mod auth;
pub mod constants;
pub mod core;
mod db;
pub mod error;
mod gql;
mod handlers;
pub mod helpers;
mod info;
pub mod schema;
pub mod search;

use actix::*;
use actix_session::{storage::RedisActorSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, middleware, web, App, HttpServer};
use argon2::Argon2;
use dotenvy::dotenv;
use opendal::{
    layers::{LoggingLayer, RetryLayer},
    services::Fs,
    Operator,
};

use std::env;

use crate::{core::RtServer, search::SearchIndex};

use self::db::pool;
use self::gql::root::{EmptySubscription, Mutation, Query, Schema};
use self::handlers::{
    gql::{gql_handler, gql_playground_handler},
    ws::connect,
};

type Conn = r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    logging_setup();

    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL not set");
    let key = Key::from(env::var("AUTH_KEY").expect("AUTH_KEY not set").as_bytes());
    let pool = pool::get_pool(&db_url).expect("Could not create database pool");
    let hasher = Argon2::default();
    let version = info::VersionInfo {
        major: 0,
        minor: 1,
        bug_fix: 1,
        version_string: "0.1.0 alpha",
    };

    let mut fs_builder = Fs::default();
    fs_builder.root("data/").enable_path_check();
    let data = Operator::new(fs_builder)
        .expect("Could not create fs data store")
        .layer(LoggingLayer::default())
        .layer(RetryLayer::new())
        .finish();

    let index = SearchIndex::default();

    let rt_server = RtServer::new(pool.clone()).start();

    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .data(pool.clone())
        .data(hasher.clone())
        .data(data.clone())
        .data(index)
        .data(version)
        .finish();

    log::info!("Running server at http://127.0.0.1:8000/graphiql");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .app_data(web::Data::new(rt_server.clone()))
            .service(gql_handler)
            .service(gql_playground_handler)
            .service(connect)
            .service(
                web::scope("/cdn").service(
                    actix_files::Files::new("/", "data/")
                        .show_files_listing()
                        .use_last_modified(true),
                ),
            )
            .wrap(SessionMiddleware::new(
                RedisActorSessionStore::new(redis_url.clone()),
                key.clone(),
            ))
            .wrap(middleware::Logger::default())
    })
    .bind("127.0.0.1:8000")
    .unwrap()
    .run()
    .await
}

fn logging_setup() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
}
