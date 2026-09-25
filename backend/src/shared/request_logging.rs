use std::time::Instant;

use axum::{
    extract::{MatchedPath, Request},
    middleware::Next,
    response::Response,
};
use tracing::info;

const UNMATCHED_ROUTE: &str = "<unmatched>";

/// Returns only the router's static matched path. The raw request URI is never
/// used so query parameters and unrecognized request targets cannot enter logs.
pub fn request_log_route(request: &Request) -> &str {
    request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
        .unwrap_or(UNMATCHED_ROUTE)
}

/// Application-wide HTTP logging policy.
///
/// Only non-secret request metadata is logged. Authorization headers,
/// cookies, query parameters, request bodies, and response bodies are never
/// inspected or emitted here.
pub async fn log_request(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let route = request_log_route(&request);
    let started_at = Instant::now();

    let response = next.run(request).await;

    info!(
        method = %method,
        route = %route,
        status = response.status().as_u16(),
        duration_ms = started_at.elapsed().as_millis() as u64,
        "http request completed"
    );

    response
}
