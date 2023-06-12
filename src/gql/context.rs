use crate::db::pool::PostgresPool;

pub struct GqlCtx {
    pub pool: PostgresPool,
}

/// Marker Trait for gql context (allows us to do stuff)
impl juniper::Context for GqlCtx {}
