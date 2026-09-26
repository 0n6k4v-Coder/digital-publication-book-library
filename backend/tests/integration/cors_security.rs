use axum::{
    body::Body,
    http::{header, HeaderValue, Request, StatusCode},
    routing::{get, post},
    Router,
};
use digital_publication_backend::app::security::apply_security;
use tower::ServiceExt;

const ALLOWED_ORIGIN: &str = "https://admin.example.com";
const DISALLOWED_ORIGIN: &str = "https://attacker.example.com";

fn app() -> Router {
    apply_security(
        Router::new()
            .route("/health", get(|| async { StatusCode::OK }))
            .route("/auth/logout", post(|| async { StatusCode::NO_CONTENT })),
        vec![HeaderValue::from_static(ALLOWED_ORIGIN)],
    )
}

#[tokio::test]
async fn allowed_origin_gets_credentialed_cors_headers() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .header(header::ORIGIN, ALLOWED_ORIGIN)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|value| value.to_str().ok()),
        Some(ALLOWED_ORIGIN)
    );
    assert_eq!(
        response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
            .and_then(|value| value.to_str().ok()),
        Some("true")
    );
    assert!(response
        .headers()
        .get_all(header::VARY)
        .iter()
        .any(|value| value == "Origin"));
}

#[tokio::test]
async fn disallowed_origin_is_rejected_server_side() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .header(header::ORIGIN, DISALLOWED_ORIGIN)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(response
        .headers()
        .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
        .is_none());
}

#[tokio::test]
async fn allowed_preflight_returns_the_required_cors_contract() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/health")
                .header(header::ORIGIN, ALLOWED_ORIGIN)
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
                .header(
                    header::ACCESS_CONTROL_REQUEST_HEADERS,
                    "authorization,content-type",
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|value| value.to_str().ok()),
        Some(ALLOWED_ORIGIN)
    );
    assert_eq!(
        response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
            .and_then(|value| value.to_str().ok()),
        Some("true")
    );
    assert_eq!(
        response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_METHODS)
            .and_then(|value| value.to_str().ok()),
        Some("GET, POST, PATCH, DELETE")
    );
    assert_eq!(
        response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_HEADERS)
            .and_then(|value| value.to_str().ok()),
        Some("Accept, Authorization, Content-Type")
    );
    assert_eq!(
        response
            .headers()
            .get(header::ACCESS_CONTROL_MAX_AGE)
            .and_then(|value| value.to_str().ok()),
        Some("600")
    );
    assert!(response
        .headers()
        .get_all(header::VARY)
        .iter()
        .any(|value| value == "Origin"));
    assert!(response
        .headers()
        .get_all(header::VARY)
        .iter()
        .any(|value| value == "Access-Control-Request-Method"));
    assert!(response
        .headers()
        .get_all(header::VARY)
        .iter()
        .any(|value| value == "Access-Control-Request-Headers"));
}

#[tokio::test]
async fn preflight_rejects_an_unapproved_request_header() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/health")
                .header(header::ORIGIN, ALLOWED_ORIGIN)
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
                .header(header::ACCESS_CONTROL_REQUEST_HEADERS, "x-not-allowed")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn authentication_endpoint_requires_an_origin() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/logout")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn authentication_endpoint_rejects_an_untrusted_origin() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/logout")
                .header(header::ORIGIN, DISALLOWED_ORIGIN)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn authentication_endpoint_accepts_an_allowed_origin() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/logout")
                .header(header::ORIGIN, ALLOWED_ORIGIN)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|value| value.to_str().ok()),
        Some(ALLOWED_ORIGIN)
    );
    assert_eq!(
        response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
            .and_then(|value| value.to_str().ok()),
        Some("true")
    );
}

#[tokio::test]
async fn requests_without_an_origin_remain_non_cors_requests() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response
        .headers()
        .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
        .is_none());
}
