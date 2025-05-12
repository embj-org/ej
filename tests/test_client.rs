use std::error::Error;

use common::{EJD, test_context::TestContext};
use diesel::prelude::*;
use ej::ej_client::api::{EjClientApi, EjClientLogin, EjClientLoginRequest, EjClientPost};
use serial_test::serial;

mod common;
fn setup_database(connection: &mut PgConnection) {
    const QUERIES: [&str; 0] = [];
    for query in QUERIES {
        let query = diesel::sql_query(query);
        query
            .execute(connection)
            .expect("Couldn't insert values to database");
    }
}

#[tokio::test]
#[serial]
async fn test_create_user() -> Result<(), Box<dyn Error>> {
    let (mut db, client) = TestContext::from_env();
    setup_database(&mut db.conn);

    let new_client = EjClientPost {
        name: String::from("My name"),
        secret: String::from("My Secret"),
    };

    let payload = serde_json::to_string(&new_client)?;
    let post_result: EjClientApi = EJD.post(&client, "client", payload).await?;
    assert_eq!(new_client.name, post_result.name);

    let login_body = EjClientLoginRequest {
        name: new_client.name,
        secret: new_client.secret,
    };

    let payload = serde_json::to_string(&login_body)?;
    let _login_result: EjClientLogin = EJD.post(&client, "login", payload).await?;

    Ok(())
}
