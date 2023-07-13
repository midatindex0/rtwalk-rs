# RtWalk (Real Time Walk)

**RtWalk** is a real-time forum. WIP.

## Directory Structure

```
├── auth.rs - Session Authentication
├── constants.rs - UNAUTHEMTICATED_MESSAGE, RESERVED_USERNAMES, CDN_PATH, ALLOWED_USERNAME_CHARS
├── core
│   ├── event.rs - gql subscription event manager
│   ├── event_session.rs - event manager sessions
│   ├── mod.rs - RtServer implementation
│   ├── packet.rs - Com types for rtserver
│   └── session.rs - RtSession
├── db
│   ├── models
│   │   ├── comment.rs - db, gql and search models
│   │   ├── file.rs - file model
│   │   ├── forum.rs - db, gql and search models
│   │   ├── mod.rs
│   │   ├── post.rs - db, gql and search models
│   │   └── user.rs - db, gql and search models
│   ├── mod.rs
│   └── pool.rs - pgpool
├── error
│   └── mod.rs - some mutation errors, not much used across: TODO
├── gql
│   ├── mod.rs
│   ├── mutation
│   │   ├── comment.rs - create and edit
│   │   ├── forum.rs - create and edit
│   │   ├── mod.rs - actual endpoints, emmits events
│   │   ├── post.rs - create and edit
│   │   └── user.rs - create and edit
│   ├── query
│   │   ├── comment.rs - multiget by criteria, filter and order
│   │   ├── forum.rs - multiget by criteria, filter and order
│   │   ├── mod.rs - actual endpoints
│   │   ├── post.rs - multiget by criteria, filter and order
│   │   └── user.rs - multiget by criteria, filter and order
│   ├── root.rs
│   └── subscription
│       └── mod.rs - event subscriptions
├── handlers
│   ├── gql.rs - post, get and subscription endpoints
│   ├── mod.rs
│   └── ws.rs - ws comment endpoint
├── helpers
│   └── mod.rs - verify username, password
├── info.rs - version
├── main.rs
├── schema.rs - diesel schema
└── search
    └── mod.rs - search index methods
```