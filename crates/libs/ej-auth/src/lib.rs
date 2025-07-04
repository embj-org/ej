//! Authentication utilities for the EJ framework.
//!
//! Provides JWT tokens, password hashing, and content verification for EJ services.
//!
//! # Features
//!
//! - **JWT Tokens**: Create and validate JSON Web Tokens
//! - **Password Hashing**: Secure Argon2-based password storage
//! - **SHA-256**: Content hashing for integrity checks
//! - **Auth Responses**: Standard Bearer token responses
//!
//! # Components
//!
//! ## JWT ([`jwt`])
//!
//! Create and validate JWT tokens for service authentication.
//!
//! ## Passwords ([`secret_hash`])
//!
//! Hash and verify passwords using Argon2.
//!
//! ## Hashing ([`sha256`])
//!
//! SHA-256 hashing for content integrity.
//!
//! ## Responses ([`auth_body`])
//!
//! Standard authentication response structures.
//!
//! # Examples
//!
//! ## JWT Tokens
//!
//! ```rust
//! use ej_auth::jwt::{jwt_encode, jwt_decode};
//! use serde::{Serialize, Deserialize};
//! use std::env;
//! unsafe { env::set_var("JWT_SECRET", "MySuperSecret"); }
//!
//! #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
//! struct Claims {
//!     sub: String,
//!     exp: usize,
//! }
//!
//! let claims = Claims {
//!     sub: "user-123".to_string(),
//!     exp: 4118335200,
//! };
//!
//! let token = jwt_encode(&claims).unwrap();
//! let decoded = jwt_decode::<Claims>(&token).unwrap();
//! assert_eq!(claims, decoded.claims);
//! ```
//!
//! ## Password Hashing
//!
//! ```rust
//! use ej_auth::secret_hash::{generate_secret_hash, is_secret_valid};
//!
//! let password = "my_password";
//! let hash = generate_secret_hash(password).unwrap();
//! let is_valid = is_secret_valid(password, &hash).unwrap();
//! assert!(is_valid);
//! ```
//!
//! ## Content Hashing
//!
//! ```rust
//! use ej_auth::sha256::generate_hash;
//!
//! let content = "some data";
//! let hash = generate_hash(content);
//! assert_eq!(hash.len(), 64);
//! ```
//!
//! # Security Notes
//!
//! - Keep JWT secrets secure and rotate regularly
//! - Set appropriate token expiration times
//! - Store passwords as hashes only
//! - Use HTTPS for authentication
//!
//! # Configuration
//!
//! Set `JWT_SECRET` environment variable for token signing.

pub mod auth_body;
pub mod error;
pub mod jwt;
pub mod prelude;
pub mod secret_hash;
pub mod sha256;

/// JWT issuer identifier.
pub const ISS: &str = "EJ";

/// HTTP Authorization header name.
pub const AUTH_HEADER: &str = "Authorization";

/// Bearer token prefix.
pub const AUTH_HEADER_PREFIX: &str = "Bearer ";

/// Token type for connection tokens.
pub const CONNECTION_TOKEN_TYPE: &str = "Bearer";
