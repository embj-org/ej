//! Secure password hashing and verification using Argon2.
//!
//! This module provides secure password hashing functionality using the Argon2
//! algorithm, which is the recommended approach for password storage. It includes
//! both password hashing and verification functions with secure defaults.
//!
//! # Usage
//!
//! The module provides two main functions:
//! - [`generate_secret_hash`]: Create secure password hashes
//! - [`is_secret_valid`]: Verify passwords against stored hashes
//!
//! # Examples
//!
//! ```rust
//! use ej_auth::secret_hash::{generate_secret_hash, is_secret_valid};
//!
//! // Hash a user's password
//! let password = "user_password_123";
//! let hash = generate_secret_hash(password).unwrap();
//!
//! // Store the hash in your database
//! // database.store_user_hash(&hash);
//!
//! // Later, verify a login attempt
//! let login_password = "user_password_123";
//! let is_valid = is_secret_valid(login_password, &hash).unwrap();
//! assert!(is_valid);
//!
//! // Wrong password fails verification
//! let wrong_password = "wrong_password";
//! let is_valid = is_secret_valid(wrong_password, &hash).unwrap();
//! assert!(!is_valid);
//! ```

use argon2::{
    Argon2, PasswordHasher, PasswordVerifier,
    password_hash::{self, PasswordHashString, SaltString},
};
use rand::rngs::OsRng;

use crate::prelude::*;

/// Generates a secure hash for the provided password.
///
/// This function creates a cryptographically secure hash of the password using
/// the Argon2 algorithm with a randomly generated salt. The resulting hash is
/// safe to store in databases and includes all necessary parameters for verification.
///
/// # Arguments
///
/// * `pw` - The plaintext password to hash
///
/// # Returns
///
/// * `Ok(String)` - Secure hash ready for storage
/// * `Err(Error)` - Password hashing errors
///
/// # Example
///
/// ```rust
/// use ej_auth::secret_hash::generate_secret_hash;
///
/// let password = "my_secure_password";
/// let hash = generate_secret_hash(password).unwrap();
/// println!("Secure hash: {}", hash);
/// ```
pub fn generate_secret_hash(pw: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    Ok(argon2.hash_password(pw.as_bytes(), &salt)?.to_string())
}

/// Verifies a password against a stored hash.
///
/// This function performs constant-time verification of a password against
/// a previously generated hash. It extracts the salt and parameters from
/// the hash string and re-computes the hash for comparison.
///
/// # Arguments
///
/// * `pw` - The plaintext password to verify
/// * `hash` - The stored hash string to verify against
///
/// # Returns
///
/// * `Ok(true)` - Password matches the hash
/// * `Ok(false)` - Password does not match the hash
/// * `Err(Error)` - Hash parsing or verification errors
///
/// # Example
///
/// ```rust
/// use ej_auth::secret_hash::{generate_secret_hash, is_secret_valid};
///
/// let password = "user_password";
/// let hash = generate_secret_hash(password).unwrap();
/// let is_valid = is_secret_valid(password, &hash).unwrap();
/// assert!(is_valid);
/// ```
pub fn is_secret_valid(pw: &str, hash: &str) -> Result<bool> {
    let hash = PasswordHashString::new(hash)?;

    Ok(Argon2::default()
        .verify_password(pw.as_bytes(), &hash.password_hash())
        .is_ok())
}

impl From<password_hash::Error> for Error {
    fn from(value: password_hash::Error) -> Self {
        Self::PasswordHash(value)
    }
}
