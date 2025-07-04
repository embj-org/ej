//! JWT token management for the EJ authentication system.
//!
//! This module provides functions for creating and validating JSON Web Tokens (JWT)
//! used throughout the EJ framework for stateless authentication. It handles token
//! signing, verification, and claim extraction with secure defaults.
//!
//! # Usage
//!
//! The module provides two main functions for JWT operations:
//! - [`jwt_encode`]: Create signed JWT tokens from claim data
//! - [`jwt_decode`]: Validate and extract claims from JWT tokens
//!
//! # Examples
//!
//! ```rust
//! use ej_auth::jwt::{jwt_encode, jwt_decode};
//! use serde::{Serialize, Deserialize};
//! use std::env;
//! unsafe { env::set_var("JWT_SECRET", "MySuperSecret"); }
//!
//! #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
//! struct UserClaims {
//!     user_id: String,
//!     role: String,
//!     exp: usize,
//! }
//!
//! // Create a token
//! let claims = UserClaims {
//!     user_id: "admin".to_string(),
//!     role: "administrator".to_string(),
//!     exp: 4118335200,
//! };
//!
//! let token = jwt_encode(&claims).unwrap();
//!
//! // Validate and decode the token
//! let decoded = jwt_decode::<UserClaims>(&token).unwrap();
//! assert_eq!(claims, decoded.claims);
//! ```

use crate::prelude::*;
use std::sync::LazyLock;

use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode,
};
use serde::{Serialize, de::DeserializeOwned};

/// Lazily initialized cryptographic keys for JWT operations.
///
/// Keys are loaded once from the JWT_SECRET environment variable and reused
/// for all token operations. This provides better performance than recreating
/// keys for each operation while maintaining security.
static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

/// JWT signing algorithm used throughout the EJ framework.
static ALGORITHM: LazyLock<Algorithm> = LazyLock::new(|| Algorithm::HS256);

/// Cryptographic key pair for JWT signing and verification.
struct Keys {
    /// Key used for signing new JWT tokens.
    encoding: EncodingKey,
    /// Key used for verifying existing JWT tokens.
    decoding: DecodingKey,
}

impl Keys {
    /// Creates a new key pair from the provided secret.
    ///
    /// # Arguments
    ///
    /// * `secret` - Raw bytes of the signing secret
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

/// Creates a signed JWT token from the provided claims.
///
/// This function serializes the claims data and creates a signed JWT token
/// using the configured signing algorithm and secret. The resulting token
/// can be used for authentication across EJ services.
///
/// # Arguments
///
/// * `body` - Claims data to encode in the token (must be serializable)
///
/// # Returns
///
/// * `Ok(String)` - Base64-encoded JWT token
/// * `Err(Error)` - Token creation or serialization errors
///
/// # Security Notes
///
/// - Claims are not encrypted, only signed for integrity
/// - Include expiration claims to prevent token replay attacks
/// - Keep payload minimal to reduce token size and attack surface
///
/// # Example
///
/// ```rust
/// use ej_auth::jwt::{jwt_encode, jwt_decode};
/// use serde::{Serialize, Deserialize};
/// use std::env;
/// unsafe { env::set_var("JWT_SECRET", "MySuperSecret"); }
///
/// #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
/// struct BuilderClaims {
///     builder_id: String,
///     exp: usize,
/// }
///
/// let claims = BuilderClaims {
///     builder_id: "builder-001".to_string(),
///     exp: 4118335200,
/// };
///
/// let token = jwt_encode(&claims).unwrap();
/// let token_data = jwt_decode::<BuilderClaims>(&token).unwrap();
/// assert_eq!(claims, token_data.claims);
/// ```
pub fn jwt_encode<T>(body: &T) -> Result<String>
where
    T: Serialize,
{
    let header = Header::new(*ALGORITHM);
    Ok(encode(&header, body, &KEYS.encoding)?)
}

/// Validates and decodes a JWT token to extract claims.
///
/// This function verifies the token signature, validates the structure,
/// and deserializes the claims data. Only tokens signed with the correct
/// secret and matching algorithm will be accepted.
///
/// # Arguments
///
/// * `token` - JWT token string to validate and decode
///
/// # Returns
///
/// * `Ok(TokenData<T>)` - Validated token with extracted claims
/// * `Err(Error)` - Invalid token, signature mismatch, or deserialization errors
///
/// # Validation
///
/// The function performs these validation steps:
/// - Signature verification using the configured secret
/// - Algorithm validation (must match HS256)
/// - Token structure validation
/// - Claims deserialization
///
/// See `jwt_encode` for a code example
/// ```
pub fn jwt_decode<T>(token: &str) -> Result<TokenData<T>>
where
    T: DeserializeOwned,
{
    Ok(decode(token, &KEYS.decoding, &Validation::new(*ALGORITHM))?)
}
