use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
};
use digital_publication_backend::{
    domains::authentication::model::{
        ACCESS_TOKEN_EXPIRES_IN,
        AUTHENTICATION_SESSION_EXPIRES_IN,
        REFRESH_TOKEN_POLICY_EXPIRES_IN,
    },
    shared::error::AppError,
};

#[test]
fn token_lifetimes_match_the_authentication_contract() {
    assert_eq!(ACCESS_TOKEN_EXPIRES_IN, 3_600);
    assert_eq!(AUTHENTICATION_SESSION_EXPIRES_IN, 86_400);
    assert_eq!(REFRESH_TOKEN_POLICY_EXPIRES_IN, 2_592_000);
}

#[test]
fn invalid_refresh_token_has_no_bearer_challenge() {
    let response = AppError::InvalidRefreshToken.into_response();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert!(response.headers().get(header::WWW_AUTHENTICATE).is_none());
}

#[test]
fn refresh_rate_limiting_is_not_reused_for_refresh_errors() {
    let response = AppError::InvalidRefreshToken.into_response();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_ne!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}