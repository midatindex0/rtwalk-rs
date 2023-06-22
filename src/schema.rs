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
    users (id) {
        id -> Int4,
        username -> Varchar,
        password -> Varchar,
        display_name -> Varchar,
        bio -> Nullable<Text>,
        pfp -> Nullable<Varchar>,
        created_at -> Timestamp,
    }
}

diesel::joinable!(forums -> users (owner_id));

diesel::allow_tables_to_appear_in_same_query!(
    forums,
    users,
);
