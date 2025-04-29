use crate::prelude::*;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use super::ctx::Ctx;

pub async fn mw_require_auth(ctx: Result<Ctx>, req: Request, next: Next) -> Result<Response> {
    ctx?;
    Ok(next.run(req).await)
}

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

#[macro_export]
macro_rules! require_permission {
    ($permission:expr) => {{
        use ej::web::mw_auth::mw_require_permission;
        axum::middleware::from_fn_with_state($permission, mw_require_permission)
    }};
}
