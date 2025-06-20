use argon2::{
    password_hash::{self, PasswordHashString, SaltString},
    Argon2, PasswordHasher, PasswordVerifier,
};
use rand::rngs::OsRng;

use crate::prelude::*;

pub fn generate_secret_hash(pw: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    Ok(argon2.hash_password(pw.as_bytes(), &salt)?.to_string())
}

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
