#![allow(dead_code)]

mod db;
mod gql;
mod handlers;
pub mod helpers;
mod info;
pub mod schema;

use actix_web::{middleware, web, App, HttpServer};
use argon2::Argon2;
use dotenvy::dotenv;
use env_logger;
use std::env;

use self::db::pool;
use self::gql::root::{EmptyMutation, EmptySubscription, Query, Schema};
use self::handlers::gql::{gql_handler, gql_playground_handler};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    logging_setup();

    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = pool::get_pool(&db_url).expect("Could not create database pool");
    let hasher = Argon2::default();
    let version = info::VersionInfo {
        major: 0,
        minor: 1,
        bug_fix: 1,
        version_string: "0.1.0 alpha",
    };
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(pool.clone())
        .data(version)
        .finish();

    log::info!("Starting http setver at http://127.0.0.1:8000/");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(hasher.clone()))
            .service(gql_handler)
            .service(gql_playground_handler)
            .wrap(middleware::Logger::default())
    })
    .bind("127.0.0.1:8000")
    .unwrap()
    .run()
    .await
}

fn logging_setup() {
    env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
}
