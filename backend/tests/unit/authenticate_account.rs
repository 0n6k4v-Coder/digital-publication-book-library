use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
};
use digital_publication_backend::{
    domains::authentication::{model::ACCESS_TOKEN_EXPIRES_IN, service::generate_opaque_token},
    shared::error::AppError,
};
use secrecy::ExposeSecret;

#[test]
fn generated_authentication_tokens_are_opaque_and_long_enough() {
    let token = generate_opaque_token();

    assert_eq!(token.expose_secret().len(), 96);
    assert!(token
        .expose_secret()
        .bytes()
        .all(|byte| byte.is_ascii_hexdigit()));
}

#[test]
fn generated_authentication_tokens_use_random_values() {
    let first = generate_opaque_token();
    let second = generate_opaque_token();

    assert_ne!(first.expose_secret(), second.expose_secret());
    assert_eq!(ACCESS_TOKEN_EXPIRES_IN, 3600);
}

#[tokio::test]
async fn invalid_credentials_are_generic_and_do_not_challenge_bearer_authentication() {
    let response = AppError::InvalidCredentials.into_response();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert!(response.headers().get(header::WWW_AUTHENTICATE).is_none());
}