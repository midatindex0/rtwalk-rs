// @generated automatically by Diesel CLI.

diesel::table! {
    comments (id) {
        id -> Int4,
        user_id -> Int4,
        post_id -> Int4,
        forum_id -> Int4,
        parent_id -> Nullable<Int4>,
        content -> Text,
        media -> Nullable<Array<Nullable<Text>>>,
        created_at -> Timestamp,
        edited -> Bool,
        edited_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    forums (id) {
        id -> Int4,
        name -> Varchar,
        display_name -> Varchar,
        icon -> Nullable<Varchar>,
        banner -> Nullable<Varchar>,
        description -> Nullable<Text>,
        created_at -> Timestamp,
        owner_id -> Int4,
    }
}

diesel::table! {
    posts (id) {
        id -> Int4,
        tags -> Nullable<Array<Nullable<Text>>>,
        stars -> Int4,
        title -> Varchar,
        slug -> Varchar,
        content -> Nullable<Text>,
        media -> Nullable<Array<Nullable<Text>>>,
        created_at -> Timestamp,
        edited -> Bool,
        edited_at -> Nullable<Timestamp>,
        forum_id -> Int4,
        poster_id -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        password -> Varchar,
        display_name -> Varchar,
        bio -> Nullable<Text>,
        pfp -> Nullable<Varchar>,
        banner -> Nullable<Varchar>,
        created_at -> Timestamp,
        v -> Int4,
        admin -> Bool,
    }
}

diesel::joinable!(comments -> forums (forum_id));
diesel::joinable!(comments -> posts (post_id));
diesel::joinable!(comments -> users (user_id));
diesel::joinable!(forums -> users (owner_id));
diesel::joinable!(posts -> forums (forum_id));
diesel::joinable!(posts -> users (poster_id));

diesel::allow_tables_to_appear_in_same_query!(
    comments,
    forums,
    posts,
    users,
);
