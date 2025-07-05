//! Authentication middleware for protecting routes.
//!
//! This module provides middleware functions and macros for protecting routes
//! that require authentication and specific permissions.

use crate::prelude::*;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use super::ctx::Ctx;

/// Middleware that requires authentication for a route.
///
/// This middleware checks if a valid authentication context exists.
/// If no valid context is found, the request is rejected.
///
/// # Examples
///
/// ```rust,no_run
/// use axum::{Router, routing::get};
/// use ej_web::mw_auth::mw_require_auth;
///
/// let app: Router<()> = Router::new()
///     .route("/protected", get(protected_handler))
///     .layer(axum::middleware::from_fn(mw_require_auth));
///
/// async fn protected_handler() -> &'static str {
///     "This requires authentication"
/// }
/// ```
pub async fn mw_require_auth(ctx: Result<Ctx>, req: Request, next: Next) -> Result<Response> {
    ctx?;
    Ok(next.run(req).await)
}

/// Middleware that requires a specific permission for a route.
///
/// This middleware checks if the authenticated user has the required permission.
/// If the permission is not present, the request is rejected with a forbidden error.
///
/// # Examples
///
/// ```rust
/// use axum::{Router, routing::get};
/// use ej_web::mw_auth::mw_require_permission;
///
/// let app: Router<()> = Router::new()
///     .route("/admin", get(admin_handler))
///     .layer(axum::middleware::from_fn_with_state("admin", mw_require_permission));
///
/// async fn admin_handler() -> &'static str {
///     "This requires admin permission"
/// }
/// ```
pub async fn mw_require_permission(
    State(permission): State<&'static str>,
    ctx: Ctx,
    req: Request,
    next: Next,
) -> Result<Response> {
    if !ctx.permissions.contains(permission) {
        return Err(Error::ApiForbidden);
    }
    Ok(next.run(req).await)
}

/// A macro for creating permission-required middleware.
///
/// This macro simplifies the creation of middleware that requires specific permissions.
/// It automatically imports the necessary dependencies and creates the middleware layer.
///
/// # Examples
///
/// ```rust
/// use axum::{Router, routing::get};
/// use ej_web::require_permission;
///
/// let app: Router<()> = Router::new()
///     .route("/admin", get(admin_handler))
///     .layer(require_permission!("admin"))
///     .route("/user", get(user_handler))
///     .layer(require_permission!("user"));
///
/// async fn admin_handler() -> &'static str {
///     "Admin only"
/// }
///
/// async fn user_handler() -> &'static str {
///     "User permission required"
/// }
/// ```
#[macro_export]
macro_rules! require_permission {
    ($permission:expr) => {{
        use ej_web::mw_auth::mw_require_permission;
        axum::middleware::from_fn_with_state($permission, mw_require_permission)
    }};
}
