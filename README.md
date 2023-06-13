# RtWalk (Real Time Walk)

**RtWalk** is a real-time forum. WIP.

## Directory Structure

- `schema.rs` Auto generated schema by diesel
- `main.rs` Contains the actix webserver
- `info.rs` Contains version info and stuff
- `db` Files related to database
    - `models` Models of data in the database
        - `user.rs` The user model.
    - `pool.rs` Database pool
- `gql` Files related to graphql api
    - `mutations` Defines functions required to mutate db records
    - `query` Defines functions required to query db records
        - `user.rs` Functiosn to query the user data
    -  `root.rs` Defines the schema
- `handlers` Defines web route handlets
    - `gql.rs` Graphql handlers
- `helpers` Helper functions/structs