use std::{env, net::SocketAddr, sync::Arc};

use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use digital_publication_backend::{
    app::{router::build_router, state::AppState},
    shared::validation::PasswordBlocklist,
};
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
#[ignore = "requires deployment-managed TLS certificate/key files and a built backend"]
async fn authentication_api_is_https_only() {
    let cert_path =
        env::var("TEST_TLS_CERT_PATH")
            .expect("TEST_TLS_CERT_PATH must be set");

    let key_path =
        env::var("TEST_TLS_KEY_PATH")
            .expect("TEST_TLS_KEY_PATH must be set");

    let _ = rustls::crypto::ring::default_provider()
        .install_default();

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy(
            "postgres://postgres:postgres@127.0.0.1:1/unreachable",
        )
        .expect("build lazy PostgreSQL pool");

    let blocklist =
        PasswordBlocklist::from_hashes(
            "test",
            Vec::<[u8; 20]>::new(),
        );

    let app = build_router(AppState::new(
        pool,
        Arc::new(blocklist),
        std::num::NonZeroUsize::new(2)
            .expect("non-zero semaphore size"),
    ));

    let listener =
        std::net::TcpListener::bind("127.0.0.1:0")
            .expect("bind TLS listener");

    listener
        .set_nonblocking(true)
        .expect("set TLS listener non-blocking");

    let address =
        listener.local_addr()
            .expect("read TLS listener address");

    let tls_config =
        RustlsConfig::from_pem_file(
            &cert_path,
            &key_path,
        )
        .await
        .expect(
            "load TLS certificate and private key",
        );

    let server = tokio::spawn(async move {
        axum_server::from_tcp_rustls(
            listener,
            tls_config,
        )
        .serve(
            app.into_make_service_with_connect_info::<
                SocketAddr,
            >(),
        )
        .await
        .expect("serve HTTPS backend");
    });

    let https_client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("build HTTPS test client");

    let https_response = https_client
        .post(format!(
            "https://{address}/auth/login"
        ))
        .header(
            "content-type",
            "application/json",
        )
        .body("{")
        .send()
        .await
        .expect("send HTTPS request");

    assert_eq!(
        https_response.status(),
        reqwest::StatusCode::BAD_REQUEST
    );

    assert_eq!(
        https_response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("application/problem+json")
    );

    assert_eq!(
        https_response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );

    let plaintext_result = Client::new()
        .post(format!(
            "http://{address}/auth/login"
        ))
        .header(
            "content-type",
            "application/json",
        )
        .body("{")
        .send()
        .await;

    assert!(plaintext_result.is_err());

    server.abort();
}