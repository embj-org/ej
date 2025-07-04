use crate::prelude::*;
use std::sync::LazyLock;

use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode,
};
use serde::{Serialize, de::DeserializeOwned};

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

static ALGORITHM: LazyLock<Algorithm> = LazyLock::new(|| Algorithm::HS256);

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

pub fn jwt_encode<T>(body: &T) -> Result<String>
where
    T: Serialize,
{
    let header = Header::new(*ALGORITHM);
    Ok(encode(&header, body, &KEYS.encoding)?)
}

pub fn jwt_decode<T>(token: &str) -> Result<TokenData<T>>
where
    T: DeserializeOwned,
{
    Ok(decode(token, &KEYS.decoding, &Validation::new(*ALGORITHM))?)
}
