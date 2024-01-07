use crate::scrape::Metrics;
use axum::extract::{ConnectInfo, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{middleware, Router};
use std::collections::HashMap;
use std::fmt::Display;
use std::io;
use std::net::SocketAddr;
use tokio::net::ToSocketAddrs;
use tokio::sync::watch::Receiver;
use tokio::time::Instant;

async fn request_logger(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let recv_time = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();

    let response = next.run(request).await;

    log::info!(
        "{} {} from {} - {} (latency: {} ms)",
        method,
        uri,
        addr,
        response.status(),
        (Instant::now() - recv_time).as_millis()
    );
    response
}

pub async fn serve<A>(
    listen: A,
    stats_rx: Receiver<Metrics>,
    metrics_path: Option<String>,
) -> io::Result<()>
where
    A: ToSocketAddrs + Display + Copy,
{
    let actual_metrics_path = metrics_path.unwrap_or("/metrics".into());

    let app = Router::new()
        .route(&actual_metrics_path, get(serve_metrics))
        .with_state(stats_rx)
        .layer(middleware::from_fn(request_logger));

    let listener = tokio::net::TcpListener::bind(listen).await?;

    log::info!("Serving metrics on http://{listen}{actual_metrics_path}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
}

async fn serve_metrics(
    State(rx): State<Receiver<Metrics>>,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, impl IntoResponse)> {
    let latest_stats = rx.borrow().clone();
    match serde_prometheus::to_string(&latest_stats, None, HashMap::new()) {
        Ok(s) => Ok((StatusCode::OK, s)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Encountered error while attempting to serialize response: {e}"),
        )),
    }
}
