// @generated automatically by Diesel CLI.

diesel::table! {
    users (username) {
        id -> Int4,
        username -> Varchar,
        password -> Varchar,
        display_name -> Varchar,
        bio -> Nullable<Text>,
        created_at -> Timestamp,
    }
}
