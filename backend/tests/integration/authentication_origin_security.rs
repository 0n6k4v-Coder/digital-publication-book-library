use axum::{
    body::Body,
    http::{header, HeaderValue, Request, StatusCode},
    routing::post,
    Router,
};
use digital_publication_backend::app::security::apply_security;
use tower::ServiceExt;

const ALLOWED_ORIGIN: &str = "https://admin.example.com";
const DISALLOWED_ORIGIN: &str = "https://attacker.example.com";

const AUTHENTICATION_ENDPOINTS: [&str; 3] =
    ["/auth/login", "/auth/refresh", "/auth/logout"];

async fn authentication_stub() -> StatusCode {
    StatusCode::NO_CONTENT
}

fn app() -> Router {
    apply_security(
        Router::new()
            .route("/auth/login", post(authentication_stub))
            .route("/auth/refresh", post(authentication_stub))
            .route("/auth/logout", post(authentication_stub)),
        vec![HeaderValue::from_static(ALLOWED_ORIGIN)],
    )
}

fn authentication_request(path: &str, origin: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method("POST").uri(path);

    if let Some(origin) = origin {
        builder = builder.header(header::ORIGIN, origin);
    }

    builder.body(Body::empty()).unwrap()
}

#[tokio::test]
async fn all_authentication_endpoints_require_origin() {
    for path in AUTHENTICATION_ENDPOINTS {
        let response = app()
            .oneshot(authentication_request(path, None))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::FORBIDDEN,
            "{path} must reject requests without Origin"
        );
        assert!(
            response
                .headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .is_none(),
            "{path} must not emit CORS headers when Origin is missing"
        );
    }
}

#[tokio::test]
async fn all_authentication_endpoints_reject_untrusted_origin() {
    for path in AUTHENTICATION_ENDPOINTS {
        let response = app()
            .oneshot(authentication_request(path, Some(DISALLOWED_ORIGIN)))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::FORBIDDEN,
            "{path} must reject an untrusted Origin"
        );
        assert!(
            response
                .headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .is_none(),
            "{path} must not emit CORS headers for an untrusted Origin"
        );
    }
}

#[tokio::test]
async fn all_authentication_endpoints_accept_only_the_explicit_origin() {
    for path in AUTHENTICATION_ENDPOINTS {
        let response = app()
            .oneshot(authentication_request(path, Some(ALLOWED_ORIGIN)))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::NO_CONTENT,
            "{path} should reach the handler for the explicit allowed Origin"
        );
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
        assert!(
            response
                .headers()
                .get_all(header::VARY)
                .iter()
                .any(|value| value == "Origin"),
            "{path} should vary responses by Origin"
        );
    }
}

#[tokio::test]
async fn login_preflight_accepts_the_browser_json_request_contract() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/auth/login")
                .header(header::ORIGIN, ALLOWED_ORIGIN)
                .header(
                    header::ACCESS_CONTROL_REQUEST_METHOD,
                    "POST",
                )
                .header(
                    header::ACCESS_CONTROL_REQUEST_HEADERS,
                    "content-type",
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
}