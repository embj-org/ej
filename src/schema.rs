// @generated automatically by Diesel CLI.

diesel::table! {
    client_permission (ejclient_id, permission_id) {
        ejclient_id -> Uuid,
        permission_id -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

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

diesel::table! {
    permission (id) {
        id -> Varchar,
    }
}

diesel::joinable!(client_permission -> ejclient (ejclient_id));
diesel::joinable!(client_permission -> permission (permission_id));

diesel::allow_tables_to_appear_in_same_query!(client_permission, ejclient, permission,);
