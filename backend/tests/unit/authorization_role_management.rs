use digital_publication_backend::domains::authorization::model::{
    RoleAssignmentState, RoleAssignmentValidationError, RoleRevocationState,
    RoleRevocationValidationError,
};

#[test]
fn role_assignment_rejects_missing_role() {
    let result = RoleAssignmentState {
        role_exists: false,
        role_enabled: false,
        account_exists: true,
        assignment_exists: false,
    }
    .validate();

    assert_eq!(
        result,
        Err(RoleAssignmentValidationError::RoleNotFound)
    );
}

#[test]
fn role_assignment_rejects_disabled_role() {
    let result = RoleAssignmentState {
        role_exists: true,
        role_enabled: false,
        account_exists: true,
        assignment_exists: false,
    }
    .validate();

    assert_eq!(
        result,
        Err(RoleAssignmentValidationError::RoleDisabled)
    );
}

#[test]
fn role_assignment_rejects_missing_account() {
    let result = RoleAssignmentState {
        role_exists: true,
        role_enabled: true,
        account_exists: false,
        assignment_exists: false,
    }
    .validate();

    assert_eq!(
        result,
        Err(RoleAssignmentValidationError::AccountNotFound)
    );
}

#[test]
fn role_assignment_rejects_duplicate_assignment() {
    let result = RoleAssignmentState {
        role_exists: true,
        role_enabled: true,
        account_exists: true,
        assignment_exists: true,
    }
    .validate();

    assert_eq!(
        result,
        Err(RoleAssignmentValidationError::RoleAlreadyAssigned)
    );
}

#[test]
fn role_assignment_allows_enabled_role_and_existing_account() {
    let result = RoleAssignmentState {
        role_exists: true,
        role_enabled: true,
        account_exists: true,
        assignment_exists: false,
    }
    .validate();

    assert_eq!(result, Ok(()));
}

#[test]
fn role_revocation_rejects_missing_assignment() {
    let result = RoleRevocationState {
        assignment_exists: false,
        is_account_admin: false,
        target_is_active: false,
        target_is_deleted: false,
        active_administrator_count: 0,
    }
    .validate();

    assert_eq!(
        result,
        Err(RoleRevocationValidationError::RoleAssignmentNotFound)
    );
}

#[test]
fn role_revocation_rejects_last_active_administrator() {
    let result = RoleRevocationState {
        assignment_exists: true,
        is_account_admin: true,
        target_is_active: true,
        target_is_deleted: false,
        active_administrator_count: 1,
    }
    .validate();

    assert_eq!(
        result,
        Err(RoleRevocationValidationError::LastActiveAdministrator)
    );
}

#[test]
fn role_revocation_allows_active_administrator_when_another_remains() {
    let result = RoleRevocationState {
        assignment_exists: true,
        is_account_admin: true,
        target_is_active: true,
        target_is_deleted: false,
        active_administrator_count: 2,
    }
    .validate();

    assert_eq!(result, Ok(()));
}

#[test]
fn role_revocation_allows_inactive_or_deleted_administrator() {
    let inactive = RoleRevocationState {
        assignment_exists: true,
        is_account_admin: true,
        target_is_active: false,
        target_is_deleted: false,
        active_administrator_count: 1,
    };

    let deleted = RoleRevocationState {
        assignment_exists: true,
        is_account_admin: true,
        target_is_active: false,
        target_is_deleted: true,
        active_administrator_count: 1,
    };

    assert_eq!(inactive.validate(), Ok(()));
    assert_eq!(deleted.validate(), Ok(()));
}

#[test]
fn role_revocation_allows_non_administrator_role() {
    let result = RoleRevocationState {
        assignment_exists: true,
        is_account_admin: false,
        target_is_active: true,
        target_is_deleted: false,
        active_administrator_count: 1,
    }
    .validate();

    assert_eq!(result, Ok(()));
}