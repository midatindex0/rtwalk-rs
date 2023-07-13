# RtWalk (Real Time Walk)

**RtWalk** is a real-time forum. WIP.

## Directory Structure

```
├── auth.rs
├── constants.rs
├── core
│   ├── event.rs
│   ├── event_session.rs
│   ├── mod.rs
│   ├── packet.rs
│   └── session.rs
├── db
│   ├── models
│   │   ├── comment.rs
│   │   ├── file.rs
│   │   ├── forum.rs
│   │   ├── mod.rs
│   │   ├── post.rs
│   │   └── user.rs
│   ├── mod.rs
│   └── pool.rs
├── error
│   └── mod.rs
├── gql
│   ├── mod.rs
│   ├── mutation
│   │   ├── comment.rs
│   │   ├── forum.rs
│   │   ├── mod.rs
│   │   ├── post.rs
│   │   └── user.rs
│   ├── query
│   │   ├── comment.rs
│   │   ├── forum.rs
│   │   ├── mod.rs
│   │   ├── post.rs
│   │   └── user.rs
│   ├── root.rs
│   └── subscription
│       └── mod.rs
├── handlers
│   ├── gql.rs
│   ├── mod.rs
│   └── ws.rs
├── helpers
│   └── mod.rs
├── info.rs
├── main.rs
├── schema.rs
└── search
    └── mod.rs
```