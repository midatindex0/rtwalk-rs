use anyhow::Result;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use r2d2::Pool;

pub type PostgresPool = Pool<ConnectionManager<PgConnection>>;

// TODO: Better error handling (using crates own types)
pub fn get_pool(url: &str) -> Result<PostgresPool> {
    let connection_manager = ConnectionManager::<PgConnection>::new(url);
    Ok(Pool::builder().build(connection_manager)?)
}
