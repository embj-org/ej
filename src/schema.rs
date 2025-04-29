// @generated automatically by Diesel CLI.

diesel::table! {
    ejclient (id) {
        id -> Uuid,
        name -> Varchar,
        #[max_length = 255]
        hash -> Varchar,
        hash_version -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
