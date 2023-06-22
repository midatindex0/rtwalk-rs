pub mod user;

use async_graphql::{Context, Object, Result};

use crate::{
    db::{models::user::User, pool::PostgresPool},
    info::VersionInfo,
};

pub struct Query;

#[Object]
impl Query {
    async fn version<'c>(&self, ctx: &Context<'c>) -> Result<&'c VersionInfo> {
        ctx.data::<VersionInfo>()
    }

    /// TODO: Error Handling (in responses)
    async fn user<'c>(&self, ctx: &Context<'c>, username: String) -> Result<User> {
        let mut conn = ctx.data::<PostgresPool>()?.get()?;

        let user = actix_rt::task::spawn_blocking(move || {
            user::get_user_by_username(&username, &mut conn)
        })
        .await??;
        Ok(user)
    }
}
