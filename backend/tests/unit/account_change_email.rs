use digital_publication_backend::domains::account::model::ChangeEmailRequest;

#[test]
fn request_deserializes_the_email_field() {
    let request =
        serde_json::from_str::<ChangeEmailRequest>(r#"{"email":"new-admin@example.com"}"#)
            .expect("deserialize request");

    assert_eq!(request.email, "new-admin@example.com");
}