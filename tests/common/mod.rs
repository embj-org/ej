#[allow(dead_code)]
use std::error::Error;

use api_client::ApiClient;
use ej::ej_client::api::{EjClientLogin, EjClientLoginRequest};

pub mod api_client;
pub mod db_test_context;
pub mod test_context;

pub static EJD: ApiClient = ApiClient {
    url: "http://localhost:3000/v1",
};

pub fn from_env(var: &str) -> String {
    std::env::var(var).expect(&format!("Env Variable '{}' missing", var))
}
pub async fn login(
    client: &reqwest::Client,
    login_body: EjClientLoginRequest,
) -> Result<EjClientLogin, Box<dyn Error>> {
    let payload = serde_json::to_string(&login_body)?;
    let login: EjClientLogin = EJD.post(&client, "login", payload).await?;
    Ok(login)
}
