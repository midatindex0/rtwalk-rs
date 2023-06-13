pub mod user;

use async_graphql::{Context, Object, Result};

use crate::info::VersionInfo;

pub struct Query;

#[Object]
impl Query {
    async fn version<'ctx>(&self, ctx: &Context<'ctx>) -> Result<&'ctx VersionInfo> {
        ctx.data::<VersionInfo>()
    }
}
