use secrecy::ExposeSecret;

use digital_publication_backend::domains::account::model::ChangePasswordRequest;

#[test]
fn request_deserializes_the_password_field() {
    let request = serde_json::from_str::<ChangePasswordRequest>(
        r#"{"password":"a secure password with enough length"}"#,
    )
    .expect("deserialize request");

    assert_eq!(
        request.password.expose_secret(),
        "a secure password with enough length"
    );
}

#[test]
fn request_supports_passwords_of_at_least_64_characters() {
    let password = "a".repeat(64);
    let body = serde_json::json!({"password": password}).to_string();

    let request =
        serde_json::from_str::<ChangePasswordRequest>(&body).expect("deserialize request");

    assert_eq!(request.password.expose_secret().chars().count(), 64);
}