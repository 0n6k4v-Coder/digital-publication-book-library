use digital_publication_backend::domains::account::model::{
    ActivationState, ActivationValidationError,
};

#[test]
fn activation_rejects_soft_deleted_account() {
    let result = ActivationState {
        is_active: false,
        is_deleted: true,
    }
    .validate();

    assert_eq!(
        result,
        Err(ActivationValidationError::AccountSoftDeleted)
    );
}

#[test]
fn activation_rejects_already_active_account() {
    let result = ActivationState {
        is_active: true,
        is_deleted: false,
    }
    .validate();

    assert_eq!(
        result,
        Err(ActivationValidationError::AccountAlreadyActive)
    );
}

#[test]
fn activation_allows_inactive_non_deleted_account() {
    let result = ActivationState {
        is_active: false,
        is_deleted: false,
    }
    .validate();

    assert_eq!(result, Ok(()));
}