use std::{net::SocketAddr, sync::Arc};

use axum_server::tls_rustls::RustlsConfig;
use digital_publication_backend::{
    app::{router::build_router, state::AppState},
    shared::validation::PasswordBlocklist,
};
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;

const TEST_TLS_CERT: &str = r#"-----BEGIN CERTIFICATE-----
MIIDJTCCAg2gAwIBAgIUeV+36J/GT8SiFpLooKZK0OEUNawwDQYJKoZIhvcNAQEL
BQAwFDESMBAGA1UEAwwJbG9jYWxob3N0MB4XDTI2MDkyNTAyNDkzOVoXDTM2MDky
MjAyNDkzOVowFDESMBAGA1UEAwwJbG9jYWxob3N0MIIBIjANBgkqhkiG9w0BAQEF
AAOCAQ8AMIIBCgKCAQEA5Edy7gp3RvCO9+JT0cKDELmyN/7qvkI/EesOxm5zYMVt
dXk3XgQq5SfRf9l6vHrQbcqfe7v9UO+pOu8UPWoaRmee28Dey7a6gKBr2Vl59riq
SBjip9SH09/uX8QTPy6pgjWUTmvYwJpV2/nv8CDBF5Qya3N2aAPIaMeaT60fJW6G
K22knF+Ka2x9ii/LgUCq8WgQ20AQ5WO8cU4kvMVx7vjJ3C+Uw4rNyhFF8+wCivRx
sAzSdAbWuUSq9V+T9CJJPHTeHlOugdzuX6fqEdBWJUsgDW+6Ynvx4H/uxylP8KQM
/tYQ8plRFC75CAh5XH4Gb2lTa19aIgRDOzQ8BLmwJQIDAQABo28wbTAdBgNVHQ4E
FgQU/OUYxzTUqmzJc/ADS94uz3sRtKswHwYDVR0jBBgwFoAU/OUYxzTUqmzJc/AD
S94uz3sRtKswDwYDVR0TAQH/BAUwAwEB/zAaBgNVHREEEzARgglsb2NhbGhvc3SH
BH8AAAEwDQYJKoZIhvcNAQELBQADggEBAHCTwPeQk0lKUtqS/geb4wNohLbNuOE+
MI90BIasnNmkTIJqfFKJrzwIkh2Xs5d5T8zuQrRZVjjwZeHCJMvNrioQzOihps/i
MwJAri7UC7EC++IJzyoFdKD/AV16g1PJR0Q9ezxvbZLwaS+xhtp0Ge4V/GCkrMjI
rFQAollmJs5AY0cL7fIKr6Hz5imITOChQC+GlHgg6qotFMNvFd1NcYyjdHy20rJJ
Pjx5UqQAmBib9IFSPOh6eGQVbOrbpU1u5VjjVJMIth1zAHciapCsFwLSeOOv/LXN
1odPVL/SZUbZi6Yyt2BgDm7Ku+ck1ErlA561PDPyKrLj5YTjH8/N8eM=
-----END CERTIFICATE-----"#;

const TEST_TLS_KEY: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvAIBADANBgkqhkiG9w0BAQEFAASCBKYwggSiAgEAAoIBAQDkR3LuCndG8I73
4lPRwoMQubI3/uq+Qj8R6w7GbnNgxW11eTdeBCrlJ9F/2Xq8etBtyp97u/1Q76k6
7xQ9ahpGZ57bwN7LtrqAoGvZWXn2uKpIGOKn1IfT3+5fxBM/LqmCNZROa9jAmlXb
+e/wIMEXlDJrc3ZoA8hox5pPrR8lboYrbaScX4prbH2KL8uBQKrxaBDbQBDlY7xx
TiS8xXHu+MncL5TDis3KEUXz7AKK9HGwDNJ0Bta5RKr1X5P0Ikk8dN4eU66B3O5f
p+oR0FYlSyANb7pie/Hgf+7HKU/wpAz+1hDymVEULvkICHlcfgZvaVNrX1oiBEM7
NDwEubAlAgMBAAECggEAA2f66ybuIFMaaFk9o8VDQQV8C298BwjEYlFgpvpcXHd5
aWSahOh1pQ92lwtkvuy4RkguDw6Sa2BLj7ksn9sadSnYWieTCEYGcJ7tw/jtsrSg
H0rpf9Anm0mTFLJuZha7VlVkw92GuhYwQJ1xOec2JZKaxLmUPfdQFXjTg+/vvvzK
/dREUVRu7b79fuOLwmpFVFHqHVbPktwmIpzLcL5v5aJ0qy/61IW4dLiNJwqXsU65
GUdrsCC1il3x67ya9HgSJFSSRvyeGJNuku/EFtFZO3fk64CGKyfGMc3ra7qWC0kQ
l2J6piLnpJBa8Ra1hRHOC7TztqryrmaEluJl0nYm0QKBgQD+lvTs8icoAVMlCqpA
U94s/kZNPAnlaNP88bzXkYJ2PUvvdAVd9EXZxQV5rpdQdj2uLb6N5AG4CUCPC6YV
zMxWTzl/Q2CDDWJ30skXtan9NN/lAwhVwkV3lyj++0yVfUZotA3cXaaBExFpraJM
7oEqahh47SjwOI91IXAjLFVq8QKBgQDliy4gJofrGJO4uFqlPWyuccHFEuuiiw3s
XtcjM+OfzNOTTD20QSOZHqkKUAR8xZD9ejzObQlxf3rJNIqqRIKEC5U9sotAm/z9
18MsoR8fO1qJLnnJgfFsALKqaMk4DmHibZckOqAnR7vaqJh2Dy2sxxfkMiV/e9zM
psc0z4zQdQKBgC2LO82XlEGn2wPpYIOZfUl3Q4RVlT+g/Stm42187mXQmWEA1GT2
afiHMm+OOCuAu5AJRumDPHt7zDzKzK9hr7xQ9+w4VW+cWV0uLCM9sGdHqjYB0N/m
nR7Dv+W9dvnXK11XuJMPfdXhX2AUW9B/akP4LuCTLJuswp0lmjXwnGdBAoGAAoWU
7CWAOMT8WnssA8S4/PGi/1dF33NHo+Em2+wmBAtsB6I+y0wr5/K+SK64XeaNwTsm
j94CzIxp/Ovm2hgGlwzJhvP/M6aDEQbdzg+3F9C/HeK009HppRYc4GJmU4dU6/fo
QS2jtMrE9ZIEmsdv6QYG7Srf3patxlHOvnXJRkECgYB1HUEGqyPT8E/iJsuhS32q
Thu6K0SH5cetxxuKgD/H7EOtjLLRlZTS27qizLeo+ihUP+JJOc48bejePZpW/wx9
2AvvGCYv7q8F0crnPE6KwGaUA8lV6HOpnYHZVmeVxgW+yQHMfW4R0QednzAaABOt
GbDkNON/YhIqR5t3A9BcwA==
-----END PRIVATE KEY-----"#;

#[tokio::test]
#[ignore = "requires a built backend"]
async fn authentication_api_is_https_only() {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy(
            "postgres://postgres:postgres@127.0.0.1:1/unreachable",
        )
        .expect("build lazy PostgreSQL pool");

    let blocklist =
        PasswordBlocklist::from_hashes("test", Vec::<[u8; 20]>::new());

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
        listener.local_addr().expect("read TLS listener address");

    let tls_config =
        RustlsConfig::from_pem(
            TEST_TLS_CERT.as_bytes().to_vec(),
            TEST_TLS_KEY.as_bytes().to_vec(),
        )
        .await
        .expect("load test TLS certificate and private key");

    let server = tokio::spawn(async move {
        axum_server::from_tcp_rustls(
            listener,
            tls_config,
        )
        .expect("create TLS server")
        .serve(
            app.into_make_service_with_connect_info::<SocketAddr>(),
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
        .header("content-type", "application/json")
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
        .header("content-type", "application/json")
        .body("{")
        .send()
        .await;

    assert!(plaintext_result.is_err());

    server.abort();
}