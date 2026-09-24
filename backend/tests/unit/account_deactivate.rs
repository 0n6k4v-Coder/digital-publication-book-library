use digital_publication_backend::domains::account::model::{
    DeactivationState, DeactivationValidationError,
};

#[test]
fn deactivation_rejects_soft_deleted_account() {
    let result = DeactivationState {
        is_active: false,
        is_deleted: true,
        is_administrator: true,
        active_administrator_count: 1,
    }
    .validate();

    assert_eq!(result, Err(DeactivationValidationError::AccountNotFound));
}

#[test]
fn deactivation_rejects_inactive_account() {
    let result = DeactivationState {
        is_active: false,
        is_deleted: false,
        is_administrator: false,
        active_administrator_count: 2,
    }
    .validate();

    assert_eq!(
        result,
        Err(DeactivationValidationError::AccountAlreadyInactive)
    );
}

#[test]
fn deactivation_rejects_last_active_administrator() {
    let result = DeactivationState {
        is_active: true,
        is_deleted: false,
        is_administrator: true,
        active_administrator_count: 1,
    }
    .validate();

    assert_eq!(
        result,
        Err(DeactivationValidationError::LastActiveAdministrator)
    );
}

#[test]
fn deactivation_allows_administrator_when_another_active_administrator_remains() {
    let result = DeactivationState {
        is_active: true,
        is_deleted: false,
        is_administrator: true,
        active_administrator_count: 2,
    }
    .validate();

    assert_eq!(result, Ok(()));
}

#[test]
fn deactivation_allows_non_administrator_without_changing_administrator_count() {
    let result = DeactivationState {
        is_active: true,
        is_deleted: false,
        is_administrator: false,
        active_administrator_count: 1,
    }
    .validate();

    assert_eq!(result, Ok(()));
}