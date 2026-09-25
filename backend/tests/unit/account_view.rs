use digital_publication_backend::domains::account::model::{
    ListAccountsQuery, ListAccountsQueryValidationError,
};

#[test]
fn list_accounts_query_uses_documented_defaults() {
    let query = serde_json::from_str::<ListAccountsQuery>("{}")
        .expect("deserialize default list query");

    assert_eq!(query.page, 1);
    assert_eq!(query.page_size, 20);
    assert!(query.status.is_none());
    assert!(!query.include_deleted);
}

#[test]
fn list_accounts_query_accepts_documented_values() {
    let query = serde_json::from_str::<ListAccountsQuery>(
        r#"{
            "page": 2,
            "page_size": 50,
            "status": "inactive",
            "include_deleted": true
        }"#,
    )
    .expect("deserialize valid list query");

    assert_eq!(query.page, 2);
    assert_eq!(query.page_size, 50);
    assert_eq!(query.status.as_deref(), Some("inactive"));
    assert!(query.include_deleted);

    assert!(query.validate().is_ok());
}

#[test]
fn list_accounts_query_accepts_active_status() {
    let query = serde_json::from_str::<ListAccountsQuery>(
        r#"{
            "status": "active"
        }"#,
    )
    .expect("deserialize active-status query");

    assert!(query.validate().is_ok());
}

#[test]
fn list_accounts_query_rejects_page_zero() {
    let query = serde_json::from_str::<ListAccountsQuery>(
        r#"{
            "page": 0
        }"#,
    )
    .expect("deserialize query");

    assert_eq!(
        query.validate(),
        Err(ListAccountsQueryValidationError::PageMustBePositive)
    );
}

#[test]
fn list_accounts_query_rejects_page_size_zero() {
    let query = serde_json::from_str::<ListAccountsQuery>(
        r#"{
            "page_size": 0
        }"#,
    )
    .expect("deserialize query");

    assert_eq!(
        query.validate(),
        Err(ListAccountsQueryValidationError::PageSizeOutOfRange)
    );
}

#[test]
fn list_accounts_query_rejects_page_size_above_maximum() {
    let query = serde_json::from_str::<ListAccountsQuery>(
        r#"{
            "page_size": 101
        }"#,
    )
    .expect("deserialize query");

    assert_eq!(
        query.validate(),
        Err(ListAccountsQueryValidationError::PageSizeOutOfRange)
    );
}

#[test]
fn list_accounts_query_rejects_invalid_status() {
    let query = serde_json::from_str::<ListAccountsQuery>(
        r#"{
            "status": "deleted"
        }"#,
    )
    .expect("deserialize query");

    assert_eq!(
        query.validate(),
        Err(ListAccountsQueryValidationError::InvalidStatus)
    );
}

#[test]
fn list_accounts_query_accepts_boundary_page_size() {
    let query = serde_json::from_str::<ListAccountsQuery>(
        r#"{
            "page_size": 100
        }"#,
    )
    .expect("deserialize query");

    assert!(query.validate().is_ok());
}

#[test]
fn list_accounts_query_rejects_invalid_numeric_values() {
    assert!(
        serde_json::from_str::<ListAccountsQuery>(
            r#"{
                "page": "abc"
            }"#,
        )
        .is_err()
    );

    assert!(
        serde_json::from_str::<ListAccountsQuery>(
            r#"{
                "page_size": "abc"
            }"#,
        )
        .is_err()
    );
}