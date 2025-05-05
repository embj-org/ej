use super::db_test_context::DBTestContext;

pub struct TestContext {}

impl TestContext {
    pub fn from_env() -> (DBTestContext, reqwest::Client) {
        (DBTestContext::from_env(), reqwest::Client::new())
    }
}
