use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
};
use digital_publication_backend::shared::error::AppError;

#[test]
fn missing_logout_authentication_uses_the_missing_bearer_challenge() {
    let response = AppError::Unauthorized.into_response();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers()[header::WWW_AUTHENTICATE],
        r#"Bearer realm="admin-api""#
    );
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
}

#[test]
fn invalid_logout_authentication_uses_the_invalid_token_challenge() {
    let response = AppError::InvalidAuthentication.into_response();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers()[header::WWW_AUTHENTICATE],
        r#"Bearer realm="admin-api", error="invalid_token""#
    );
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
}