// @generated automatically by Diesel CLI.

diesel::table! {
    ejclient (id) {
        id -> Uuid,
        name -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
