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
    ejboard (id) {
        id -> Uuid,
        ejconfig_id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        description -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    ejboard_config (id) {
        id -> Uuid,
        ejboard_id -> Uuid,
        description -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    ejboard_config_tag (ejboard_config_id, ejtag_id) {
        ejboard_config_id -> Uuid,
        ejtag_id -> Uuid,
    }
}

diesel::table! {
    ejbuilder (id) {
        id -> Uuid,
        ejclient_id -> Uuid,
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
    ejconfig (id) {
        id -> Uuid,
        ejclient_id -> Uuid,
        #[max_length = 50]
        version -> Varchar,
        #[max_length = 255]
        hash -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    ejtag (id) {
        id -> Uuid,
        #[max_length = 100]
        name -> Varchar,
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
diesel::joinable!(ejboard -> ejconfig (ejconfig_id));
diesel::joinable!(ejboard_config -> ejboard (ejboard_id));
diesel::joinable!(ejboard_config_tag -> ejboard_config (ejboard_config_id));
diesel::joinable!(ejboard_config_tag -> ejtag (ejtag_id));
diesel::joinable!(ejbuilder -> ejclient (ejclient_id));
diesel::joinable!(ejconfig -> ejclient (ejclient_id));

diesel::allow_tables_to_appear_in_same_query!(
    client_permission,
    ejboard,
    ejboard_config,
    ejboard_config_tag,
    ejbuilder,
    ejclient,
    ejconfig,
    ejtag,
    permission,
);
