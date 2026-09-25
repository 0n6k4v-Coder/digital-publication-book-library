use digital_publication_backend::domains::account::model::{AccountResponse, ViewedAccount};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[test]
fn account_response_contains_only_the_documented_fields() {
    let id = Uuid::new_v4();
    let timestamp = OffsetDateTime::now_utc();
    let account = ViewedAccount {
        id,
        email: "admin@example.com".to_owned(),
        status: "active".to_owned(),
        created_at: timestamp,
        updated_at: timestamp,
        deleted_at: None,
    };

    let payload: Value = serde_json::to_value(AccountResponse::from(account)).unwrap();

    assert_eq!(payload["id"], id.to_string());
    assert_eq!(payload["email"], "admin@example.com");
    assert_eq!(payload["status"], "active");
    assert_eq!(payload["deleted_at"], Value::Null);
    assert!(payload.get("password").is_none());
    assert!(payload.get("password_hash").is_none());
}

#[test]
fn account_response_maps_all_account_timestamps() {
    let id = Uuid::new_v4();
    let created_at = OffsetDateTime::from_unix_timestamp(1_758_605_600).unwrap();
    let updated_at = OffsetDateTime::from_unix_timestamp(1_758_605_601).unwrap();
    let deleted_at = OffsetDateTime::from_unix_timestamp(1_758_605_602).unwrap();

    let account = ViewedAccount {
        id,
        email: "inactive@example.com".to_owned(),
        status: "inactive".to_owned(),
        created_at,
        updated_at,
        deleted_at: Some(deleted_at),
    };

    let payload: Value = serde_json::to_value(AccountResponse::from(account)).unwrap();

    assert_eq!(
        payload["created_at"],
        created_at
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap()
    );
    assert_eq!(
        payload["updated_at"],
        updated_at
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap()
    );
    assert_eq!(
        payload["deleted_at"],
        deleted_at
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap()
    );
}