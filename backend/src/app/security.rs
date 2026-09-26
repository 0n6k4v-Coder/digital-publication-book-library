use std::sync::Arc;

use axum::{
    body::Body,
    extract::Request,
    http::{header, HeaderMap, HeaderName, HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Router,
};

use crate::shared::error::AppError;

const ALLOWED_METHODS_VALUE: &str = "GET, POST, PATCH, DELETE";
const ALLOWED_HEADERS_VALUE: &str = "Accept, Authorization, Content-Type";
const ACCESS_CONTROL_MAX_AGE_SECONDS: &str = "600";

const AUTHENTICATION_ENDPOINTS: [&str; 3] = ["/auth/login", "/auth/refresh", "/auth/logout"];

pub fn apply_security(router: Router, allowed_origins: Vec<HeaderValue>) -> Router {
    let allowed_origins = Arc::<[HeaderValue]>::from(allowed_origins);

    router.layer(middleware::from_fn(
        move |request: Request<Body>, next: Next| {
            let allowed_origins = Arc::clone(&allowed_origins);

            async move { apply_origin_and_cors(request, next, allowed_origins).await }
        },
    ))
}

async fn apply_origin_and_cors(
    request: Request<Body>,
    next: Next,
    allowed_origins: Arc<[HeaderValue]>,
) -> Response {
    let requires_origin = requires_origin_policy(&request);

    let Some(origin) = request.headers().get(header::ORIGIN).cloned() else {
        if requires_origin {
            return AppError::Forbidden.into_response();
        }

        return next.run(request).await;
    };

    if !origin_is_allowed(&origin, &allowed_origins) {
        return AppError::Forbidden.into_response();
    }

    if request.method() == Method::OPTIONS
        && request
            .headers()
            .contains_key(header::ACCESS_CONTROL_REQUEST_METHOD)
    {
        return preflight_response(&origin, request.headers());
    }

    let mut response = next.run(request).await;

    add_cors_headers(response.headers_mut(), &origin);

    response
}

fn requires_origin_policy(request: &Request<Body>) -> bool {
    request.method() == Method::POST && AUTHENTICATION_ENDPOINTS.contains(&request.uri().path())
}

fn origin_is_allowed(origin: &HeaderValue, allowed_origins: &[HeaderValue]) -> bool {
    allowed_origins
        .iter()
        .any(|allowed_origin| allowed_origin == origin)
}

fn preflight_response(origin: &HeaderValue, request_headers: &HeaderMap) -> Response {
    let requested_method = request_headers
        .get(header::ACCESS_CONTROL_REQUEST_METHOD)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<Method>().ok());

    let Some(requested_method) = requested_method else {
        return AppError::Forbidden.into_response();
    };

    if !method_is_allowed(&requested_method) {
        return AppError::Forbidden.into_response();
    }

    if let Some(requested_headers) = request_headers.get(header::ACCESS_CONTROL_REQUEST_HEADERS) {
        if !request_headers_are_allowed(requested_headers) {
            return AppError::Forbidden.into_response();
        }
    }

    let mut response = StatusCode::NO_CONTENT.into_response();

    add_cors_headers(response.headers_mut(), origin);

    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static(ALLOWED_METHODS_VALUE),
    );
    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static(ALLOWED_HEADERS_VALUE),
    );
    response.headers_mut().insert(
        header::ACCESS_CONTROL_MAX_AGE,
        HeaderValue::from_static(ACCESS_CONTROL_MAX_AGE_SECONDS),
    );

    append_vary_header(response.headers_mut(), "Access-Control-Request-Method");
    append_vary_header(response.headers_mut(), "Access-Control-Request-Headers");

    response
}

fn add_cors_headers(headers: &mut HeaderMap, origin: &HeaderValue) {
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.clone());
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
        HeaderValue::from_static("true"),
    );

    append_vary_header(headers, "Origin");
}

fn append_vary_header(headers: &mut HeaderMap, value: &str) {
    let already_present = headers
        .get_all(header::VARY)
        .iter()
        .filter_map(|header| header.to_str().ok())
        .flat_map(|header| header.split(','))
        .any(|entry| entry.trim().eq_ignore_ascii_case(value));

    if !already_present {
        headers.append(
            header::VARY,
            HeaderValue::from_str(value).expect("static Vary value must be valid"),
        );
    }
}

fn method_is_allowed(method: &Method) -> bool {
    *method == Method::GET
        || *method == Method::POST
        || *method == Method::PATCH
        || *method == Method::DELETE
}

fn request_headers_are_allowed(value: &HeaderValue) -> bool {
    let Ok(value) = value.to_str() else {
        return false;
    };

    value.split(',').all(|header_name| {
        let header_name = header_name.trim();

        if header_name.is_empty() {
            return false;
        }

        let Ok(header_name) = HeaderName::from_bytes(header_name.as_bytes()) else {
            return false;
        };

        header_name == header::ACCEPT
            || header_name == header::AUTHORIZATION
            || header_name == header::CONTENT_TYPE
    })
}
