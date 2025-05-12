use std::error::Error;

use common::{EJD, login, test_context::TestContext};
use diesel::prelude::*;
use ej::{ej_builder::api::EjBuilderApi, ej_client::api::EjClientLoginRequest};
use serial_test::serial;

mod common;
fn setup_database(connection: &mut PgConnection) {
    const QUERIES: [&str; 3] = [
        "INSERT INTO permission (id) VALUES ('builder.create');",
        "INSERT INTO ejclient (name, hash) VALUES ('root', '$argon2id$v=19$m=19456,t=2,p=1$C4ZIwZW3k3Lec4ml/LlXhg$o5GwzCUQKicsKHfJvLAyL2GSyx9topN12vgZA4avM+g');",
        "INSERT INTO client_permission (ejclient_id, permission_id) VALUES ((SELECT id FROM ejclient WHERE name = 'root'), 'builder.create')",
    ];
    for query in QUERIES {
        let query = diesel::sql_query(query);
        query
            .execute(connection)
            .expect("Couldn't insert values to database");
    }
}

#[tokio::test]
#[serial]
async fn test_create_builder() -> Result<(), Box<dyn Error>> {
    let (mut db, client) = TestContext::from_env();
    setup_database(&mut db.conn);
    login(
        &client,
        EjClientLoginRequest::new("root", "my_super_secret"),
    )
    .await?;

    let _builder: EjBuilderApi = EJD.post_no_body(&client, "client/builder").await?;

    Ok(())
}
