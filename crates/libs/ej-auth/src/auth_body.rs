//! Authentication response structures.
//!
//! Standard response format for authentication tokens.

use serde::{Deserialize, Serialize};

use super::CONNECTION_TOKEN_TYPE;

/// Authentication response with access token.
///
/// Contains an access token and token type for HTTP authentication.
///
/// # JSON Format
///
/// ```json
/// {
///   "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
///   "token_type": "Bearer"
/// }
/// ```
#[derive(Debug, Deserialize, Serialize)]
pub struct AuthBody {
    /// The access token.
    pub access_token: String,
    /// The token type (always "Bearer").
    pub token_type: String,
}
impl AuthBody {
    /// Creates a new authentication response.
    ///
    /// # Arguments
    ///
    /// * `access_token` - The authentication token
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ej_auth::auth_body::AuthBody;
    ///
    /// let response = AuthBody::new("some_token".to_string());
    /// assert_eq!(response.token_type, "Bearer");
    /// ```
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: String::from(CONNECTION_TOKEN_TYPE),
        }
    }
}
