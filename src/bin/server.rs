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
use ej::{
    db::{config::DbConfig, connection::DbConnection},
    ej_client::api::{EjClientApi, EjClientLogin, EjClientPost},
    ej_connected_client::EjConnectedClient,
    ej_message::{EjClientMessage, EjServerMessage},
    require_permission,
    web::{
        auth::{AuthBody, authenticate_and_generate_token},
        ctx::{Ctx, mw_ctx_resolver},
        mw_auth::mw_require_auth,
    },
};
use tokio::sync::mpsc::{Receiver, channel};
use tower_cookies::{CookieManagerLayer, Cookies};

use std::net::SocketAddr;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use axum::extract::connect_info::ConnectInfo;
use axum::extract::ws::CloseFrame;

use ej::prelude::*;

//allows to split the websocket stream into separate TX and RX branches
use futures::{sink::SinkExt, stream::StreamExt};

#[derive(Clone)]
struct ApiState {
    clients: Vec<EjConnectedClient>,
    connection: DbConnection,
}
impl ApiState {
    pub fn new(connection: DbConnection) -> Self {
        Self {
            connection,
            clients: Vec::new(),
        }
    }
}

fn v1(path: &str) -> String {
    format!("/v1/{path}")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = DbConnection::new(&DbConfig::from_env());
    let api_state = ApiState::new(db);

    let app = Router::new()
        .route(&v1("builder/ws"), any(builder_handler))
        .route_layer(require_permission!("builder"))
        .route_layer(middleware::from_fn(mw_require_auth))
        .route(&v1("login"), post(login))
        .route(&v1("client"), post(post_client))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .layer(CorsLayer::permissive())
        .layer(middleware::from_fn_with_state(
            api_state.clone(),
            mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new())
        .with_state(api_state);

    // run it with hyper
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

#[axum::debug_handler]
async fn post_client(
    State(state): State<ApiState>,
    Json(payload): Json<EjClientPost>,
) -> Result<Json<EjClientApi>> {
    Ok(Json(payload.persist(&state.connection)?))
}
#[axum::debug_handler]
async fn login(
    state: State<ApiState>,
    cookies: Cookies,
    Json(payload): Json<EjClientLogin>,
) -> Result<Json<AuthBody>> {
    Ok(Json(authenticate_and_generate_token(
        &payload,
        &state.connection,
        &cookies,
    )?))
}

/// The handler for the HTTP request (this gets called when the HTTP request lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
#[axum::debug_handler]
async fn builder_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    ctx: Ctx,
    mut state: State<ApiState>,
) -> impl IntoResponse {
    println!("Client at {addr} connected.");
    let (tx, rx) = channel(2);

    state.clients.push(ctx.client.connect(tx, addr));
    ws.on_upgrade(move |socket| handle_socket(socket, addr, rx))
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(mut socket: WebSocket, who: SocketAddr, mut rx: Receiver<EjServerMessage>) {
    // send a ping (unsupported by some browsers) just to kick things off and get a response
    if socket
        .send(Message::Ping(Bytes::from_static(&[1, 2, 3])))
        .await
        .is_ok()
    {
        tracing::debug!("Pinged {who}...");
    } else {
        tracing::error!("Failed to send ping message to {who}. Closing connection");
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

    // By splitting socket we can send and receive at the same time. In this example we will send
    // unsolicited messages to client based on some sort of server's internal event (i.e .timer).
    let (mut sender, mut receiver) = socket.split();

    // Spawn a task that will push several messages to the client (does not matter what client does)
    let mut send_task: tokio::task::JoinHandle<Result<()>> = tokio::spawn(async move {
        loop {
            let message = rx.recv().await.ok_or(Error::Generic(String::from(
                "Couldn't receive data from channel",
            )))?;

            let is_close = message == EjServerMessage::Close;

            if is_close {
                println!("Sending close to {who}...");
                sender
                    .send(Message::Close(Some(CloseFrame {
                        code: axum::extract::ws::close_code::NORMAL,
                        reason: Utf8Bytes::from_static("Goodbye"),
                    })))
                    .await
                    .map_err(|_| {
                        Error::Generic(String::from("Couldn't receive data from channel"))
                    })?;

                return Ok(());
            }
            let serialized_message = serde_json::to_string(&message)
                .map_err(|_| Error::Generic(String::from("Failed to serialize message")))?;

            sender
                .send(Message::Text(serialized_message.into()))
                .await
                .map_err(|_| {
                    Error::Generic(String::from("Couldn't send message data to socket"))
                })?;
        }
    });

    let mut recv_task = tokio::spawn(async move {
        loop {
            let message = receiver
                .next()
                .await
                .ok_or(Error::Generic(String::from(
                    "Failed to receive message from socket",
                )))?
                .map_err(|_| {
                    Error::Generic(String::from("Failed to receive message from socket"))
                })?;

            match message {
                Message::Text(t) => {
                    let message: EjClientMessage = serde_json::from_str(&t).map_err(|_| {
                        Error::Generic(String::from("Failed to parse message from client"))
                    })?;

                    match message {
                        EjClientMessage::Results { results } => todo!(),
                        EjClientMessage::JobLog { log } => todo!(),
                        EjClientMessage::JobFailure => todo!(),
                        EjClientMessage::JobSucess => todo!(),
                    }
                }
                Message::Close(c) => {
                    if let Some(cf) = c {
                        tracing::info!(
                            ">>> {} sent close with code {} and reason `{}`",
                            who,
                            cf.code,
                            cf.reason
                        );
                    } else {
                        tracing::warn!(">>> {who} somehow sent close message without CloseFrame");
                    }
                    return Ok(());
                }
                Message::Binary(_) => {
                    return Err(Error::Generic(String::from(
                        "Invalid message format from client",
                    )));
                }
                Message::Ping(_) | Message::Pong(_) => {}
            }
        }
    });

    // If any one of the tasks exit, abort the other.
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
    tracing::info!("Websocket context {who} destroyed");
}
