use diesel::prelude::*;

use crate::db::models::user::User;
use crate::schema::users::dsl::*;

type Conn = r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

pub fn get_user_by_username(uname: &str, conn: &mut Conn) -> anyhow::Result<User> {
    let s = users
        .filter(username.eq(uname))
        .select(User::as_select())
        .get_result(conn);
    Ok(s?)
}
