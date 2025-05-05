use api_client::ApiClient;

pub mod api_client;
pub mod db_test_context;
pub mod test_context;

pub static EJD: ApiClient = ApiClient {
    url: "http://localhost:3000/v1",
};

pub fn from_env(var: &str) -> String {
    std::env::var(var).expect(&format!("Env Variable '{}' missing", var))
}
