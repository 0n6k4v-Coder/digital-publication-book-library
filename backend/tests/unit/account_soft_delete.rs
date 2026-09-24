use digital_publication_backend::domains::account::model::{
    SoftDeleteState, SoftDeleteValidationError,
};

#[test]
fn soft_delete_rejects_already_deleted_account() {
    let result = SoftDeleteState {
        is_active: false,
        is_deleted: true,
        is_administrator: true,
        active_administrator_count: 1,
    }
    .validate();

    assert_eq!(
        result,
        Err(SoftDeleteValidationError::AccountAlreadyDeleted)
    );
}

#[test]
fn soft_delete_rejects_last_active_administrator() {
    let result = SoftDeleteState {
        is_active: true,
        is_deleted: false,
        is_administrator: true,
        active_administrator_count: 1,
    }
    .validate();

    assert_eq!(
        result,
        Err(SoftDeleteValidationError::LastActiveAdministrator)
    );
}

#[test]
fn soft_delete_allows_active_administrator_when_another_remains() {
    let result = SoftDeleteState {
        is_active: true,
        is_deleted: false,
        is_administrator: true,
        active_administrator_count: 2,
    }
    .validate();

    assert_eq!(result, Ok(()));
}

#[test]
fn soft_delete_allows_inactive_administrator() {
    let result = SoftDeleteState {
        is_active: false,
        is_deleted: false,
        is_administrator: true,
        active_administrator_count: 1,
    }
    .validate();

    assert_eq!(result, Ok(()));
}

#[test]
fn soft_delete_allows_active_non_administrator() {
    let result = SoftDeleteState {
        is_active: true,
        is_deleted: false,
        is_administrator: false,
        active_administrator_count: 1,
    }
    .validate();

    assert_eq!(result, Ok(()));
}