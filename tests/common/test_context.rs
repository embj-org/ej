use reqwest::header;

use super::db_test_context::DBTestContext;

pub struct TestContext {}

impl TestContext {
    pub fn from_env() -> (DBTestContext, reqwest::Client) {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "content-type",
            header::HeaderValue::from_static("application/json"),
        );
        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .cookie_store(true)
            .build()
            .expect("Failed to build reqwest Client");
        (DBTestContext::from_env(), client)
    }
}
