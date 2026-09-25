use std::{
    io::{self, Write},
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use digital_publication_backend::{
    app::{router::build_router, state::AppState},
    shared::validation::PasswordBlocklist,
};
use tower::ServiceExt;
use tracing_subscriber::fmt::writer::MakeWriter;

#[derive(Clone)]
struct SharedLogWriter(Arc<Mutex<Vec<u8>>>);

impl Write for SharedLogWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .expect("lock log buffer")
            .extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for SharedLogWriter {
    type Writer = SharedLogWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

fn test_router() -> axum::Router {
    let pool = sqlx::PgPool::connect_lazy("postgres://invalid")
        .expect("build lazy PostgreSQL pool");
    let blocklist = PasswordBlocklist::from_hashes("test", Vec::<[u8; 20]>::new());

    build_router(AppState::new(
        pool,
        Arc::new(blocklist),
        NonZeroUsize::new(2).expect("non-zero semaphore size"),
    ))
}

#[tokio::test]
async fn application_logging_never_records_credentials_or_query_parameters() {
    let logs = Arc::new(Mutex::new(Vec::new()));
    let writer = SharedLogWriter(logs.clone());
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_ansi(false)
        .with_writer(writer)
        .finish();

    let bearer_token = "bearer-secret-that-must-not-be-logged";
    let password = "password-secret-that-must-not-be-logged";

    let request = Request::builder()
        .method("POST")
        .uri(format!("/auth/login?token={bearer_token}"))
        .header(header::AUTHORIZATION, format!("Bearer {bearer_token}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(format!(
            r#"{{"email":"user@example.com","password":"{password}""#
        )))
        .expect("build request");

    let _subscriber_guard = tracing::subscriber::set_default(subscriber);
    let response = test_router()
        .oneshot(request)
        .await
        .expect("run request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let log_output = String::from_utf8(
        logs.lock()
            .expect("lock log buffer")
            .clone(),
    )
    .expect("decode logs");

    assert!(log_output.contains("http request completed"));
    assert!(log_output.contains("/auth/login"));
    assert!(!log_output.contains(bearer_token));
    assert!(!log_output.contains(password));
    assert!(!log_output.contains("?token="));
    assert!(!log_output.contains("authorization"));
}