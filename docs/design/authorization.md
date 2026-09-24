# Authorization Domain Ready For Implement Design

## Table of Contents

1. [Requirements](#1-requirements)

   * [1.1 Authorization](#11-authorization)
   * [1.2 RBAC](#12-rbac)
   * [1.3 Integration](#13-integration)
   * [1.4 Non-Functional Requirements](#14-non-functional-requirements)

2. [Security](#2-security)

   * [2.1 Security Requirements](#21-security-requirements)
   * [2.2 Security Decisions](#22-security-decisions)

3. [Design Decisions](#3-design-decisions)

   * [3.1 Authorization Model](#31-authorization-model)
   * [3.2 Roles](#32-roles)
   * [3.3 Permissions](#33-permissions)
   * [3.4 Authorization Decision](#34-authorization-decision)
   * [3.5 Deny by Default](#35-deny-by-default)

4. [Data Model](#4-data-model)

   * [4.1 `authorization_role`](#41-authorization_role)
   * [4.2 `authorization_permission`](#42-authorization_permission)
   * [4.3 `authorization_role_permission`](#43-authorization_role_permission)
   * [4.4 `authorization_account_role`](#44-authorization_account_role)

5. [Use Cases](#5-use-cases)

   * [5.1 Authorize Action](#51-authorize-action)
   * [5.2 Assign Role](#52-assign-role)
   * [5.3 Revoke Role](#53-revoke-role)

6. [Permission Catalog](#6-permission-catalog)

7. [Role Catalog](#7-role-catalog)

8. [Integration Contract](#8-integration-contract)

   * [8.1 Authentication Domain](#81-authentication-domain)
   * [8.2 Account Domain](#82-account-domain)
   * [8.3 API Request Flow](#83-api-request-flow)

9. [Error Contract](#9-error-contract)

10. [Implementation Status](#10-implementation-status)

---

# 1. Requirements

## 1.1 Authorization

| ID             | Requirement                                                              |
| -------------- | ------------------------------------------------------------------------ |
| `AZ_REQ_FC_01` | Authorize an authenticated principal to perform an action.               |
| `AZ_REQ_FC_02` | Base authorization decisions on permissions.                             |
| `AZ_REQ_FC_03` | Obtain permissions through assigned roles.                               |
| `AZ_REQ_FC_04` | Support multiple roles for one account.                                  |
| `AZ_REQ_FC_05` | Support multiple permissions for one role.                               |
| `AZ_REQ_FC_06` | Deny access when the required permission is absent.                      |
| `AZ_REQ_FC_07` | Return an authorization decision to the protected application operation. |

## 1.2 RBAC

| ID             | Requirement                                                                                              |
| -------------- | -------------------------------------------------------------------------------------------------------- |
| `AZ_REQ_FC_08` | Roles must be assignable to accounts.                                                                    |
| `AZ_REQ_FC_09` | Permissions must be assignable to roles.                                                                 |
| `AZ_REQ_FC_10` | Accounts must not receive direct permissions outside a role.                                             |
| `AZ_REQ_FC_11` | A role assignment must be revocable.                                                                     |
| `AZ_REQ_FC_12` | Authorization must support the initial application role and permission catalog defined by this document. |
| `AZ_REQ_FC_13` | Role hierarchy is not required.                                                                          |
| `AZ_REQ_FC_14` | Separation-of-duty rules are not required unless explicitly added to this design.                        |

## 1.3 Integration

| ID             | Requirement                                                                            |
| -------------- | -------------------------------------------------------------------------------------- |
| `AZ_REQ_FC_15` | Authorization must consume `AuthenticatedPrincipal` from Authentication.               |
| `AZ_REQ_FC_16` | Authorization must not authenticate credentials.                                       |
| `AZ_REQ_FC_17` | Authorization must not validate passwords or bearer tokens.                            |
| `AZ_REQ_FC_18` | Authorization must not own account profile or credential data.                         |
| `AZ_REQ_FC_19` | Authorization must not execute Account business operations.                            |
| `AZ_REQ_FC_20` | Protected domain operations must perform authorization before executing the operation. |

## 1.4 Non-Functional Requirements

| ID                 | Requirement                                                                                                                           |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------- |
| `AZ_REQ_NON_FC_01` | Authorization must be enforced server-side.                                                                                           |
| `AZ_REQ_NON_FC_02` | Authorization must deny by default.                                                                                                   |
| `AZ_REQ_NON_FC_03` | Authorization decisions must not depend on client-supplied role or permission claims unless independently validated by Authorization. |
| `AZ_REQ_NON_FC_04` | Authorization failures must not expose internal role or permission configuration unnecessarily.                                       |
| `AZ_REQ_NON_FC_05` | Authorization logs must not contain access tokens, passwords, or equivalent authentication secrets.                                   |
| `AZ_REQ_NON_FC_06` | Authorization must produce deterministic decisions for the same principal, permission, and policy state.                              |

---

# 2. Security

## 2.1 Security Requirements

| ID              | Requirement                                                                                            |
| --------------- | ------------------------------------------------------------------------------------------------------ |
| `AZ_SEC_REQ_01` | An unauthenticated request must not reach an authorization decision as an authenticated principal.     |
| `AZ_SEC_REQ_02` | Authorization must require a valid `AuthenticatedPrincipal`.                                           |
| `AZ_SEC_REQ_03` | Authorization must evaluate permissions server-side.                                                   |
| `AZ_SEC_REQ_04` | Missing required permission must result in `403 Forbidden`.                                            |
| `AZ_SEC_REQ_05` | Authorization must not return `401 Unauthorized` for an authenticated principal that lacks permission. |
| `AZ_SEC_REQ_06` | Authorization must not trust a client-provided role name as proof of authorization.                    |
| `AZ_SEC_REQ_07` | Role assignments must be stored and evaluated from authoritative server-side state.                    |
| `AZ_SEC_REQ_08` | Authorization must use least privilege.                                                                |
| `AZ_SEC_REQ_09` | Authorization must deny by default.                                                                    |

## 2.2 Security Decisions

### 2.2.1 RBAC Model

| ID                   | Decision             | Definition                                       |
| -------------------- | -------------------- | ------------------------------------------------ |
| `AZ_SEC_DEC_RBAC_01` | Model                | Use Core RBAC.                                   |
| `AZ_SEC_DEC_RBAC_02` | Account → Role       | An account may have zero or more roles.          |
| `AZ_SEC_DEC_RBAC_03` | Role → Permission    | A role may have zero or more permissions.        |
| `AZ_SEC_DEC_RBAC_04` | Account → Permission | Accounts receive permissions only through roles. |
| `AZ_SEC_DEC_RBAC_05` | Role Hierarchy       | Not implemented.                                 |
| `AZ_SEC_DEC_RBAC_06` | Direct Permissions   | Not implemented.                                 |
| `AZ_SEC_DEC_RBAC_07` | Separation of Duty   | Not implemented.                                 |
| `AZ_SEC_DEC_RBAC_08` | Default              | No role means no permissions.                    |

### 2.2.2 Authorization Decision

| ID                    | Decision           | Definition                                                                                |
| --------------------- | ------------------ | ----------------------------------------------------------------------------------------- |
| `AZ_SEC_DEC_AUTHZ_01` | Input              | `AuthenticatedPrincipal` + required permission.                                           |
| `AZ_SEC_DEC_AUTHZ_02` | Evaluation         | Permit when the principal has the required permission through one or more assigned roles. |
| `AZ_SEC_DEC_AUTHZ_03` | Deny               | Deny when no assigned role grants the required permission.                                |
| `AZ_SEC_DEC_AUTHZ_04` | Default            | Deny when the permission is unknown or no policy matches.                                 |
| `AZ_SEC_DEC_AUTHZ_05` | Server Enforcement | Authorization is enforced on the server.                                                  |
| `AZ_SEC_DEC_AUTHZ_06` | Result             | Return `Allowed` or `Denied`.                                                             |

### 2.2.3 Failure

| ID                      | Decision           | Definition                                                     |
| ----------------------- | ------------------ | -------------------------------------------------------------- |
| `AZ_SEC_DEC_FAILURE_01` | Missing Principal  | Authentication handles the request and returns `401`.          |
| `AZ_SEC_DEC_FAILURE_02` | Missing Permission | Authorization returns `403`.                                   |
| `AZ_SEC_DEC_FAILURE_03` | Error Content Type | Use `application/problem+json`.                                |
| `AZ_SEC_DEC_FAILURE_04` | Cache              | Authorization failure responses use `Cache-Control: no-store`. |

---

# 3. Design Decisions

## 3.1 Authorization Model

```text
Account
   │
   │ assigned roles
   ▼
Role
   │
   │ grants permissions
   ▼
Permission
   │
   │ required by operation
   ▼
Authorization Decision
```

Authorization uses Core RBAC:

```text
Account ──< AccountRole >── Role ──< RolePermission >── Permission
```

No direct:

```text
Account ─── Permission
```

relationship is allowed.

## 3.2 Roles

A role represents a set of permissions required for a defined responsibility.

Initial roles:

| Role             | Purpose                        |
| ---------------- | ------------------------------ |
| `account_admin`  | Manage administrator accounts. |
| `account_viewer` | View administrator accounts.   |

Roles are stable policy definitions. Permissions are assigned to roles rather than individual accounts.

## 3.3 Permissions

Permission format:

```text
<resource>:<action>
```

Example:

```text
account:view
account:create
```

Permissions identify an operation, not a role.

## 3.4 Authorization Decision

Input:

```text
AuthenticatedPrincipal
Required Permission
```

Decision:

```text
if principal has permission
    ALLOWED
else
    DENIED
```

The authorization layer does not execute the requested business operation.

## 3.5 Deny by Default

The default decision is:

```text
DENY
```

A request is permitted only when an explicit role assignment grants the required permission. This follows the server-side and deny-by-default authorization model recommended by OWASP.

---

# 4. Data Model

## 4.1 `authorization_role`

**ID:** `AZ_DM_01`

| Column        | Type          | Null | Constraint |
| ------------- | ------------- | ---: | ---------- |
| `id`          | `uuid`        |   No | PK         |
| `name`        | `text`        |   No | UNIQUE     |
| `description` | `text`        |   No |            |
| `created_at`  | `timestamptz` |   No |            |
| `updated_at`  | `timestamptz` |   No |            |
| `disabled_at` | `timestamptz` |  Yes |            |

Role names:

```text
account_admin
account_viewer
```

## 4.2 `authorization_permission`

**ID:** `AZ_DM_02`

| Column        | Type          | Null | Constraint |
| ------------- | ------------- | ---: | ---------- |
| `id`          | `uuid`        |   No | PK         |
| `name`        | `text`        |   No | UNIQUE     |
| `description` | `text`        |   No |            |
| `created_at`  | `timestamptz` |   No |            |
| `updated_at`  | `timestamptz` |   No |            |

Permission names are defined in Section 6.

## 4.3 `authorization_role_permission`

**ID:** `AZ_DM_03`

| Column          | Type   | Null | Constraint                         |
| --------------- | ------ | ---: | ---------------------------------- |
| `role_id`       | `uuid` |   No | FK → `authorization_role.id`       |
| `permission_id` | `uuid` |   No | FK → `authorization_permission.id` |

Primary key:

```text
(role_id, permission_id)
```

## 4.4 `authorization_account_role`

**ID:** `AZ_DM_04`

| Column       | Type          | Null | Constraint                   |
| ------------ | ------------- | ---: | ---------------------------- |
| `account_id` | `uuid`        |   No | References `account.id`      |
| `role_id`    | `uuid`        |   No | FK → `authorization_role.id` |
| `created_at` | `timestamptz` |   No |                              |
| `created_by` | `uuid`        |  Yes | Authenticated actor          |

Primary key:

```text
(account_id, role_id)
```

One account may have multiple roles.

---

# 5. Use Cases

## 5.1 Authorize Action

**ID:** `AZ_UC_01`

| Item   | Definition              |
| ------ | ----------------------- |
| Actor  | Authenticated Principal |
| Input  | Required Permission     |
| Result | Allowed or Denied       |

### Rules

1. Obtain the principal from Authentication.
2. Load the principal's assigned roles.
3. Load permissions granted by those roles.
4. Compare the required permission.
5. Return `Allowed` when present.
6. Return `Denied` when absent.

## 5.2 Assign Role

**ID:** `AZ_UC_02`

| Item   | Definition           |
| ------ | -------------------- |
| Actor  | Authorized Principal |
| Input  | Account ID, Role     |
| Result | Role assigned        |

### Rules

1. Caller must have `authorization:role_assign`.
2. Target role must exist.
3. Target role must be enabled.
4. Account must exist.
5. Existing assignment must not be duplicated.
6. Record the assignment actor and timestamp.

## 5.3 Revoke Role

**ID:** `AZ_UC_03`

| Item   | Definition              |
| ------ | ----------------------- |
| Actor  | Authorized Principal    |
| Input  | Account ID, Role        |
| Result | Role assignment revoked |

### Rules

1. Caller must have `authorization:role_revoke`.
2. Role assignment must exist.
3. Remove the assignment.
4. Record the assignment-revocation actor and timestamp.

Authorization role-management operations must not bypass Account lifecycle invariants.

---

# 6. Permission Catalog

| Permission                  | Description                                  |
| --------------------------- | -------------------------------------------- |
| `account:view`              | View non-deleted administrator accounts.     |
| `account:view_deleted`      | View soft-deleted administrator accounts.    |
| `account:create`            | Create administrator accounts.               |
| `account:update`            | Update administrator account fields.         |
| `account:deactivate`        | Deactivate administrator accounts.           |
| `account:activate`          | Activate administrator accounts.             |
| `account:delete`            | Soft-delete administrator accounts.          |
| `account:restore`           | Restore soft-deleted administrator accounts. |
| `account:purge`             | Hard-delete administrator accounts.          |
| `account:change_email`      | Change administrator email credentials.      |
| `account:change_password`   | Change administrator password credentials.   |
| `authorization:role_assign` | Assign a role to an account.                 |
| `authorization:role_revoke` | Revoke a role from an account.               |

Permission names are the authorization contract consumed by protected domain operations.

---

# 7. Role Catalog

## 7.1 `account_viewer`

Permissions:

```text
account:view
account:view_deleted
```

## 7.2 `account_admin`

Permissions:

```text
account:view
account:view_deleted
account:create
account:update
account:deactivate
account:activate
account:delete
account:restore
account:purge
account:change_email
account:change_password
```

Role-management permissions are intentionally separate from account-management permissions.

No initial role is granted automatically by this document.

Initial role assignment must be established through a controlled bootstrap or administrative provisioning mechanism.

---

# 8. Integration Contract

## 8.1 Authentication Domain

Authentication provides:

```text
AuthenticatedPrincipal {
    account_id
    session_id
    authenticated_at
}
```

Authorization consumes this principal.

Authentication does not provide authorization decisions.

## 8.2 Account Domain

Account owns:

```text
account
account_credentials
account lifecycle
```

Authorization owns:

```text
roles
permissions
account-role assignments
```

Authorization references `account.id` but does not own Account data.

Account operations remain responsible for Account business rules.

Example:

```text
Authorization
    → caller may deactivate accounts

Account
    → target exists
    → target is active
    → target is not the last active administrator
    → perform deactivation
```

The definition of which role makes an account an administrator is owned by Authorization.

## 8.3 API Request Flow

```text
HTTP Request
    ↓
Authentication
    ↓
AuthenticatedPrincipal
    ↓
Authorization
    ↓
Required Permission
    ↓
Domain Handler
    ↓
Domain Service
```

Example:

```text
GET /admin/accounts/{id}
    ↓
Authentication
    ↓
AuthenticatedPrincipal
    ↓
Authorization
    ↓
account:view
    ↓
Account::View Account
```

### Account API Permission Contract

For `GET /admin/accounts`, the required permissions are:

| Request condition         | Required permissions                      |
| ------------------------- | ----------------------------------------- |
| `include_deleted = false` | `account:view`                            |
| `include_deleted = true`  | `account:view` and `account:view_deleted` |

Authorization MUST enforce the following rules:

1. Authorization MUST evaluate the required permissions against the authenticated principal's server-side role and permission assignments.
2. The request MUST be denied if any required permission is missing.
3. A denied authenticated request MUST return `403 Forbidden`, and the Account list operation MUST NOT execute.
4. Client-supplied roles, permissions, or authorization decisions MUST NOT affect the authorization result.
5. After authorization succeeds, the Account domain MUST apply the `include_deleted` filter and execute the Account list operation.

---

# 9. Error Contract

## 9.1 Forbidden

When authentication succeeded but authorization fails:

```text
HTTP/1.1 403 Forbidden
Content-Type: application/problem+json
Cache-Control: no-store
```

Example:

```json
{
  "type": "https://github.com/0n6k4v-Coder/digital-publication-book-library/problems/forbidden",
  "title": "Forbidden",
  "status": 403,
  "detail": "The authenticated principal is not authorized to perform this operation.",
  "code": "FORBIDDEN"
}
```

HTTP `403 Forbidden` is used when the server understands the request but refuses to fulfill it; credentials may be valid but insufficient for the requested operation.

## 9.2 Authentication vs Authorization

| Condition                            | Domain         | Status |
| ------------------------------------ | -------------- | -----: |
| Missing authentication               | Authentication |  `401` |
| Invalid authentication               | Authentication |  `401` |
| Expired authentication               | Authentication |  `401` |
| Revoked authentication               | Authentication |  `401` |
| Authenticated but missing permission | Authorization  |  `403` |

---

# 10. Implementation Status

## 10.1 Requirements

| ID                 | Status         | Reason                                                                                                                                                                                                                                      |
| ------------------ | -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `AZ_REQ_FC_01`     | 🟢 Implemented | Authorization evaluates an `AuthenticatedPrincipal` against a required permission through the Authorization service.                                                                                                                        |
| `AZ_REQ_FC_02`     | 🟢 Implemented | Authorization decisions are based on permission names supplied to the authorization service.                                                                                                                                                |
| `AZ_REQ_FC_03`     | 🟢 Implemented | Permissions are obtained through assigned roles using `authorization_account_role` and `authorization_role_permission`.                                                                                                                     |
| `AZ_REQ_FC_04`     | 🟢 Implemented | The account-role schema allows an account to have multiple role assignments.                                                                                                                                                                |
| `AZ_REQ_FC_05`     | 🟢 Implemented | The role-permission schema and seeded catalog allow multiple permissions for one role.                                                                                                                                                      |
| `AZ_REQ_FC_06`     | 🟢 Implemented | Missing permissions cause the Authorization service to return `403 Forbidden`.                                                                                                                                                              |
| `AZ_REQ_FC_07`     | 🟢 Implemented | The authorization result is returned to the protected operation through the authorization boundary as success or `403 Forbidden`.                                                                                                           |
| `AZ_REQ_FC_08`     | 🟡 Partial     | Account-role assignments are persisted in `authorization_account_role`, but no application-level role-assignment operation is implemented yet.                                                                                              |
| `AZ_REQ_FC_09`     | 🟢 Implemented | Permissions are persisted and assigned to roles through `authorization_role_permission`, including the seeded role catalog.                                                                                                                 |
| `AZ_REQ_FC_10`     | 🟢 Implemented | No direct account-permission relationship exists; permissions are granted only through roles.                                                                                                                                               |
| `AZ_REQ_FC_11`     | 🟡 Partial     | Role assignments can be removed at the data-model level, but no application-level role-revocation operation is implemented yet.                                                                                                             |
| `AZ_REQ_FC_12`     | 🟢 Implemented | The initial application roles and permissions defined by this document are created by the authorization migration.                                                                                                                          |
| `AZ_REQ_FC_13`     | 🟢 Implemented | No role-hierarchy mechanism exists in the implemented authorization model.                                                                                                                                                                  |
| `AZ_REQ_FC_14`     | 🟢 Implemented | No separation-of-duty mechanism exists, consistent with the current design.                                                                                                                                                                 |
| `AZ_REQ_FC_15`     | 🟢 Implemented | Authorization consumes `AuthenticatedPrincipal` from Authentication in the authorization boundary.                                                                                                                                          |
| `AZ_REQ_FC_16`     | 🟢 Implemented | Authorization does not authenticate credentials; Authentication establishes the principal before authorization.                                                                                                                             |
| `AZ_REQ_FC_17`     | 🟢 Implemented | Authorization does not validate passwords or bearer tokens; it consumes the principal established by Authentication.                                                                                                                        |
| `AZ_REQ_FC_18`     | 🟢 Implemented | Authorization owns roles, permissions, and assignments, not Account profile or credential data.                                                                                                                                             |
| `AZ_REQ_FC_19`     | 🟢 Implemented | Authorization does not execute Account business operations.                                                                                                                                                                                 |
| `AZ_REQ_FC_20`     | 🟡 Partial     | Authorization runs before Account Create Account and View Accounts execution, but the new authorization boundary is not yet wired into every protected Account endpoint; View Account still uses the legacy `AuthenticatedAdmin` extractor. |
| `AZ_REQ_NON_FC_01` | 🟢 Implemented | Authorization decisions are enforced server-side by the authorization boundary and Authorization service.                                                                                                                                   |
| `AZ_REQ_NON_FC_02` | 🟢 Implemented | `has_permission` returns false when no matching role grants the required permission, resulting in denial.                                                                                                                                   |
| `AZ_REQ_NON_FC_03` | 🟢 Implemented | Authorization uses server-side role and permission records and does not accept client-supplied role or permission claims.                                                                                                                   |
| `AZ_REQ_NON_FC_04` | 🟢 Implemented | Authorization failures return a generic `403 Forbidden` response without exposing role or permission configuration.                                                                                                                         |
| `AZ_REQ_NON_FC_05` | 🟢 Implemented | The implemented Authorization path does not log access tokens, passwords, or equivalent authentication secrets.                                                                                                                             |
| `AZ_REQ_NON_FC_06` | 🟢 Implemented | The same principal, permission, and server-side policy state produce a deterministic authorization result.                                                                                                                                  |

## 10.2 Security

| ID                      | Status         | Reason                                                                                                       |
| ----------------------- | -------------- | ------------------------------------------------------------------------------------------------------------ |
| `AZ_SEC_REQ_01`         | 🟢 Implemented | The authorization boundary establishes the authenticated principal before calling the Authorization service. |
| `AZ_SEC_REQ_02`         | 🟢 Implemented | The Authorization service requires an `AuthenticatedPrincipal`.                                              |
| `AZ_SEC_REQ_03`         | 🟢 Implemented | Permission evaluation is performed server-side through the Authorization repository.                         |
| `AZ_SEC_REQ_04`         | 🟢 Implemented | Missing required permission results in `AppError::Forbidden` and HTTP `403`.                                 |
| `AZ_SEC_REQ_05`         | 🟢 Implemented | An authenticated principal without the required permission receives `403`, not `401`.                        |
| `AZ_SEC_REQ_06`         | 🟢 Implemented | The implementation does not accept client-supplied role names as authorization proof.                        |
| `AZ_SEC_REQ_07`         | 🟢 Implemented | Role assignments and role-permission mappings are stored and evaluated from authoritative database state.    |
| `AZ_SEC_REQ_08`         | 🟢 Implemented | Authorization checks the specific required permission and does not grant access from authentication alone.   |
| `AZ_SEC_REQ_09`         | 🟢 Implemented | The authorization query denies when no matching permission is found.                                         |
| `AZ_SEC_DEC_RBAC_01`    | 🟢 Implemented | The implemented model uses Account → Role → Permission Core RBAC relationships.                              |
| `AZ_SEC_DEC_RBAC_02`    | 🟢 Implemented | `authorization_account_role` allows an account to have zero or more roles.                                   |
| `AZ_SEC_DEC_RBAC_03`    | 🟢 Implemented | `authorization_role_permission` allows a role to have zero or more permissions.                              |
| `AZ_SEC_DEC_RBAC_04`    | 🟢 Implemented | There is no direct account-permission table; permissions are granted through roles.                          |
| `AZ_SEC_DEC_RBAC_05`    | 🟢 Implemented | Role hierarchy is intentionally not implemented by the current design.                                       |
| `AZ_SEC_DEC_RBAC_06`    | 🟢 Implemented | Direct permissions are intentionally not implemented; permissions are role-based.                            |
| `AZ_SEC_DEC_RBAC_07`    | 🟢 Implemented | Separation-of-duty rules are intentionally not implemented by the current design.                            |
| `AZ_SEC_DEC_RBAC_08`    | 🟢 Implemented | An account with no matching role-permission mapping receives no authorization permission.                    |
| `AZ_SEC_DEC_AUTHZ_01`   | 🟢 Implemented | The authorization service accepts `AuthenticatedPrincipal` and a required permission.                        |
| `AZ_SEC_DEC_AUTHZ_02`   | 🟢 Implemented | Authorization permits when a role assigned to the principal grants the required permission.                  |
| `AZ_SEC_DEC_AUTHZ_03`   | 🟢 Implemented | Authorization denies when no assigned role grants the required permission.                                   |
| `AZ_SEC_DEC_AUTHZ_04`   | 🟢 Implemented | Unknown or unmatched permissions result in `403 Forbidden` because the permission lookup returns false.      |
| `AZ_SEC_DEC_AUTHZ_05`   | 🟢 Implemented | Authorization is enforced by server-side authorization boundary and service code.                            |
| `AZ_SEC_DEC_AUTHZ_06`   | 🟢 Implemented | The Authorization service produces an allowed result or `AppError::Forbidden`.                               |
| `AZ_SEC_DEC_FAILURE_01` | 🟢 Implemented | Authentication establishes the principal first and returns `401` when authentication fails.                  |
| `AZ_SEC_DEC_FAILURE_02` | 🟢 Implemented | Missing authorization permission produces `403 Forbidden`.                                                   |
| `AZ_SEC_DEC_FAILURE_03` | 🟢 Implemented | Authorization errors use the shared `application/problem+json` error response.                               |
| `AZ_SEC_DEC_FAILURE_04` | 🟢 Implemented | Authorization error responses use `Cache-Control: no-store`.                                                 |

## 10.3 Data Model

| ID         | Description                     | Status         | Reason                                                                                                                                |
| ---------- | ------------------------------- | -------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| `AZ_DM_01` | `authorization_role`            | 🟢 Implemented | The `authorization_role` table is implemented with UUID identity, unique role names, descriptions, timestamps, and `disabled_at`.     |
| `AZ_DM_02` | `authorization_permission`      | 🟢 Implemented | The `authorization_permission` table is implemented with UUID identity, unique permission names, descriptions, and timestamps.        |
| `AZ_DM_03` | `authorization_role_permission` | 🟢 Implemented | The role-permission join table is implemented with a composite primary key and foreign keys to roles and permissions.                 |
| `AZ_DM_04` | `authorization_account_role`    | 🟢 Implemented | The account-role join table is implemented with a composite primary key, Account and Role foreign keys, timestamps, and `created_by`. |

## 10.4 Use Cases

| ID         | Description      | Status             | Reason                                                                                                                                                                                                                           |
| ---------- | ---------------- | ------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `AZ_UC_01` | Authorize Action | 🟡 Partial         | Generic permission evaluation is implemented and wired into Create Account and View Accounts, including conditional `account:view_deleted` authorization, but the new authorization boundary is not yet applied to View Account. |
| `AZ_UC_02` | Assign Role      | 🔴 Not Implemented | No application-level role-assignment operation is implemented.                                                                                                                                                                   |
| `AZ_UC_03` | Revoke Role      | 🔴 Not Implemented | No application-level role-revocation operation is implemented.                                                                                                                                                                   |

## 10.5 Permission Catalog

| Permission                  | Status         | Reason                                                                                                                    |
| --------------------------- | -------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `account:view`              | 🟢 Implemented | Permission is created by the authorization migration and actively evaluated by the View Accounts authorization boundary.  |
| `account:view_deleted`      | 🟢 Implemented | Permission is created by the authorization migration and actively evaluated when `include_deleted=true`.                  |
| `account:create`            | 🟢 Implemented | Permission is created by the authorization migration and actively evaluated by the Create Account authorization boundary. |
| `account:update`            | 🟢 Implemented | Permission is created by the authorization migration and assigned to `account_admin`.                                     |
| `account:deactivate`        | 🟢 Implemented | Permission is created by the authorization migration and assigned to `account_admin`.                                     |
| `account:activate`          | 🟢 Implemented | Permission is created by the authorization migration and assigned to `account_admin`.                                     |
| `account:delete`            | 🟢 Implemented | Permission is created by the authorization migration and assigned to `account_admin`.                                     |
| `account:restore`           | 🟢 Implemented | Permission is created by the authorization migration and assigned to `account_admin`.                                     |
| `account:purge`             | 🟢 Implemented | Permission is created by the authorization migration and assigned to `account_admin`.                                     |
| `account:change_email`      | 🟢 Implemented | Permission is created by the authorization migration and assigned to `account_admin`.                                     |
| `account:change_password`   | 🟢 Implemented | Permission is created by the authorization migration and assigned to `account_admin`.                                     |
| `authorization:role_assign` | 🟢 Implemented | Permission is created by the authorization migration as part of the authorization catalog.                                |
| `authorization:role_revoke` | 🟢 Implemented | Permission is created by the authorization migration as part of the authorization catalog.                                |

## 10.6 Integration

| ID          | Description                                       | Status         | Reason                                                                                                                                                                                                                    |
| ----------- | ------------------------------------------------- | -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `AZ_INT_01` | Authentication → Authorization principal contract | 🟢 Implemented | Authorization consumes `AuthenticatedPrincipal` directly from Authentication before evaluating the required permission.                                                                                                   |
| `AZ_INT_02` | Account → Authorization account reference         | 🟢 Implemented | Authorization role assignments reference `account.id`, and permission evaluation uses the authenticated `account_id`.                                                                                                     |
| `AZ_INT_03` | Authorization middleware/check boundary           | 🟡 Partial     | The Authentication → Authorization boundary is wired for Create Account and View Accounts, including the conditional `account:view_deleted` check, but View Account still uses the legacy `AuthenticatedAdmin` extractor. |

---

# Domain Boundary

```text
ACCOUNT
  Owns:
    account
    account_credentials
    account lifecycle

AUTHENTICATION
  Owns:
    credential verification
    sessions
    access tokens
    refresh tokens
    AuthenticatedPrincipal

AUTHORIZATION
  Owns:
    roles
    permissions
    account-role assignments
    authorization decisions
```

```text
Authentication
    "WHO IS THE CALLER?"

Authorization
    "WHAT MAY THE CALLER DO?"

Account / Business Domain
    "WHAT HAPPENS WHEN THE ACTION IS ALLOWED?"
```
