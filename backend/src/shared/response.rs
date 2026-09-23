use axum::http::{header, HeaderMap, HeaderValue};

pub fn add_no_store(headers: &mut HeaderMap) {
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    );
}