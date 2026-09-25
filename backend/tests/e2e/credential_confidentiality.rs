use std::{
    net::SocketAddr,
    num::NonZeroUsize,
    sync::Arc,
};

use axum::Router;
use digital_publication_backend::{
    app::{router::build_router, state::AppState},
    shared::validation::PasswordBlocklist,
};
use reqwest::Client;
use tokio::net::TcpListener;

fn test_router() -> Router {
    let pool = sqlx::PgPool::connect_lazy("postgres://invalid")
        .expect("build lazy PostgreSQL pool");
    let blocklist = PasswordBlocklist::from_hashes("test", Vec::<[u8; 20]>::new());

    build_router(AppState::new(
        pool,
        Arc::new(blocklist),
        NonZeroUsize::new(2).expect("non-zero semaphore size"),
    ))
}

async fn start_server(app: Router) -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test server");
    let address = listener.local_addr().expect("read test server address");

    let task = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .expect("serve test application");
    });

    (address, task)
}

#[tokio::test]
async fn http_response_does_not_echo_bearer_or_password_credentials() {
    let bearer_token = "bearer-secret-that-must-not-be-returned";
    let password = "password-secret-that-must-not-be-returned";

    let (address, server) = start_server(test_router()).await;

    let response = Client::new()
        .post(format!("http://{address}/auth/login"))
        .header("Authorization", format!("Bearer {bearer_token}"))
        .header("Content-Type", "application/json")
        .body(format!(
            r#"{{"email":"user@example.com","password":"{password}""#
        ))
        .send()
        .await
        .expect("send credential-confidentiality request");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);

    let body = response.text().await.expect("read response body");

    assert!(!body.contains(bearer_token));
    assert!(!body.contains(password));

    server.abort();
}