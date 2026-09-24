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
   * [3.6 Revocation Audit](#36-revocation-audit)

4. [Data Model](#4-data-model)

   * [4.1 `authorization_role`](#41-authorization_role)
   * [4.2 `authorization_permission`](#42-authorization_permission)
   * [4.3 `authorization_role_permission`](#43-authorization_role_permission)
   * [4.4 `authorization_account_role`](#44-authorization_account_role)
   * [4.5 `authorization_role_revocation_audit`](#45-authorization_role_revocation_audit)

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
   * [8.4 Role Revocation Audit](#84-role-revocation-audit)

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

## 3.6 Revocation Audit

Every successful role revocation must produce a durable Authorization audit record.

The authoritative revocation history is stored in:

```text
authorization_role_revocation_audit
```

The audit record is part of the same database transaction as the role-assignment deletion.

The authoritative record rules are:

1. A successful `AZ_UC_03` operation must create exactly one revocation-audit record for the revoked role assignment.
2. The audit record must contain the target Account identifier, revoked Role identifier, authenticated revocation actor identifier, and database-generated revocation timestamp.
3. `revoked_by` must be taken from the Authentication-provided `AuthenticatedPrincipal.account_id`.
4. `revoked_by` must never be accepted from the HTTP request body, query parameters, path parameters, client-supplied role claims, or client-supplied permission claims.
5. `revoked_at` must be generated by PostgreSQL and must not be supplied by the client.
6. The audit record must be inserted and the role assignment must be deleted inside the same database transaction.
7. The transaction must not commit the assignment deletion unless the revocation-audit record has also been inserted successfully.
8. If audit-record persistence fails, the role assignment deletion must be rolled back.
9. If role-assignment deletion fails, the audit-record insertion must be rolled back.
10. Application logs may additionally emit a structured security event, but application logs are supplementary and are not the authoritative revocation record.
11. Authorization application code must not update or delete revocation-audit records as part of `AZ_UC_03`.
12. Revocation-audit records are historical records and must not be removed automatically when the target Account, revoking Account, Role, or role assignment is deleted.
13. Revocation-audit records must not contain passwords, password hashes, bearer tokens, refresh tokens, session credentials, or other authentication secrets.

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

| Column       | Type          | Null | Constraint                                          |
| ------------ | ------------- | ---: | --------------------------------------------------- |
| `account_id` | `uuid`        |   No | FK → `account.id`; `ON DELETE CASCADE`              |
| `role_id`    | `uuid`        |   No | FK → `authorization_role.id`; `ON DELETE NO ACTION` |
| `created_at` | `timestamptz` |   No |                                                     |
| `created_by` | `uuid`        |  Yes | FK → `account.id`; `ON DELETE SET NULL`             |

Primary key:

```text
(account_id, role_id)
```

Rules:

1. One role assignment belongs to exactly one Account.
2. An Account may have multiple roles.
3. The role assignment cannot survive deletion of its target Account.
4. Hard deletion of an Account therefore deletes all `authorization_account_role` rows where `account_id` equals the deleted Account through `ON DELETE CASCADE`.
5. `created_by` is optional audit metadata and does not determine whether the role assignment belongs to the target Account.
6. When an Account that acted as `created_by` is hard-deleted, `created_by` becomes `NULL` on surviving role-assignment rows belonging to other Accounts.
7. Deleting an Account must not delete `authorization_role` or `authorization_permission` definitions.
8. The `role_id` foreign key therefore does not cascade from Account deletion.
9. All referential actions caused by Account hard deletion execute inside the same database transaction as the Account deletion.
10. Revocation history is not stored in this relation because deleting the assignment would otherwise remove the historical revocation record.
11. Successful assignment revocation is audited in `authorization_role_revocation_audit` as defined by `AZ_DM_05`.

## 4.5 `authorization_role_revocation_audit`

**ID:** `AZ_DM_05`

| Column       | Type          | Null | Constraint |
| ------------ | ------------- | ---: | ---------- |
| `id`         | `uuid`        |   No | PK, default `uuidv7()` |
| `account_id` | `uuid`        |   No | Historical target Account identifier |
| `role_id`    | `uuid`        |   No | Historical revoked Role identifier |
| `revoked_by` | `uuid`        |   No | Historical authenticated actor Account identifier |
| `revoked_at` | `timestamptz` |   No | Database-generated timestamp |

The authoritative SQL schema is:

```sql
CREATE TABLE authorization_role_revocation_audit (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    account_id UUID NOT NULL,
    role_id UUID NOT NULL,
    revoked_by UUID NOT NULL,
    revoked_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

Audit-record rules:

1. One row represents one successful role-revocation event.
2. Reassigning the same Role and revoking it again creates a new audit row.
3. `account_id` identifies the Account from which the Role assignment was revoked.
4. `role_id` identifies the Role that was revoked.
5. `revoked_by` identifies the authenticated principal that performed the revocation.
6. `revoked_at` is generated by PostgreSQL and represents the authoritative revocation event timestamp.
7. `account_id`, `role_id`, and `revoked_by` are historical identifiers and are intentionally not foreign keys.
8. The audit table must not use `ON DELETE CASCADE`, `ON DELETE SET NULL`, or other referential actions to remove or rewrite historical audit records when Accounts or Roles are deleted.
9. Account hard deletion therefore removes the corresponding `authorization_account_role` rows but does not remove `authorization_role_revocation_audit` rows.
10. Deletion of the Account that performed a revocation does not rewrite `revoked_by`.
11. Deletion of a Role definition does not rewrite `role_id`.
12. The audit record is append-only from the Authorization application layer.
13. `AZ_UC_03` must not update or delete an existing audit record.
14. Audit-record retention, archival, or controlled administrative disposal is outside `AZ_UC_03` and must be defined by a separate policy before implementation.
15. The audit record must contain only information required to establish the revocation event and must not contain authentication secrets.

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
2. The authenticated principal must be obtained from Authentication.
3. Target role assignment must exist.
4. The role-management operation must not use client-supplied role or permission claims as proof of authorization.
5. If the target Role is `account_admin`, the operation must first acquire the shared `account_administrator_invariant_lock`.
6. If the target Role is `account_admin`, the operation must evaluate the current active, non-deleted administrator state after acquiring the invariant lock.
7. Revoking `account_admin` from an active, non-deleted administrator must fail with `LAST_ACTIVE_ADMINISTRATOR` when the revocation would leave no active, non-deleted administrator.
8. The revocation must execute inside one database transaction.
9. The transaction must verify the role assignment before changing persistent state.
10. The transaction must create one `authorization_role_revocation_audit` record before successful commit.
11. `revoked_by` must be the authenticated principal's `account_id`.
12. `revoked_at` must be generated by PostgreSQL.
13. The role assignment deletion and revocation-audit insertion must commit atomically.
14. If either the role-assignment deletion or audit-record insertion fails, the complete transaction must roll back.
15. The operation must not return passwords, password hashes, access tokens, refresh tokens, session credentials, or other authentication secrets.
16. Structured application logging may be emitted as a supplementary security event, but it is not a substitute for `authorization_role_revocation_audit`.
17. Authorization role-management operations must not bypass Account lifecycle invariants.

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

### Account Hard-Delete Integration Contract

When `AC_UC_09` physically deletes an Account:

```text
Account
    ↓
DELETE account row
    ↓
authorization_account_role.account_id
    ON DELETE CASCADE
    ↓
all role assignments for the deleted Account are removed
```

At the same time, for role assignments belonging to surviving Accounts:

```text
authorization_account_role.created_by
    ON DELETE SET NULL
```

means:

```text
deleted creator Account
    ↓
surviving role assignment remains
    ↓
created_by becomes NULL
```

This distinction is authoritative:

```text
account_id
    = ownership of the role assignment
    = CASCADE

created_by
    = optional historical actor reference
    = SET NULL
```

Account hard deletion must not delete roles, permissions, or role assignments belonging to other surviving Accounts.

Authorization does not need a separate application-level cleanup call after Account deletion. The database referential actions provide the required cleanup atomically within the Account transaction.

If the Account transaction rolls back, the Authorization role assignments and creator references remain unchanged. If it commits, no role assignment belonging to the deleted Account remains. SQLx provides the transaction boundary needed for this atomic behavior. ([Docs.rs][2])

### Authorization Security Boundary

Authorization continues to evaluate access server-side using the authenticated principal and current server-side role/permission state. It must not trust client-supplied roles or permissions. This remains consistent with the OWASP least-privilege and deny-by-default authorization model. ([OWASP Cheat Sheet Series][3])

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

## 8.4 Role Revocation Audit

The Authentication, Authorization, and Account domains use the following revocation-audit contract.

### Authentication → Authorization

Authentication provides:

```text
AuthenticatedPrincipal {
    account_id
    session_id
    authenticated_at
}
```

Authorization uses:

```text
AuthenticatedPrincipal.account_id
```

as the authoritative `revoked_by` value.

The HTTP request must not provide the revocation actor.

### Authorization Persistence

A successful `AZ_UC_03` operation performs:

```text
Begin database transaction
    ↓
Acquire account_administrator_invariant_lock
    when revoking account_admin
    ↓
Load and verify the target role assignment
    ↓
Evaluate account administrator invariant
    when revoking account_admin
    ↓
Insert authorization_role_revocation_audit
    ↓
Delete authorization_account_role
    ↓
Commit transaction
```

The transaction must satisfy:

```text
commit(revocation audit + assignment deletion)
```

or:

```text
rollback(revocation audit + assignment deletion)
```

There must be no durable state where the role assignment is deleted without its corresponding revocation-audit record.

There must be no durable state where a revocation-audit record exists for a role assignment that was not successfully revoked.

### Account Hard Delete

Account hard deletion removes:

```text
authorization_account_role.account_id
```

through the existing `ON DELETE CASCADE` relationship.

Account hard deletion must not remove:

```text
authorization_role_revocation_audit
```

records.

The historical values of:

```text
authorization_role_revocation_audit.account_id
authorization_role_revocation_audit.role_id
authorization_role_revocation_audit.revoked_by
```

remain unchanged after Account or Role deletion.

This is intentional because revocation-audit records represent historical Authorization events rather than current ownership relationships.

### Structured Application Logging

Structured application logging may record the same successful revocation event for security monitoring and incident investigation.

Application logging is supplementary.

The authoritative application record is:

```text
authorization_role_revocation_audit
```

A missing or unavailable application log does not make a committed revocation unauthoritative.

A failed write to `authorization_role_revocation_audit` must prevent the revocation transaction from committing.

### Security Boundary

No client-provided value may determine:

```text
revoked_by
revoked_at
```

or the authorization decision.

The Authorization domain continues to evaluate permission using the authenticated principal and server-side role and permission state.

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

| ID                 | Status         | Reason                                                                                                                                               |
| ------------------ | -------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `AZ_REQ_FC_01`     | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_FC_02`     | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_FC_03`     | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_FC_04`     | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_FC_05`     | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_FC_06`     | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_FC_07`     | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_FC_08`     | 🟡 Partial     | Account-role assignments are persisted in `authorization_account_role`, but no application-level role-assignment operation is implemented yet.       |
| `AZ_REQ_FC_09`     | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_FC_10`     | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_FC_11`     | 🟡 Partial     | Role assignments can be removed automatically by Account hard-delete cascade, but no application-level role-revocation operation is implemented yet. |
| `AZ_REQ_FC_12`     | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_FC_13`     | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_FC_14`     | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_FC_15`     | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_FC_16`     | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_FC_17`     | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_FC_18`     | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_FC_19`     | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_FC_20`     | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_NON_FC_01` | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_NON_FC_02` | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_NON_FC_03` | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_NON_FC_04` | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_NON_FC_05` | 🟢 Implemented |                                                                                                                                                      |
| `AZ_REQ_NON_FC_06` | 🟢 Implemented |                                                                                                                                                      |

## 10.2 Security

| ID                      | Status         | Reason |
| ----------------------- | -------------- | ------ |
| `AZ_SEC_REQ_01`         | 🟢 Implemented |        |
| `AZ_SEC_REQ_02`         | 🟢 Implemented |        |
| `AZ_SEC_REQ_03`         | 🟢 Implemented |        |
| `AZ_SEC_REQ_04`         | 🟢 Implemented |        |
| `AZ_SEC_REQ_05`         | 🟢 Implemented |        |
| `AZ_SEC_REQ_06`         | 🟢 Implemented |        |
| `AZ_SEC_REQ_07`         | 🟢 Implemented |        |
| `AZ_SEC_REQ_08`         | 🟢 Implemented |        |
| `AZ_SEC_REQ_09`         | 🟢 Implemented |        |
| `AZ_SEC_DEC_RBAC_01`    | 🟢 Implemented |        |
| `AZ_SEC_DEC_RBAC_02`    | 🟢 Implemented |        |
| `AZ_SEC_DEC_RBAC_03`    | 🟢 Implemented |        |
| `AZ_SEC_DEC_RBAC_04`    | 🟢 Implemented |        |
| `AZ_SEC_DEC_RBAC_05`    | 🟢 Implemented |        |
| `AZ_SEC_DEC_RBAC_06`    | 🟢 Implemented |        |
| `AZ_SEC_DEC_RBAC_07`    | 🟢 Implemented |        |
| `AZ_SEC_DEC_RBAC_08`    | 🟢 Implemented |        |
| `AZ_SEC_DEC_AUTHZ_01`   | 🟢 Implemented |        |
| `AZ_SEC_DEC_AUTHZ_02`   | 🟢 Implemented |        |
| `AZ_SEC_DEC_AUTHZ_03`   | 🟢 Implemented |        |
| `AZ_SEC_DEC_AUTHZ_04`   | 🟢 Implemented |        |
| `AZ_SEC_DEC_AUTHZ_05`   | 🟢 Implemented |        |
| `AZ_SEC_DEC_AUTHZ_06`   | 🟢 Implemented |        |
| `AZ_SEC_DEC_FAILURE_01` | 🟢 Implemented |        |
| `AZ_SEC_DEC_FAILURE_02` | 🟢 Implemented |        |
| `AZ_SEC_DEC_FAILURE_03` | 🟢 Implemented |        |
| `AZ_SEC_DEC_FAILURE_04` | 🟢 Implemented |        |

## 10.3 Data Model

| ID         | Description                            | Status         | Reason |
| ---------- | -------------------------------------- | -------------- | ------ |
| `AZ_DM_01` | `authorization_role`                   | 🟢 Implemented |        |
| `AZ_DM_02` | `authorization_permission`             | 🟢 Implemented |        |
| `AZ_DM_03` | `authorization_role_permission`        | 🟢 Implemented |        |
| `AZ_DM_04` | `authorization_account_role`           | 🟢 Implemented |        |
| `AZ_DM_05` | `authorization_role_revocation_audit`  | 🔴 Not Implemented | Required by `AZ_UC_03` to persist the authoritative revocation actor and timestamp transactionally with assignment deletion. |

## 10.4 Use Cases

| ID         | Description      | Status             | Reason |
| ---------- | ---------------- | ------------------ | ------ |
| `AZ_UC_01` | Authorize Action | 🟢 Implemented     |        |
| `AZ_UC_02` | Assign Role      | 🔴 Not Implemented |        |
| `AZ_UC_03` | Revoke Role      | 🔴 Not Implemented |        |

## 10.5 Permission Catalog

| Permission                  | Status         | Reason |
| --------------------------- | -------------- | ------ |
| `account:view`              | 🟢 Implemented |        |
| `account:view_deleted`      | 🟢 Implemented |        |
| `account:create`            | 🟢 Implemented |        |
| `account:update`            | 🟢 Implemented |        |
| `account:deactivate`        | 🟢 Implemented |        |
| `account:activate`          | 🟢 Implemented |        |
| `account:delete`            | 🟢 Implemented |        |
| `account:restore`           | 🟢 Implemented |        |
| `account:purge`             | 🟢 Implemented |        |
| `account:change_email`      | 🟢 Implemented |        |
| `account:change_password`   | 🟢 Implemented |        |
| `authorization:role_assign` | 🟢 Implemented |        |
| `authorization:role_revoke` | 🟢 Implemented |        |

## 10.6 Integration

| ID          | Description                                                   | Status         | Reason |
| ----------- | ------------------------------------------------------------- | -------------- | ------ |
| `AZ_INT_01` | Authentication → Authorization principal contract             | 🟢 Implemented |        |
| `AZ_INT_02` | Account → Authorization account reference                     | 🟢 Implemented |        |
| `AZ_INT_03` | Authorization middleware/check boundary for protected actions | 🟢 Implemented |        |
| `AZ_INT_04` | Role revocation audit, Account hard-delete preservation, and atomic assignment-revocation persistence | 🔴 Not Implemented | Required by `AZ_UC_03`; the authoritative audit persistence contract is defined but not implemented yet. |

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

The business domain answers:

```text
WHAT HAPPENS WHEN ACCESS IS ALLOWED?
```

[1]: https://www.postgresql.org/docs/18/sql-createtable.html "PostgreSQL 18: CREATE TABLE"
[2]: https://docs.rs/sqlx/latest/sqlx/struct.Transaction.html "SQLx 0.9: Transaction"
[3]: https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html "OWASP Authorization Cheat Sheet"
[4]: https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html "OWASP Logging Cheat Sheet"
[5]: https://csrc.nist.gov/pubs/sp/800/53/r5/upd1/final "NIST SP 800-53 Rev. 5, Release 5.2.0"

