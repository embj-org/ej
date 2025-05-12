use serde::{Deserialize, Serialize};

use super::CONNECTION_TOKEN_TYPE;

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthBody {
    pub access_token: String,
    pub token_type: String,
}
impl AuthBody {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: String::from(CONNECTION_TOKEN_TYPE),
        }
    }
}
