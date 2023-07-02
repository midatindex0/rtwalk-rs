// @generated automatically by Diesel CLI.

diesel::table! {
    forums (id) {
        id -> Int4,
        name -> Varchar,
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
        forum -> Int4,
        poster -> Int4,
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
    }
}

diesel::joinable!(forums -> users (owner_id));
diesel::joinable!(posts -> forums (forum));
diesel::joinable!(posts -> users (poster));

diesel::allow_tables_to_appear_in_same_query!(
    forums,
    posts,
    users,
);
