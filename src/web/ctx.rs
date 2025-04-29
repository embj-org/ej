use std::collections::HashSet;

use crate::{ej_client::EjClient, prelude::*};
use axum::{
    body::Body,
    extract::{FromRequestParts, Request},
    http::{HeaderMap, request::Parts},
    middleware::Next,
    response::Response,
};
use tower_cookies::{Cookie, Cookies};

use super::auth::{AuthError, decode_token};

#[derive(Clone, Debug)]
pub struct Ctx {
    pub client: EjClient,
    pub permissions: HashSet<String>,
}

const AUTH_TOKEN_COOKIE: &str = "auth-token";
const AUTH_HEADER: &str = "Authorization";
const AUTH_HEADER_PREFIX: &str = "Bearer ";

impl Ctx {
    pub fn new(client: EjClient) -> Self {
        Self {
            client,
            permissions: HashSet::new(),
        }
    }
}

#[axum::debug_middleware]
pub async fn mw_ctx_resolver(
    cookies: Cookies,
    headers: HeaderMap,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let token = cookies
        .get(AUTH_TOKEN_COOKIE)
        .map(|c| c.value().to_string())
        .or_else(|| {
            headers
                .get(AUTH_HEADER)
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.strip_prefix(AUTH_HEADER_PREFIX))
                .map(|s| s.to_string())
        })
        .ok_or(AuthError::TokenMissing)
        .and_then(|token| decode_token(&token))
        .and_then(|token| {
            if token.exp < chrono::Utc::now().timestamp() {
                Err(AuthError::TokenExpired)
            } else {
                Ok(token)
            }
        });

    let ctx = token.map(|token| Ctx::new(token.sub));

    if ctx.is_err() {
        cookies.remove(Cookie::from(AUTH_TOKEN_COOKIE));
    }
    req.extensions_mut().insert(ctx);

    next.run(req).await
}

impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        Ok(parts
            .extensions
            .get::<std::result::Result<Ctx, AuthError>>()
            .ok_or(Error::CtxMissing)?
            .clone()?)
    }
}
