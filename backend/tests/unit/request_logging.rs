use axum::{
    body::Body,
    http::{header, Request},
};
use digital_publication_backend::shared::request_logging::request_log_route;

#[test]
fn request_logging_does_not_use_the_raw_uri_for_unmatched_requests() {
    let bearer_token = "bearer-secret-that-must-not-be-logged";
    let request = Request::builder()
        .method("GET")
        .uri(format!("/unknown?access_token={bearer_token}"))
        .header(header::AUTHORIZATION, format!("Bearer {bearer_token}"))
        .body(Body::empty())
        .expect("build request");

    let route = request_log_route(&request);

    assert_eq!(route, "<unmatched>");
    assert!(!route.contains(bearer_token));
}