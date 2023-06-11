#![allow(dead_code)]

mod db;

use dotenvy::dotenv;
use std::env;

use crate::db::pool;

fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL")?;
    let pool = pool::get_pool(&db_url)?;
    dbg!(pool);
    Ok(())
}
