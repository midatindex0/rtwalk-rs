#![allow(dead_code)]

mod db;
pub mod schema;

use actix_web::{middleware, web, App, HttpServer};
use dotenvy::dotenv;
use env_logger;
use std::env;

use crate::db::pool;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    logging_setup();

    dotenv().ok();
    let db_url = env::var("DATABASE_URL").unwrap();
    let pool = pool::get_pool(&db_url).unwrap();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
    })
    .bind("127.0.0.1:4000")
    .unwrap()
    .run()
    .await
}

fn logging_setup() {
    env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
}
