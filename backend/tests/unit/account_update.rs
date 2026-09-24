use digital_publication_backend::domains::account::{
    model::UpdateAccountRequest,
    service::{normalize_display_name, DisplayNameValidationError},
};

#[test]
fn update_patch_supports_display_name_string() {
    let request = serde_json::from_str::<UpdateAccountRequest>(
        r#"{"display_name":"Library Administrator"}"#,
    )
    .expect("deserialize patch");

    assert_eq!(
        request.display_name,
        Some(Some("Library Administrator".to_owned()))
    );
}

#[test]
fn update_patch_supports_display_name_null() {
    let request =
        serde_json::from_str::<UpdateAccountRequest>(r#"{"display_name":null}"#)
            .expect("deserialize patch");

    assert_eq!(request.display_name, Some(None));
}

#[test]
fn update_patch_detects_empty_patch() {
    let request =
        serde_json::from_str::<UpdateAccountRequest>(r#"{}"#).expect("deserialize patch");

    assert_eq!(request.display_name, None);
}

#[test]
fn update_patch_rejects_unknown_fields() {
    let result = serde_json::from_str::<UpdateAccountRequest>(
        r#"{"display_name":"Library Administrator","roles":["account_admin"]}"#,
    );

    assert!(result.is_err());
}

#[test]
fn display_name_normalization_trims_unicode_whitespace_and_applies_nfc() {
    let normalized =
        normalize_display_name("\u{00A0}Cafe\u{0301}\u{2003}").expect("normalize display name");

    assert_eq!(normalized, "Café");
}

#[test]
fn display_name_normalization_preserves_internal_whitespace_and_case() {
    let normalized =
        normalize_display_name("  Library\tAdministrator  ").expect("normalize display name");

    assert_eq!(normalized, "Library\tAdministrator");
}

#[test]
fn display_name_normalization_rejects_empty_value() {
    assert_eq!(
        normalize_display_name(" \u{00A0}\t "),
        Err(DisplayNameValidationError::Empty)
    );
}

#[test]
fn display_name_normalization_rejects_more_than_100_unicode_scalars() {
    let value = "a".repeat(101);

    assert_eq!(
        normalize_display_name(&value),
        Err(DisplayNameValidationError::TooLong)
    );
}

#[test]
fn display_name_normalization_accepts_exactly_100_unicode_scalars() {
    let value = "a".repeat(100);

    assert_eq!(normalize_display_name(&value).unwrap(), value);
}