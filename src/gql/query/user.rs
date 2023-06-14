use diesel::prelude::*;

use crate::db::models::user::User;
use crate::schema::users::dsl::*;

type Conn = r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

pub fn get_user_by_username(pk: &str, conn: &mut Conn) -> anyhow::Result<User> {
    let s = users.find(pk).first::<User>(conn);
    Ok(s?)
}
