pub use async_graphql::{EmptyMutation, Schema as GqlSchema};

pub use super::mutation::Mutation;
pub use super::query::Query;
pub use super::subscription::Subscription;

pub type Schema = GqlSchema<Query, Mutation, Subscription>;
