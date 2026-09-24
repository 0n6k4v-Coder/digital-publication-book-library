use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct RoleAssignmentState {
    pub role_exists: bool,
    pub role_enabled: bool,
    pub account_exists: bool,
    pub assignment_exists: bool,
}

impl RoleAssignmentState {
    pub fn validate(self) -> Result<(), RoleAssignmentValidationError> {
        if !self.role_exists {
            return Err(RoleAssignmentValidationError::RoleNotFound);
        }

        if !self.role_enabled {
            return Err(RoleAssignmentValidationError::RoleDisabled);
        }

        if !self.account_exists {
            return Err(RoleAssignmentValidationError::AccountNotFound);
        }

        if self.assignment_exists {
            return Err(RoleAssignmentValidationError::RoleAlreadyAssigned);
        }

        Ok(())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RoleAssignmentValidationError {
    #[error("role not found")]
    RoleNotFound,

    #[error("role is disabled")]
    RoleDisabled,

    #[error("account not found")]
    AccountNotFound,

    #[error("role is already assigned")]
    RoleAlreadyAssigned,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RoleRevocationState {
    pub assignment_exists: bool,
    pub is_account_admin: bool,
    pub target_is_active: bool,
    pub target_is_deleted: bool,
    pub active_administrator_count: i64,
}

impl RoleRevocationState {
    pub fn validate(self) -> Result<(), RoleRevocationValidationError> {
        if !self.assignment_exists {
            return Err(RoleRevocationValidationError::RoleAssignmentNotFound);
        }

        if self.is_account_admin
            && self.target_is_active
            && !self.target_is_deleted
            && self.active_administrator_count <= 1
        {
            return Err(RoleRevocationValidationError::LastActiveAdministrator);
        }

        Ok(())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RoleRevocationValidationError {
    #[error("role assignment not found")]
    RoleAssignmentNotFound,

    #[error("last active administrator")]
    LastActiveAdministrator,
}
