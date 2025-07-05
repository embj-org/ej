//! Context resolver for extracting client information from HTTP requests.

use axum::{
    body::Body,
    extract::{FromRequestParts, Request},
    http::{HeaderMap, request::Parts},
    middleware::Next,
    response::Response,
};
use ej_auth::{AUTH_HEADER, AUTH_HEADER_PREFIX, jwt::jwt_decode};
use ej_dispatcher_sdk::{
    ejbuilder::EjBuilderApi,
    ejclient::{EjClientLogin, EjClientLoginRequest},
};
use ej_models::db::connection::DbConnection;
use tower_cookies::{Cookie, Cookies};

use crate::{
    auth_token::AuthToken,
    ctx::{Ctx, ctx_client::generate_token},
};
use crate::{auth_token::authenticate, prelude::*};

/// The name of the cookie used to store authentication tokens.
pub const AUTH_TOKEN_COOKIE: &str = "auth-token";

/// Middleware for resolving request context from authentication tokens.
///
/// Extracts authentication tokens from cookies or headers, validates them,
/// and adds the resulting context to the request extensions.
///
/// # Examples
///
/// ```rust
/// use axum::Router;
/// use ej_web::ctx::resolver::mw_ctx_resolver;
///
/// let app: Router<()> = Router::new()
///     .layer(axum::middleware::from_fn(mw_ctx_resolver));
/// ```
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
        .ok_or(ej_auth::error::Error::TokenMissing)
        .and_then(|token| Ok(jwt_decode::<AuthToken>(&token)?.claims))
        .and_then(|token| {
            if token.exp < chrono::Utc::now().timestamp() {
                Err(ej_auth::error::Error::TokenExpired)
            } else {
                Ok(token)
            }
        });

    let ctx = token.map(|token: AuthToken| Ctx::new(token.sub, token.who, token.permissions));

    if ctx.is_err() {
        cookies.remove(Cookie::from(AUTH_TOKEN_COOKIE));
    }
    req.extensions_mut().insert(ctx);

    next.run(req).await
}

/// Logs in a builder and sets authentication cookie.
///
/// # Examples
///
///
/// ```rust,no_run
/// use ej_web::ctx::resolver::login_builder;
/// use ej_dispatcher_sdk::ejbuilder::EjBuilderApi;
/// use tower_cookies::Cookies;
/// use uuid::Uuid;
///
/// # fn example(cookies: &Cookies) -> Result<(), Box<dyn std::error::Error>> {
/// let builder = EjBuilderApi {
///     id: Uuid::new_v4(),
///     token: "jwt_tokezn_here".to_string(),
/// };
///
/// // In a real handler, cookies would be extracted from the request
/// let logged_in_builder = login_builder(builder, cookies)?;
/// # Ok(())
/// # }
/// ```
pub fn login_builder(auth: EjBuilderApi, cookies: &Cookies) -> Result<EjBuilderApi> {
    cookies.add(Cookie::new(AUTH_TOKEN_COOKIE, auth.token.clone()));
    Ok(auth)
}

/// Logs in a client and sets authentication cookie.
///
/// Authenticates the client credentials and generatezs a JWT token for subsequent requests.
///
/// # Examples
///
/// ```rust
/// use ej_web::ctx::resolver::login_client;
/// use ej_dispatcher_sdk::ejclient::EjClientLoginRequest;
/// use ej_models::db::connection::DbConnection;
/// use tower_cookies::Cookies;
///
/// # async fn example(connection: &DbConnection, cookies: &Cookies) -> Result<(), Box<dyn std::error::Error>> {
/// let request = EjClientLoginRequest {
///     name: "client-name".to_string(),
///     secret: "client-secret".to_string(),
/// };
///
/// let login_result = login_client(&request, connection, cookies)?;
/// println!("Client logged in with token: {}", login_result.access_token);
/// # Ok(())
/// # }
/// ```
pub fn login_client(
    auth: &EjClientLoginRequest,
    connection: &DbConnection,
    cookies: &Cookies,
) -> Result<EjClientLogin> {
    let (client, permissions) = authenticate(auth, connection)?;
    let token = generate_token(&client, permissions)?;
    cookies.add(Cookie::new(AUTH_TOKEN_COOKIE, token.access_token.clone()));

    Ok(EjClientLogin {
        access_token: token.access_token,
        token_type: token.token_type,
    })
}

impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        Ok(parts
            .extensions
            .get::<std::result::Result<Ctx, ej_auth::error::Error>>()
            .ok_or(Error::CtxMissing)?
            .clone()?)
    }
}
