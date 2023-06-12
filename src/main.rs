#![allow(dead_code)]

mod db;
mod gql;
pub mod schema;

use actix_web::{middleware, web, App, HttpServer};
use argon2::Argon2;
use dotenvy::dotenv;
use env_logger;
use std::env;

use crate::db::pool;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    logging_setup();

    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = pool::get_pool(&db_url).expect("Could not create database pool");
    let hasher = Argon2::default();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(hasher.clone()))
            .wrap(middleware::Logger::default())
    })
    .bind("127.0.0.1:4000")
    .unwrap()
    .run()
    .await
}

fn logging_setup() {
    env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();
}
