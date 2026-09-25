use digital_publication_backend::domains::account::model::{
    RestoreState, RestoreValidationError,
};

#[test]
fn restore_rejects_non_deleted_account() {
    let result = RestoreState { is_deleted: false }.validate();

    assert_eq!(result, Err(RestoreValidationError::AccountNotDeleted));
}

#[test]
fn restore_allows_soft_deleted_account() {
    let result = RestoreState { is_deleted: true }.validate();

    assert_eq!(result, Ok(()));
}