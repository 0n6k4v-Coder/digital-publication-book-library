use digital_publication_backend::domains::account::model::{
    HardDeleteState, HardDeleteValidationError,
};

#[test]
fn hard_delete_rejects_the_last_active_administrator() {
    let result = HardDeleteState {
        is_active: true,
        is_deleted: false,
        is_administrator: true,
        active_administrator_count: 1,
    }
    .validate();

    assert_eq!(
        result,
        Err(HardDeleteValidationError::LastActiveAdministrator)
    );
}

#[test]
fn hard_delete_allows_active_administrator_when_another_remains() {
    let result = HardDeleteState {
        is_active: true,
        is_deleted: false,
        is_administrator: true,
        active_administrator_count: 2,
    }
    .validate();

    assert_eq!(result, Ok(()));
}

#[test]
fn hard_delete_allows_active_non_administrator() {
    let result = HardDeleteState {
        is_active: true,
        is_deleted: false,
        is_administrator: false,
        active_administrator_count: 1,
    }
    .validate();

    assert_eq!(result, Ok(()));
}

#[test]
fn hard_delete_allows_inactive_administrator() {
    let result = HardDeleteState {
        is_active: false,
        is_deleted: false,
        is_administrator: true,
        active_administrator_count: 0,
    }
    .validate();

    assert_eq!(result, Ok(()));
}

#[test]
fn hard_delete_allows_soft_deleted_administrator() {
    let result = HardDeleteState {
        is_active: false,
        is_deleted: true,
        is_administrator: true,
        active_administrator_count: 0,
    }
    .validate();

    assert_eq!(result, Ok(()));
}