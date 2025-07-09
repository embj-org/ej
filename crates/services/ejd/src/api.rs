//! API server setup and routing for the EJ Dispatcher Service.

use axum::{
    Json, Router,
    body::Bytes,
    extract::{
        State,
        ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    },
    middleware,
    response::IntoResponse,
    routing::{any, post},
};
use ej_config::ej_config::{EjConfig, EjUserConfig};
use ej_dispatcher_sdk::{
    ejbuilder::EjBuilderApi,
    ejclient::{EjClientApi, EjClientLogin, EjClientLoginRequest, EjClientPost},
    ejjob::{
        EjDeployableJob, EjJob,
        results::{EjBuilderBuildResult, EjBuilderRunResult},
    },
    ejws_message::{EjWsClientMessage, EjWsServerMessage},
};
use ej_web::{
    ctx::{
        Ctx,
        resolver::{login_builder, login_client, mw_ctx_resolver},
    },
    ejclient::create_client,
    ejconfig::save_config,
    ejjob::create_job,
    mw_auth::mw_require_auth,
    require_permission,
    traits::job_result::EjJobResult,
};
use tokio::{sync::mpsc::channel, task::JoinHandle};
use tower_cookies::{CookieManagerLayer, Cookies};
use tracing::{error, info};

use std::net::SocketAddr;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};

use axum::extract::connect_info::ConnectInfo;
use axum::extract::ws::CloseFrame;
use futures::{sink::SinkExt, stream::StreamExt};

use crate::dispatcher::Dispatcher;
use crate::prelude::*;
use ej_web::prelude::Result as EjWebResult;

/// Helper function to create versioned API paths.
fn v1(path: &str) -> String {
    format!("/v1/{path}")
}

/// Sets up the API server with all routes and middleware.
///
/// Creates the HTTP server with:
/// - Builder authentication and management routes
/// - Client authentication and job dispatch routes
/// - WebSocket endpoints for real-time communication
/// - Middleware for authentication, logging, and CORS
///
/// # Returns
///
/// Returns a `JoinHandle` for the spawned HTTP server task.
pub async fn setup_api(dispatcher: Dispatcher) -> Result<JoinHandle<Result<()>>> {
    let builder_routes = Router::new()
        .route(&v1("builder/ws"), any(builder_handler))
        .route(&v1("builder/config"), post(push_config))
        .route(
            &v1("builder/build_result"),
            post(job_result::<EjBuilderBuildResult>),
        )
        .route(
            &v1("builder/run_result"),
            post(job_result::<EjBuilderRunResult>),
        )
        .route_layer(require_permission!("builder"))
        .route_layer(middleware::from_fn(mw_require_auth));

    let builder_create_routes = Router::new()
        .route(&v1("client/builder"), post(create_builder))
        .route_layer(require_permission!("builder.create"))
        .route_layer(middleware::from_fn(mw_require_auth));

    let client_dispatch_routes = Router::new()
        .route(&v1("client/dispatch"), post(dispatch_job))
        .route_layer(require_permission!("client.dispatch"))
        .route_layer(middleware::from_fn(mw_require_auth));

    let client_create_routes = Router::new()
        .route(&v1("client"), post(post_client))
        .route_layer(require_permission!("client.create"))
        .route_layer(middleware::from_fn(mw_require_auth));

    let client_routes = Router::new()
        .route(&v1("login"), post(login))
        .route(&v1("builder/login"), post(login_builder_api));

    let app = Router::new()
        .merge(builder_routes)
        .merge(client_routes)
        .merge(builder_create_routes)
        .merge(client_create_routes)
        .merge(client_dispatch_routes)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .layer(CorsLayer::permissive())
        .layer(middleware::from_fn_with_state(
            dispatcher.clone(),
            mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new())
        .with_state(dispatcher);

    // run it with hyper
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    let handle = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await?;
        Ok(())
    });

    Ok(handle)
}

/// Creates a new client in the system.
///
/// Handles POST requests to create new clients with authentication credentials.
async fn post_client(
    State(state): State<Dispatcher>,
    Json(payload): Json<EjClientPost>,
) -> EjWebResult<Json<EjClientApi>> {
    let client = create_client(payload, &state.connection)?;
    Ok(Json(client))
}

/// Creates a new builder for an authenticated client.
///
/// Generates a builder instance with appropriate permissions and authentication token
/// for the requesting client.
async fn create_builder(
    State(mut state): State<Dispatcher>,
    ctx: Ctx,
) -> EjWebResult<Json<EjBuilderApi>> {
    Ok(Json(ctx.client.create_builder(&mut state.connection)?))
}

/// Handles client login requests.
///
/// Authenticates clients using their credentials and sets authentication cookies
/// for subsequent requests.
async fn login(
    state: State<Dispatcher>,
    cookies: Cookies,
    Json(payload): Json<EjClientLoginRequest>,
) -> EjWebResult<Json<EjClientLogin>> {
    Ok(Json(login_client(&payload, &state.connection, &cookies)?))
}

/// Handles builder login requests.
///
/// Authenticates builders using their JWT tokens and sets authentication cookies
/// for WebSocket and API communication.
async fn login_builder_api(
    cookies: Cookies,
    Json(payload): Json<EjBuilderApi>,
) -> EjWebResult<Json<EjBuilderApi>> {
    Ok(Json(login_builder(payload, &cookies)?))
}

/// Dispatches a job to all connected builders.
///
/// Creates a deployable job from the request and sends it to all available builders
/// via WebSocket connections. Returns the created job for tracking.
async fn dispatch_job(
    State(mut state): State<Dispatcher>,
    Json(payload): Json<EjJob>,
) -> EjWebResult<Json<EjDeployableJob>> {
    let builders = state.builders.lock().await;
    let job = create_job(payload, &mut state.connection)?;
    for builder in builders.iter() {
        if let Err(err) = builder
            .tx
            .send(EjWsServerMessage::BuildAndRun(job.clone()))
            .await
        {
            tracing::error!("Failed to dispatch job {err}");
        }
    }
    Ok(Json(job))
}

/// Handles builder configuration uploads.
///
/// Receives and stores configuration from authenticated builders, converting
/// user configurations to the internal format.
#[axum::debug_handler]
async fn push_config(
    State(mut state): State<Dispatcher>,
    ctx: Ctx,
    Json(payload): Json<EjUserConfig>,
) -> EjWebResult<Json<EjConfig>> {
    let config = EjConfig::from_user_config(payload);
    let config = save_config(config, &ctx.client.id, &mut state.connection)?;
    Ok(Json(config))
}

/// Handles job result submissions from builders.
///
/// Generic endpoint that accepts build or run results from builders and
/// forwards them to the dispatcher for processing and storage.
async fn job_result<T: EjJobResult>(
    State(mut dispatcher): State<Dispatcher>,
    Json(payload): Json<T>,
) -> EjWebResult<()> {
    if let Err(err) = dispatcher.on_job_result(payload).await {
        error!("Failed to dispach job {err}");
        if matches!(err, Error::NoBuildersAvailable) {
            return Err(ej_web::error::Error::NoBuildersAvailable);
        } else {
            return Err(ej_web::error::Error::InternalErrorDispatchingJob);
        }
    } else {
        Ok(())
    }
}

/// The handler for the HTTP request (this gets called when the HTTP request lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
#[axum::debug_handler]
async fn builder_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    ctx: Ctx,
    State(state): State<Dispatcher>,
) -> impl IntoResponse {
    println!("Client at {addr} connected.");

    info!("ctx: {} {:?}", ctx.client.id, ctx.who);
    ws.on_upgrade(move |socket| handle_socket(ctx, state, socket, addr))
}

/// RAII guard to automatically remove builders from the dispatcher when connections close.
struct BuilderGuard {
    dispatcher: Dispatcher,
    index: usize,
}

impl Drop for BuilderGuard {
    /// Automatically removes the builder from the dispatcher's builder list when dropped.
    fn drop(&mut self) {
        let builders = self.dispatcher.builders.clone();
        let index = self.index;
        tokio::spawn(async move {
            builders.lock().await.remove(index);
        });
    }
}
/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(ctx: Ctx, dispatcher: Dispatcher, mut socket: WebSocket, addr: SocketAddr) {
    let (tx, mut rx) = channel(2);

    let builder_index = {
        let mut builders = dispatcher.builders.lock().await;
        builders.push(ctx.client.connect(tx.clone(), addr));
        builders.len() - 1
    };

    let _guard = BuilderGuard {
        dispatcher: dispatcher.clone(),
        index: builder_index,
    };

    if socket
        .send(Message::Ping(Bytes::from_static(&[1, 2, 3])))
        .await
        .is_ok()
    {
        tracing::debug!("Pinged {addr}...");
    } else {
        tracing::error!("Failed to send ping message to {addr}. Closing connection");
        return;
    }

    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            if let Message::Close(_) = msg {
                return;
            }
        } else {
            return;
        }
    }

    let (mut sender, mut receiver) = socket.split();

    let mut send_task: JoinHandle<Result<()>> = tokio::spawn(async move {
        loop {
            let message = rx.recv().await;

            if let Some(message) = message {
                let is_close = matches!(message, EjWsServerMessage::Close);

                if is_close {
                    println!("Sending close to {addr}...");
                    sender
                        .send(Message::Close(Some(CloseFrame {
                            code: axum::extract::ws::close_code::NORMAL,
                            reason: Utf8Bytes::from_static("Goodbye"),
                        })))
                        .await?;

                    return Ok(());
                }
                let serialized_message = serde_json::to_string(&message)?;

                sender
                    .send(Message::Text(serialized_message.into()))
                    .await?;
            } else {
                info!("Websocket send channel closed");
                return Ok(());
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        loop {
            let message = receiver
                .next()
                .await
                .ok_or(Error::WsSocketReceiveFail)?
                .map_err(|err| Error::WsSocketReceiveError(err.to_string()))?;

            match message {
                Message::Text(t) => {
                    let _message: EjWsClientMessage = serde_json::from_str(&t)?;
                }
                Message::Close(c) => {
                    if let Some(cf) = c {
                        tracing::info!(
                            ">>> {} sent close with code {} and reason `{}`",
                            addr,
                            cf.code,
                            cf.reason
                        );
                    } else {
                        tracing::warn!(">>> {addr} somehow sent close message without CloseFrame");
                    }
                    return Ok(());
                }
                Message::Binary(_) => {
                    return Err(Error::InvalidWsMessage);
                }
                Message::Ping(_) | Message::Pong(_) => {}
            }
        }
    });

    tokio::select! {
        rv_a = (&mut send_task) => {
            tracing::info!("{:?}", rv_a);
            recv_task.abort();
        },
        rv_b = (&mut recv_task) => {
            tracing::info!("{:?}", rv_b);
            send_task.abort();
        }
    }
    tracing::info!("Websocket context {addr} destroyed");
}
