pub use async_graphql::{EmptyMutation, EmptySubscription, Schema as GqlSchema};

pub use crate::gql::mutations::Mutation;
pub use crate::gql::query::Query;

pub type Schema = GqlSchema<Query, Mutation, EmptySubscription>;
