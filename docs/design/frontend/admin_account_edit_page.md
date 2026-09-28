# Frontend Admin Account Edit Page

## Table of Contents

1. [Scope](#1-scope)
2. [Routes](#2-routes)
3. [Requirements](#3-requirements)
4. [Page Structure](#4-page-structure)
5. [Account Data Contract](#5-account-data-contract)
6. [Account Field Editing](#6-account-field-editing)
7. [Change Email](#7-change-email)
8. [Change Password](#8-change-password)
9. [Edit Page Action Boundary](#9-edit-page-action-boundary)
10. [API Contract](#10-api-contract)
11. [Validation and Error States](#11-validation-and-error-states)
12. [UI States](#12-ui-states)
13. [Responsive Layout](#13-responsive-layout)
14. [Security](#14-security)
15. [Accessibility](#15-accessibility)
16. [Component Structure](#16-component-structure)
17. [Testing](#17-testing)
18. [Implementation Criteria](#18-implementation-criteria)
19. [Traceability](#19-traceability)
20. [Standards and Technology References](#20-standards-and-technology-references)

---

# 1. Scope

This document defines the protected **Administrator Account Edit** frontend page.

The canonical route is:

```text
/admin/accounts/:id/edit
```

The page is rendered inside the existing [Admin Shell](./admin_shell.md).

The page allows an authorized administrator to edit the supported Account fields and credentials through the authoritative Account API.

The page owns:

* Display name editing.
* Administrator email change.
* Administrator password change.
* Validation presentation.
* Submission state.
* Server error presentation.
* Return navigation to Account Management.

The page does not own:

* Account creation.
* Account viewing as a read-only Detail page.
* Account deactivation.
* Account activation.
* Account restoration.
* Account soft deletion.
* Account hard deletion.
* Delete confirmation.
* Role assignment.
* Role revocation.
* Authentication implementation.
* Authorization policy implementation.
* Password hashing.
* Email normalization persistence.
* Account persistence or lifecycle invariants.

Soft-deleted Accounts are excluded from normal Account operations by the backend contract. A soft-deleted Account therefore cannot be edited through this page.

## Responsibilities

| ID                          | Responsibility                                                                                     |
| --------------------------- | -------------------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-EDIT-SCOPE-001` | Load one administrator Account by ID.                                                              |
| `FE-ACCOUNT-EDIT-SCOPE-002` | Display the authoritative Account representation.                                                  |
| `FE-ACCOUNT-EDIT-SCOPE-003` | Edit the supported `display_name` field.                                                           |
| `FE-ACCOUNT-EDIT-SCOPE-004` | Change the administrator email through the dedicated Account API.                                  |
| `FE-ACCOUNT-EDIT-SCOPE-005` | Change the administrator password through the dedicated Account API.                               |
| `FE-ACCOUNT-EDIT-SCOPE-006` | Preserve originating Account List return navigation.                                               |
| `FE-ACCOUNT-EDIT-SCOPE-007` | Handle authentication, authorization, validation, conflict, not-found, server, and network errors. |
| `FE-ACCOUNT-EDIT-SCOPE-008` | Prevent duplicate submissions for each edit operation.                                             |
| `FE-ACCOUNT-EDIT-SCOPE-009` | Provide keyboard-accessible and responsive form interaction.                                       |
| `FE-ACCOUNT-EDIT-SCOPE-010` | Preserve Admin Shell authentication and navigation behavior.                                       |

## Out of Scope

| ID                        | Excluded Area                          |
| ------------------------- | -------------------------------------- |
| `FE-ACCOUNT-EDIT-OOS-001` | Authentication implementation.         |
| `FE-ACCOUNT-EDIT-OOS-002` | Authorization implementation.          |
| `FE-ACCOUNT-EDIT-OOS-003` | Account creation.                      |
| `FE-ACCOUNT-EDIT-OOS-004` | Account lifecycle mutations.           |
| `FE-ACCOUNT-EDIT-OOS-005` | Account deletion and purge.            |
| `FE-ACCOUNT-EDIT-OOS-006` | Role assignment and revocation.        |
| `FE-ACCOUNT-EDIT-OOS-007` | Password hashing implementation.       |
| `FE-ACCOUNT-EDIT-OOS-008` | Email normalization persistence.       |
| `FE-ACCOUNT-EDIT-OOS-009` | Database transactions and persistence. |
| `FE-ACCOUNT-EDIT-OOS-010` | Client-side authorization enforcement. |

---

# 2. Routes

| ID                          | Route                      | Access                       | Behavior                                                                 | References                                         |
| --------------------------- | -------------------------- | ---------------------------- | ------------------------------------------------------------------------ | -------------------------------------------------- |
| `FE-ACCOUNT-EDIT-ROUTE-001` | `/admin/accounts/:id/edit` | Authenticated                | Render Account Edit page.                                                | `PAGE-ADM-010`, `ADM-AUTH-005`                     |
| `FE-ACCOUNT-EDIT-ROUTE-002` | `/admin/accounts/:id/edit` | Bootstrap                    | Keep route pending until authentication resolves.                        | Authentication domain                              |
| `FE-ACCOUNT-EDIT-ROUTE-003` | `/admin/accounts/:id/edit` | Unauthenticated              | Redirect through shared Admin Shell authentication behavior to `/login`. | `ADM-AUTH-001`, Admin Shell                        |
| `FE-ACCOUNT-EDIT-ROUTE-004` | `/admin/accounts/:id/edit` | Authenticated + unauthorized | Preserve authentication state and render authorization error.            | Authorization domain                               |
| `FE-ACCOUNT-EDIT-ROUTE-005` | `/admin/accounts/:id/edit` | Invalid Account ID           | Render an invalid Account identifier error.                              | `AC_API_03`, `AC_API_04`, `AC_API_10`, `AC_API_11` |
| `FE-ACCOUNT-EDIT-ROUTE-006` | `/admin/accounts/:id/edit` | Unavailable Account          | Render Account Not Found state.                                          | `AC_API_03`                                        |

The route parameter is the Account UUID:

```text
/admin/accounts/:id/edit
```

Authentication tokens, refresh credentials, passwords, or other secrets must never appear in the route.

## Return Navigation

Account Edit may receive an encoded `return_to` query parameter:

```text
/admin/accounts/:id/edit?return_to=%2Fadmin%2Faccounts%3Fpage%3D2%26page_size%3D20%26status%3Dactive
```

Supported return destination:

```text
/admin/accounts
```

Supported Account List query parameters are:

```text
page
page_size
status
include_deleted
```

The frontend must:

1. Validate `return_to` before using it.
2. Require the destination to use the same browser origin.
3. Require the destination pathname to be `/admin/accounts`.
4. Permit only the supported Account List query parameters.
5. Reject malformed, external, unrelated, or unsupported destinations or query parameters.
6. Fall back to `/admin/accounts` when validation fails.
7. Never place credentials, access tokens, passwords, or other secrets in `return_to`.
8. Preserve only Account List navigation state defined by the Account Management page.

The return destination is navigation context only.

It is not authorization state.

## Route Relationships

```text
/admin/accounts
    │
    ├── View
    │    └── /admin/accounts/:id
    │
    └── Edit
         └── /admin/accounts/:id/edit
              ├── Display name
              ├── Change email
              └── Change password
```

Account lifecycle and deletion navigation remain outside Account Edit.

---

# 3. Requirements

| ID                        | Requirement                                                                                                     | Repository Reference                               |
| ------------------------- | --------------------------------------------------------------------------------------------------------------- | -------------------------------------------------- |
| `FE-ACCOUNT-EDIT-REQ-001` | Provide a protected Account Edit route.                                                                         | `ADM-AUTH-001`, `ADM-AUTH-005`, `PAGE-ADM-010`     |
| `FE-ACCOUNT-EDIT-REQ-002` | Load the Account using `GET /admin/accounts/{id}`.                                                              | `AC_UC_03`, `AC_API_03`                            |
| `FE-ACCOUNT-EDIT-REQ-003` | Display the authoritative Account representation.                                                               | Account API                                        |
| `FE-ACCOUNT-EDIT-REQ-004` | Support editing `display_name`.                                                                                 | `AC_UC_04`, `AC_API_04`                            |
| `FE-ACCOUNT-EDIT-REQ-005` | Support changing administrator email.                                                                           | `AC_UC_10`, `AC_API_10`                            |
| `FE-ACCOUNT-EDIT-REQ-006` | Support changing administrator password.                                                                        | `AC_UC_11`, `AC_API_11`                            |
| `FE-ACCOUNT-EDIT-REQ-007` | Preserve originating Account List navigation state.                                                             | Account Management frontend design                 |
| `FE-ACCOUNT-EDIT-REQ-008` | Do not provide lifecycle mutation controls on Account Edit.                                                     | Account Management frontend design                 |
| `FE-ACCOUNT-EDIT-REQ-009` | Do not provide delete or purge controls on Account Edit.                                                        | Account Detail / Account Delete frontend design    |
| `FE-ACCOUNT-EDIT-REQ-010` | Treat the backend as authoritative for validation.                                                              | Account API                                        |
| `FE-ACCOUNT-EDIT-REQ-011` | Treat `401 Unauthorized` as an authentication failure.                                                          | Authentication domain                              |
| `FE-ACCOUNT-EDIT-REQ-012` | Treat `403 Forbidden` as an authorization failure.                                                              | Authorization domain                               |
| `FE-ACCOUNT-EDIT-REQ-013` | Handle `404 ACCOUNT_NOT_FOUND` as unavailable Account state.                                                    | `AC_API_03`, `AC_API_04`, `AC_API_10`, `AC_API_11` |
| `FE-ACCOUNT-EDIT-REQ-014` | Handle `409 EMAIL_ALREADY_IN_USE` as an email conflict.                                                         | `AC_API_10`                                        |
| `FE-ACCOUNT-EDIT-REQ-015` | Handle `415 UNSUPPORTED_MEDIA_TYPE` for Account field updates.                                                  | `AC_API_04`                                        |
| `FE-ACCOUNT-EDIT-REQ-016` | Handle `422 VALIDATION_ERROR` for Account field updates.                                                        | `AC_API_04`, `AC_API_10`                           |
| `FE-ACCOUNT-EDIT-REQ-017` | Handle `422 PASSWORD_POLICY_VIOLATION` for password changes.                                                    | `AC_API_11`                                        |
| `FE-ACCOUNT-EDIT-REQ-018` | Prevent duplicate submission for each mutation form.                                                            | Frontend behavior                                  |
| `FE-ACCOUNT-EDIT-REQ-019` | Never persist passwords in browser storage.                                                                     | Account Security, Authentication domain            |
| `FE-ACCOUNT-EDIT-REQ-020` | Never place passwords in URLs.                                                                                  | Account Security                                   |
| `FE-ACCOUNT-EDIT-REQ-021` | Never log passwords, access tokens, refresh credentials, or Authorization headers.                              | Account Security, Authentication domain            |
| `FE-ACCOUNT-EDIT-REQ-022` | Provide visible focus and keyboard-accessible interaction.                                                      | Admin Shell accessibility                          |
| `FE-ACCOUNT-EDIT-REQ-023` | Provide accessible labels and error associations for all editable controls.                                     | WCAG 2.2                                           |
| `FE-ACCOUNT-EDIT-REQ-024` | Preserve Admin Shell behavior across supported viewport sizes.                                                  | Admin Shell                                        |
| `FE-ACCOUNT-EDIT-REQ-025` | Do not infer authorization from client-held roles or permissions.                                               | Authorization domain                               |
| `FE-ACCOUNT-EDIT-REQ-026` | Never place password values in browser history, query strings, navigation state, or persistent browser storage. | Security boundary                                  |

---

# 4. Page Structure

## 4.1 Page Header

Recommended structure:

```text
Back to Administrator Accounts

Edit Administrator Account
Library Administrator
admin@example.com
Active
```

| ID                       | Requirement                                                          |
| ------------------------ | -------------------------------------------------------------------- |
| `FE-ACCOUNT-EDIT-UI-001` | Use `Edit Administrator Account` as the primary page heading.        |
| `FE-ACCOUNT-EDIT-UI-002` | Display the current `display_name` when available.                   |
| `FE-ACCOUNT-EDIT-UI-003` | Display the current Account email.                                   |
| `FE-ACCOUNT-EDIT-UI-004` | Display Account lifecycle status as read-only information.           |
| `FE-ACCOUNT-EDIT-UI-005` | Provide a native link back to the validated `return_to` destination. |
| `FE-ACCOUNT-EDIT-UI-006` | Keep lifecycle and deletion actions outside this page.               |

The Edit page must not render:

```text
Deactivate
Activate
Restore
Delete
Permanently Delete
```

## 4.2 Account Information

The current Account remains visible while editing.

```text
Account Information

Display name
[Library Administrator                ]

Email
admin@example.com

Account ID
019...

Status
Active
```

Read-only metadata:

| ID                          | Field        | Source       | Editable |
| --------------------------- | ------------ | ------------ | -------- |
| `FE-ACCOUNT-EDIT-FIELD-001` | Account ID   | `id`         | No       |
| `FE-ACCOUNT-EDIT-FIELD-002` | Status       | `status`     | No       |
| `FE-ACCOUNT-EDIT-FIELD-003` | Created      | `created_at` | No       |
| `FE-ACCOUNT-EDIT-FIELD-004` | Last updated | `updated_at` | No       |

The frontend must never expose:

```text
password
password_hash
deleted_by
created_by
updated_by
```

as editable values.

## 4.3 Edit Sections

The page should use separate forms for separate backend operations:

```text
Account Information
    └── Display Name
         PATCH /admin/accounts/{id}

Change Email
    └── Email
         PATCH /admin/accounts/{id}/email

Change Password
    └── New password
         PATCH /admin/accounts/{id}/password
```

Each form owns its own:

```text
field state
dirty state
validation state
pending state
success state
error state
```

An operation must not submit unrelated form fields.

---

# 5. Account Data Contract

The Account Edit page consumes the authoritative Account response:

```json
{
  "id": "019...",
  "email": "admin@example.com",
  "display_name": "Library Administrator",
  "status": "active",
  "created_at": "2026-09-23T10:00:00Z",
  "updated_at": "2026-09-23T10:00:00Z",
  "deleted_at": null
}
```

| ID                         | Field          | Type                    | Frontend Rule                                            |
| -------------------------- | -------------- | ----------------------- | -------------------------------------------------------- |
| `FE-ACCOUNT-EDIT-DATA-001` | `id`           | UUID string             | Read-only identifier.                                    |
| `FE-ACCOUNT-EDIT-DATA-002` | `email`        | string                  | Editable only through Change Email.                      |
| `FE-ACCOUNT-EDIT-DATA-003` | `display_name` | string or null          | Editable through Account Update.                         |
| `FE-ACCOUNT-EDIT-DATA-004` | `status`       | `active` or `inactive`  | Read-only on this page.                                  |
| `FE-ACCOUNT-EDIT-DATA-005` | `created_at`   | RFC 3339 string         | Read-only timestamp.                                     |
| `FE-ACCOUNT-EDIT-DATA-006` | `updated_at`   | RFC 3339 string         | Read-only timestamp.                                     |
| `FE-ACCOUNT-EDIT-DATA-007` | `deleted_at`   | RFC 3339 string or null | Expected to be `null` for a successful normal Edit load. |

The frontend must not fabricate values.

The server response is authoritative.

Unknown response fields must not be rendered automatically.

Password and password hash data must never be expected in the Account response.

Account API responses use:

```text
Cache-Control: no-store
```

---

# 6. Account Field Editing

## 6.1 Supported Field

AC_UC_04 currently supports exactly one mutable Account entity field:

```text
display_name
```

The following fields are not editable through this operation:

```text
id
created_at
created_by
updated_at
updated_by
status
deleted_at
deleted_by
email
password_hash
```

Email and password have dedicated operations.

## 6.2 Form

```text
Display name

[Library Administrator                    ]

Leave empty to clear the display name.

[Save changes]
```

| ID                            | Requirement                                                                         |
| ----------------------------- | ----------------------------------------------------------------------------------- |
| `FE-ACCOUNT-EDIT-DISPLAY-001` | Render `display_name` as an editable text input.                                    |
| `FE-ACCOUNT-EDIT-DISPLAY-002` | Associate the input with a visible label.                                           |
| `FE-ACCOUNT-EDIT-DISPLAY-003` | Initialize the input from the authoritative Account response.                       |
| `FE-ACCOUNT-EDIT-DISPLAY-004` | Treat an empty value as a request to clear the display name.                        |
| `FE-ACCOUNT-EDIT-DISPLAY-005` | Permit the backend-supported maximum of 100 Unicode scalar values.                  |
| `FE-ACCOUNT-EDIT-DISPLAY-006` | Do not impose unsupported character-composition rules.                              |
| `FE-ACCOUNT-EDIT-DISPLAY-007` | Submit only `display_name` to `AC_API_04`.                                          |
| `FE-ACCOUNT-EDIT-DISPLAY-008` | Use `application/merge-patch+json`.                                                 |
| `FE-ACCOUNT-EDIT-DISPLAY-009` | Disable the submit control while the operation is pending.                          |
| `FE-ACCOUNT-EDIT-DISPLAY-010` | Prevent duplicate submissions.                                                      |
| `FE-ACCOUNT-EDIT-DISPLAY-011` | Use the Account returned by the successful response as the new authoritative state. |

### Request

```http
PATCH /admin/accounts/{id}
Authorization: Bearer <access-token>
Content-Type: application/merge-patch+json
Accept: application/json, application/problem+json

{
  "display_name": "Library Administrator"
}
```

Clear operation:

```json
{
  "display_name": null
}
```

### Dirty State

The `Save changes` control should remain disabled when the local value has not changed from the value loaded from the authoritative Account response.

The frontend must not claim success before the server response succeeds.

### Success

`200 OK`

The Account response returned by the backend becomes authoritative.

The page must update:

```text
display_name
updated_at
```

from the returned representation.

---

# 7. Change Email

## 7.1 Form

```text
Change Email

Current email
admin@example.com

New email
[new-admin@example.com                ]

[Change email]
```

| ID                          | Requirement                                                                                |
| --------------------------- | ------------------------------------------------------------------------------------------ |
| `FE-ACCOUNT-EDIT-EMAIL-001` | Provide a dedicated Change Email form.                                                     |
| `FE-ACCOUNT-EDIT-EMAIL-002` | Show the current Account email as read-only context.                                       |
| `FE-ACCOUNT-EDIT-EMAIL-003` | Provide an editable email control for the replacement address.                             |
| `FE-ACCOUNT-EDIT-EMAIL-004` | Use the semantic `email` input type.                                                       |
| `FE-ACCOUNT-EDIT-EMAIL-005` | Associate the input with a visible label.                                                  |
| `FE-ACCOUNT-EDIT-EMAIL-006` | Do not submit display-name, status, or lifecycle fields with the email request.            |
| `FE-ACCOUNT-EDIT-EMAIL-007` | Submit only the supported email payload to `AC_API_10`.                                    |
| `FE-ACCOUNT-EDIT-EMAIL-008` | Disable the Change Email control while the request is pending.                             |
| `FE-ACCOUNT-EDIT-EMAIL-009` | Prevent duplicate submissions.                                                             |
| `FE-ACCOUNT-EDIT-EMAIL-010` | Treat the returned Account response as authoritative.                                      |
| `FE-ACCOUNT-EDIT-EMAIL-011` | Do not allow native browser constraint validation to reject server-supported email syntax. |

The frontend may use `type="email"` for semantic input behavior, keyboard/input assistance, and user-agent hints.

However, native browser email constraint validation must not be allowed to block server-supported Unicode local-parts or IDN domains. The Change Email form should therefore use `noValidate` and rely on server validation for authoritative acceptance.

The frontend must not reject server-supported Unicode local-parts or IDN domains through unsupported client-only validation rules.

### Request

```http
PATCH /admin/accounts/{id}/email
Authorization: Bearer <access-token>
Content-Type: application/json
Accept: application/json, application/problem+json

{
  "email": "new-admin@example.com"
}
```

### Backend Validation

The frontend must treat the backend as authoritative for:

```text
email syntax
case-insensitive identity
Unicode local-parts
IDN domains
normalization
email uniqueness
security validation
```

The frontend must not reimplement the backend normalization algorithm as the authority.

The frontend may provide ordinary input assistance for immediate user feedback, but a server rejection remains authoritative.

The frontend must not reject valid server-supported input because of an unsupported client-only composition rule.

### Success

`200 OK`

The returned Account response becomes authoritative.

The page must update the displayed email from the response.

`account.updated_at` must not be fabricated as changed by the frontend because the backend contract explicitly states that Change Email updates credential timestamps rather than `account.updated_at`.

---

# 8. Change Password

## 8.1 Form

```text
Change Password

New password
[••••••••••••••••••••••••••      ]

Minimum length: 15 characters.

[Change password]
```

The form must not pre-populate the password field.

| ID                             | Requirement                                                                                    |
| ------------------------------ | ---------------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-EDIT-PASSWORD-001` | Provide a dedicated Change Password form.                                                      |
| `FE-ACCOUNT-EDIT-PASSWORD-002` | Use `type="password"`.                                                                         |
| `FE-ACCOUNT-EDIT-PASSWORD-003` | Use `autocomplete="new-password"`.                                                             |
| `FE-ACCOUNT-EDIT-PASSWORD-004` | Do not pre-populate the field.                                                                 |
| `FE-ACCOUNT-EDIT-PASSWORD-005` | Require at least 15 characters on the client before submission.                                |
| `FE-ACCOUNT-EDIT-PASSWORD-006` | Do not require uppercase, lowercase, number, or symbol combinations in the frontend.           |
| `FE-ACCOUNT-EDIT-PASSWORD-007` | Do not impose a smaller maximum password length than the backend contract supports.            |
| `FE-ACCOUNT-EDIT-PASSWORD-008` | Submit only the supported password payload to `AC_API_11`.                                     |
| `FE-ACCOUNT-EDIT-PASSWORD-009` | Disable the Change Password control while the request is pending.                              |
| `FE-ACCOUNT-EDIT-PASSWORD-010` | Prevent duplicate submissions.                                                                 |
| `FE-ACCOUNT-EDIT-PASSWORD-011` | Clear the password input after successful submission.                                          |
| `FE-ACCOUNT-EDIT-PASSWORD-012` | Never place the password in a URL, query parameter, navigation state, log, or browser storage. |

### Request

```http
PATCH /admin/accounts/{id}/password
Authorization: Bearer <access-token>
Content-Type: application/json
Accept: application/json, application/problem+json

{
  "password": "new-secure-password"
}
```

### Backend Validation

The backend remains authoritative for:

```text
minimum length
maximum supported length
Argon2id hashing
unique salt generation
compromised/common password blocklists
password policy
credential persistence
```

The frontend must not attempt to determine whether a password is present in the configured compromised-password corpus.

The frontend must display a generic password-policy error when the server returns:

```text
PASSWORD_POLICY_VIOLATION
```

The page must not expose the server's internal password blocklist contents.

### Success

`204 No Content`

No Account representation is returned.

The page must:

1. Clear the password input.
2. Reset password dirty state.
3. Display a successful status message.
4. Remain on the Account Edit page unless the shared authentication layer redirects the user.

The frontend must not fabricate changes to Account metadata that the backend did not return.

---

# 9. Edit Page Action Boundary

## 9.1 Actions Provided

The Account Edit page provides:

```text
Back to Administrator Accounts
Save changes
Change email
Change password
```

| ID                           | Requirement                                                        |
| ---------------------------- | ------------------------------------------------------------------ |
| `FE-ACCOUNT-EDIT-ACTION-001` | Back navigation returns to the validated Account List destination. |
| `FE-ACCOUNT-EDIT-ACTION-002` | Save Changes submits only `display_name`.                          |
| `FE-ACCOUNT-EDIT-ACTION-003` | Change Email submits only `email`.                                 |
| `FE-ACCOUNT-EDIT-ACTION-004` | Change Password submits only `password`.                           |
| `FE-ACCOUNT-EDIT-ACTION-005` | Each mutation uses its dedicated backend contract.                 |
| `FE-ACCOUNT-EDIT-ACTION-006` | Each mutation has independent pending and error state.             |

## 9.2 Actions Not Provided

The page must not provide:

```text
Deactivate
Activate
Restore
Delete
Permanently Delete
Assign Role
Revoke Role
```

| ID                           | Requirement                       |
| ---------------------------- | --------------------------------- |
| `FE-ACCOUNT-EDIT-ACTION-007` | Do not expose Deactivate.         |
| `FE-ACCOUNT-EDIT-ACTION-008` | Do not expose Activate.           |
| `FE-ACCOUNT-EDIT-ACTION-009` | Do not expose Restore.            |
| `FE-ACCOUNT-EDIT-ACTION-010` | Do not expose Delete.             |
| `FE-ACCOUNT-EDIT-ACTION-011` | Do not expose Permanently Delete. |
| `FE-ACCOUNT-EDIT-ACTION-012` | Do not expose role assignment.    |
| `FE-ACCOUNT-EDIT-ACTION-013` | Do not expose role revocation.    |

Lifecycle actions remain Account Management responsibilities.

Destructive operations remain the dedicated Account Delete page responsibility.

Role management remains the Authorization responsibility.

---

# 10. API Contract

## 10.1 Load Account

```http
GET /admin/accounts/{id}
Authorization: Bearer <access-token>
Accept: application/json, application/problem+json
```

Permission:

```text
account:view
```

Soft-deleted Accounts are not returned by normal lookup.

Expected errors:

| Status          | Code                 | UI Behavior                                 |
| --------------- | -------------------- | ------------------------------------------- |
| `400`           | `INVALID_ACCOUNT_ID` | Show invalid Account identifier error.      |
| `401`           | —                    | Delegate to shared authentication handling. |
| `403`           | —                    | Render authorization error without logout.  |
| `404`           | `ACCOUNT_NOT_FOUND`  | Render Account Not Found state.             |
| `500`           | —                    | Render retryable server error.              |
| Network failure | —                    | Render retryable network error.             |

## 10.2 Update Display Name

```http
PATCH /admin/accounts/{id}
Authorization: Bearer <access-token>
Content-Type: application/merge-patch+json
Accept: application/json, application/problem+json
```

Permission:

```text
account:update
```

Success:

```text
200 OK
Account Response
```

Errors:

| Status          | Code                     | UI Behavior                                                      |
| --------------- | ------------------------ | ---------------------------------------------------------------- |
| `400`           | `INVALID_ACCOUNT_ID`     | Show invalid Account identifier error.                           |
| `401`           | —                        | Delegate to shared authentication handling.                      |
| `403`           | —                        | Show authorization error.                                        |
| `404`           | `ACCOUNT_NOT_FOUND`      | Treat Account as unavailable and refresh/return to Account List. |
| `415`           | `UNSUPPORTED_MEDIA_TYPE` | Show request-format error.                                       |
| `422`           | `VALIDATION_ERROR`       | Show validation error.                                           |
| `500`           | —                        | Show retryable server error.                                     |
| Network failure | —                        | Show network error.                                              |

## 10.3 Change Email

```http
PATCH /admin/accounts/{id}/email
Authorization: Bearer <access-token>
Content-Type: application/json
Accept: application/json, application/problem+json
```

Permission:

```text
account:change_email
```

Success:

```text
200 OK
Account Response
```

Errors:

| Status          | Code                   | UI Behavior                                   |
| --------------- | ---------------------- | --------------------------------------------- |
| `401`           | —                      | Delegate to shared authentication handling.   |
| `403`           | —                      | Show authorization error.                     |
| `404`           | `ACCOUNT_NOT_FOUND`    | Treat Account as unavailable.                 |
| `409`           | `EMAIL_ALREADY_IN_USE` | Show email conflict on the Change Email form. |
| `422`           | `VALIDATION_ERROR`     | Show email validation error.                  |
| `500`           | —                      | Show retryable server error.                  |
| Network failure | —                      | Show network error.                           |

## 10.4 Change Password

```http
PATCH /admin/accounts/{id}/password
Authorization: Bearer <access-token>
Content-Type: application/json
Accept: application/json, application/problem+json
```

Permission:

```text
account:change_password
```

Success:

```text
204 No Content
```

Errors:

| Status          | Code                        | UI Behavior                                 |
| --------------- | --------------------------- | ------------------------------------------- |
| `401`           | —                           | Delegate to shared authentication handling. |
| `403`           | —                           | Show authorization error.                   |
| `404`           | `ACCOUNT_NOT_FOUND`         | Treat Account as unavailable.               |
| `422`           | `PASSWORD_POLICY_VIOLATION` | Show password-policy error.                 |
| `500`           | —                           | Show retryable server error.                |
| Network failure | —                           | Show network error.                         |

## 10.5 Authorization Boundary

The frontend must not enforce:

```text
account:update
account:change_email
account:change_password
```

as security controls.

The backend must enforce each permission server-side.

The frontend may render an authorization error when a protected mutation returns `403`, but the frontend is never the authorization authority.

## 10.6 Cache

Account API responses must use the repository Account API cache contract.

Account responses must use:

```http
Cache-Control: no-store
```

The frontend must not store Account representations in:

```text
localStorage
sessionStorage
IndexedDB
```

## 10.7 Problem Details

Account API errors use:

```text
application/problem+json
```

with RFC 9457 Problem Details semantics.

The frontend should use the structured `code` when available.

The frontend must not render internal stack traces or sensitive server diagnostics.

---

# 11. Validation and Error States

## 11.1 Display Name Validation

Client behavior:

```text
empty
    ↓
submit null
```

```text
non-empty
    ↓
submit string
    ↓
server validates normalization and length
```

Known server validation failure:

```text
The display name is invalid. Review the value and try again.
```

The frontend must not claim that client-side normalization is the authoritative Account normalization algorithm.

## 11.2 Email Validation

The email field should use the native `email` input semantics for ordinary input assistance.

The containing form must not allow native browser constraint validation to block server-supported Unicode local-parts or IDN domains.

Server validation remains authoritative.

Known conflict:

```text
EMAIL_ALREADY_IN_USE
```

User-facing message:

```text
That email address is already in use.
```

Known validation failure:

```text
The email address is not valid.
```

The frontend must not expose internal normalization or account lookup details.

## 11.3 Password Validation

Client-side requirements:

```text
minimum 15 characters
no unsupported composition rule
no client-side compromised-password database
```

Server-side requirements remain authoritative.

Known failure:

```text
PASSWORD_POLICY_VIOLATION
```

User-facing message:

```text
The new password does not meet the password policy.
```

The frontend should not reveal whether a specific password was found in the server's compromised-password blocklist.

## 11.4 Authorization Errors

For `403 Forbidden`:

```text
You do not have permission to perform this account change.
```

The page must preserve the authenticated session.

The frontend must not redirect to `/login` solely because of `403`.

## 11.5 Not Found

For `404 ACCOUNT_NOT_FOUND`:

```text
This administrator account is no longer available.
```

The page must:

1. Stop all edit controls from submitting.
2. Avoid displaying stale Account values as current.
3. Provide navigation back to Account Management.
4. Re-fetch Account List data when returning to the list if required by the originating flow.

## 11.6 General Error

```text
Unable to save this account change.

[Try Again]
```

The frontend must keep the user's non-sensitive editable state when it is safe to retry.

Password state may remain only in component memory until the user leaves the page or a successful password change clears it.

---

# 12. UI States

## 12.1 Authentication Bootstrap

While authentication is unresolved:

```text
Checking authentication…
```

Do not flash the login page before shared authentication state resolves.

## 12.2 Initial Loading

```text
Edit Administrator Account

Loading administrator account…
```

The page must not display fabricated Account values.

Edit forms must not be enabled before authoritative Account data is available.

## 12.3 Loaded

```text
Back to Administrator Accounts

Edit Administrator Account
Library Administrator
admin@example.com
Active

Account Information
Display name
[Library Administrator              ]
[Save changes]

Change Email
New email
[                                ]
[Change email]

Change Password
New password
[                                ]
[Change password]
```

Lifecycle controls must not appear.

## 12.4 Mutation Pending

Each form must expose its own pending state.

Examples:

```text
Saving…
Changing email…
Changing password…
```

The pending control must be disabled while its operation is active.

Duplicate submissions must be prevented.

## 12.5 Mutation Success

Examples:

```text
Account changes saved.
```

```text
Email changed successfully.
```

```text
Password changed successfully.
```

Success messages should use an accessible status mechanism.

## 12.6 Authorization Error

```text
You do not have permission to perform this account change.
```

The Admin Shell remains active.

## 12.7 Account Not Found

```text
Administrator Account

This administrator account is no longer available.

[Back to Administrator Accounts]
```

No edit forms are displayed.

## 12.8 Authentication Expiry

The page relies on the shared Authentication service.

Expected behavior:

```text
401
  ↓
authentication recovery
  ↓
if authentication cannot be recovered
  ↓
/login
```

The Account Edit page must not implement an independent authentication system.

---

# 13. Responsive Layout

The page inherits the Admin Shell responsive behavior and manages only its Main Content layout.

| ID                         | Viewport       | Layout                                                                       |
| -------------------------- | -------------- | ---------------------------------------------------------------------------- |
| `FE-ACCOUNT-EDIT-RESP-001` | `>= 1280px`    | Two-column content may be used for Account summary and edit sections.        |
| `FE-ACCOUNT-EDIT-RESP-002` | `768px–1279px` | Single-column or constrained two-column layout according to available width. |
| `FE-ACCOUNT-EDIT-RESP-003` | `< 768px`      | Single-column stacked forms.                                                 |
| `FE-ACCOUNT-EDIT-RESP-004` | All viewports  | Form controls remain accessible and usable.                                  |
| `FE-ACCOUNT-EDIT-RESP-005` | All viewports  | No unintended page-level horizontal scrolling.                               |
| `FE-ACCOUNT-EDIT-RESP-006` | All viewports  | Focused controls remain visible.                                             |
| `FE-ACCOUNT-EDIT-RESP-007` | All viewports  | Authentication and authorization behavior remain unchanged.                  |

Suggested desktop structure:

```text
┌─────────────────────────────────────────────────────────────┐
│ Back to Administrator Accounts                              │
│                                                             │
│ Edit Administrator Account                                  │
│ Library Administrator    admin@example.com    Active        │
│                                                             │
│ ┌─────────────────────────┐ ┌─────────────────────────────┐ │
│ │ Account Information     │ │ Change Email                │ │
│ │ Display name            │ │ New email                   │ │
│ │ [....................]  │ │ [.........................] │ │
│ │ [Save changes]          │ │ [Change email]              │ │
│ └─────────────────────────┘ └─────────────────────────────┘ │
│                                                             │
│ ┌──────────────────────────────────────────────────────────┐ │
│ │ Change Password                                          │ │
│ │ New password                                             │ │
│ │ [......................................................] │ │
│ │ [Change password]                                        │ │
│ └──────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

Mobile:

```text
Edit Administrator Account

Back to Administrator Accounts

Account Information
Display name
[.........................]
[Save changes]

Change Email
New email
[.........................]
[Change email]

Change Password
New password
[.........................]
[Change password]
```

The exact visual styling must use the existing native CSS architecture and design conventions.

---

# 14. Security

| ID                        | Security Rule            | Requirement                                                                                                    |
| ------------------------- | ------------------------ | -------------------------------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-EDIT-SEC-001` | Authentication           | Account Edit requires authenticated Admin Shell state.                                                         |
| `FE-ACCOUNT-EDIT-SEC-002` | Authentication authority | Backend Authentication remains authoritative.                                                                  |
| `FE-ACCOUNT-EDIT-SEC-003` | Authorization            | Backend Authorization remains authoritative.                                                                   |
| `FE-ACCOUNT-EDIT-SEC-004` | View permission          | Account loading requires `account:view`.                                                                       |
| `FE-ACCOUNT-EDIT-SEC-005` | Update permission        | Display name update requires `account:update`.                                                                 |
| `FE-ACCOUNT-EDIT-SEC-006` | Email permission         | Email change requires `account:change_email`.                                                                  |
| `FE-ACCOUNT-EDIT-SEC-007` | Password permission      | Password change requires `account:change_password`.                                                            |
| `FE-ACCOUNT-EDIT-SEC-008` | Token transport          | Bearer credentials use the `Authorization` header only.                                                        |
| `FE-ACCOUNT-EDIT-SEC-009` | URL secrecy              | Tokens and passwords never appear in URLs.                                                                     |
| `FE-ACCOUNT-EDIT-SEC-010` | Browser persistence      | Passwords and Account response data are not stored in browser persistence.                                     |
| `FE-ACCOUNT-EDIT-SEC-011` | Logging                  | Passwords, tokens, and Authorization headers are never logged.                                                 |
| `FE-ACCOUNT-EDIT-SEC-012` | DOM exposure             | Password values must not be rendered as ordinary text or exposed outside the password input's protected value. |
| `FE-ACCOUNT-EDIT-SEC-013` | Error exposure           | Internal server diagnostics are not exposed.                                                                   |
| `FE-ACCOUNT-EDIT-SEC-014` | Client authorization     | Client-held roles or permissions are never trusted as authorization proof.                                     |
| `FE-ACCOUNT-EDIT-SEC-015` | Open redirect            | `return_to` accepts only validated same-origin `/admin/accounts` destinations with supported query parameters. |
| `FE-ACCOUNT-EDIT-SEC-016` | Soft deletion            | Soft-deleted Accounts are unavailable to normal Edit operations.                                               |
| `FE-ACCOUNT-EDIT-SEC-017` | Mutation isolation       | Each form calls only its dedicated mutation endpoint.                                                          |
| `FE-ACCOUNT-EDIT-SEC-018` | Credential minimization  | The frontend never sends email/password fields to an unsupported endpoint.                                     |

The frontend must not attempt to implement:

```text
password hashing
email identity persistence
password blocklist persistence
account authorization policy
account lifecycle invariants
session revocation policy
```

Those remain backend/domain responsibilities.

## Password Security Boundary

The project backend requires:

```text
Argon2id
unique password salt
minimum 15 characters
support for at least 64 characters
no arbitrary composition rules
compromised/common password blocking
```

The frontend must present these user-facing requirements consistently but must not replace backend validation.

---

# 15. Accessibility

## 15.1 Semantic Structure

| ID                         | Requirement                                                                      |
| -------------------------- | -------------------------------------------------------------------------------- |
| `FE-ACCOUNT-EDIT-A11Y-001` | Use semantic `main`, `header`, `section`, `form`, `label`, and heading elements. |
| `FE-ACCOUNT-EDIT-A11Y-002` | Provide exactly one primary `h1`.                                                |
| `FE-ACCOUNT-EDIT-A11Y-003` | Use logical `h2` headings for each edit section.                                 |
| `FE-ACCOUNT-EDIT-A11Y-004` | Associate every editable field with a visible label.                             |
| `FE-ACCOUNT-EDIT-A11Y-005` | Do not use placeholder text as the only field label.                             |

## 15.2 Form Labels and Descriptions

Each field must have:

```text
visible label
optional descriptive help text
optional error message
```

When an error exists:

```text
aria-invalid="true"
aria-describedby="..."
```

should connect the field to its error or guidance text where appropriate.

## 15.3 Error Identification

| ID                         | Requirement                                                        |
| -------------------------- | ------------------------------------------------------------------ |
| `FE-ACCOUNT-EDIT-A11Y-006` | Automatically detected input errors are identified in text.        |
| `FE-ACCOUNT-EDIT-A11Y-007` | Validation messages identify the affected field.                   |
| `FE-ACCOUNT-EDIT-A11Y-008` | Known corrective guidance is provided when safe.                   |
| `FE-ACCOUNT-EDIT-A11Y-009` | Internal or security-sensitive validation details are not exposed. |

## 15.4 Async Status

Use accessible status messaging for:

```text
loading
saving
email change pending
password change pending
success
recoverable errors
```

A suitable success/pending region may use:

```html
<p role="status" aria-live="polite"></p>
```

Errors may use:

```html
<div role="alert"></div>
```

Status messaging must not move focus unnecessarily.

## 15.5 Focus

| ID                         | Requirement                                                                  |
| -------------------------- | ---------------------------------------------------------------------------- |
| `FE-ACCOUNT-EDIT-A11Y-010` | Keyboard focus is visibly indicated.                                         |
| `FE-ACCOUNT-EDIT-A11Y-011` | Focus is not completely obscured by responsive or sticky UI.                 |
| `FE-ACCOUNT-EDIT-A11Y-012` | Error focus management does not repeatedly steal focus during normal typing. |
| `FE-ACCOUNT-EDIT-A11Y-013` | After a recoverable submit failure, focus remains usable for correction.     |

## 15.6 Target Size

Interactive controls must satisfy the applicable WCAG 2.2 Target Size (Minimum) requirement or a documented exception.

The following controls must remain independently targetable:

```text
Back
Save changes
Change email
Change password
```

## 15.7 Password Input

The password control must:

```text
use type=password
use autocomplete="new-password"
have a visible label
not expose the password value in visible text
```

Password managers must remain able to recognize the field correctly.

---

# 16. Component Structure

The page should remain feature-local.

Suggested structure:

```text
src/pages/accounts/
├── AccountListPage
├── AccountCreatePage
├── AccountDetailPage
└── AccountEditPage
```

Suggested page-local components:

```text
AccountEditPage
├── AccountEditHeader
├── AccountEditSummary
├── AccountDisplayNameForm
├── AccountEmailForm
├── AccountPasswordForm
├── AccountFormField
├── AccountFormStatus
├── AccountLoadingState
├── AccountErrorState
└── AccountNotFoundState
```

| ID                         | Component                | Responsibility                                                                       |
| -------------------------- | ------------------------ | ------------------------------------------------------------------------------------ |
| `FE-ACCOUNT-EDIT-COMP-001` | `AccountEditPage`        | Coordinate route state, Account data, form state, submission state, and page layout. |
| `FE-ACCOUNT-EDIT-COMP-002` | `AccountEditHeader`      | Render page heading, summary, status, and Back navigation.                           |
| `FE-ACCOUNT-EDIT-COMP-003` | `AccountEditSummary`     | Render read-only Account metadata.                                                   |
| `FE-ACCOUNT-EDIT-COMP-004` | `AccountDisplayNameForm` | Edit `display_name` through `AC_API_04`.                                             |
| `FE-ACCOUNT-EDIT-COMP-005` | `AccountEmailForm`       | Change email through `AC_API_10`.                                                    |
| `FE-ACCOUNT-EDIT-COMP-006` | `AccountPasswordForm`    | Change password through `AC_API_11`.                                                 |
| `FE-ACCOUNT-EDIT-COMP-007` | `AccountFormField`       | Provide consistent labels, help text, and error association where useful.            |
| `FE-ACCOUNT-EDIT-COMP-008` | `AccountFormStatus`      | Render accessible pending and success messaging.                                     |
| `FE-ACCOUNT-EDIT-COMP-009` | `AccountLoadingState`    | Render initial Account loading state.                                                |
| `FE-ACCOUNT-EDIT-COMP-010` | `AccountErrorState`      | Render route-level load and server errors.                                           |
| `FE-ACCOUNT-EDIT-COMP-011` | `AccountNotFoundState`   | Render unavailable Account state.                                                    |

The page must not introduce:

```text
AccountLifecycleSection
AccountDeleteDialog
RoleManagementSection
PasswordHashField
```

API calls should remain in:

```text
src/services/
```

Types should remain in:

```text
src/types/
```

Routing helpers should remain in the existing Account route/navigation layer.

No page-specific UI framework or large component dependency should be introduced solely for Account Edit.

---

# 17. Testing

The implementation must follow the repository test strategy:

```text
Static Checks
    ↓
Unit
    ↓
Integration
    ↓
E2E
    ↓
Manual
```

## 17.1 Unit

| ID                              | Test                                                                                                                |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-EDIT-TEST-UNIT-001` | Account response initializes the Edit page correctly.                                                               |
| `FE-ACCOUNT-EDIT-TEST-UNIT-002` | `display_name=null` initializes as an empty editable value.                                                         |
| `FE-ACCOUNT-EDIT-TEST-UNIT-003` | Display name dirty-state detection works.                                                                           |
| `FE-ACCOUNT-EDIT-TEST-UNIT-004` | Empty display name produces `display_name: null`.                                                                   |
| `FE-ACCOUNT-EDIT-TEST-UNIT-005` | Display name submit uses only the supported Account Update payload.                                                 |
| `FE-ACCOUNT-EDIT-TEST-UNIT-006` | Email input uses email semantics.                                                                                   |
| `FE-ACCOUNT-EDIT-TEST-UNIT-007` | Email submission uses only the supported email payload.                                                             |
| `FE-ACCOUNT-EDIT-TEST-UNIT-008` | Email form does not block server-supported Unicode local-parts or IDN domains through native constraint validation. |
| `FE-ACCOUNT-EDIT-TEST-UNIT-009` | Password input uses `type=password`.                                                                                |
| `FE-ACCOUNT-EDIT-TEST-UNIT-010` | Password input uses `autocomplete=new-password`.                                                                    |
| `FE-ACCOUNT-EDIT-TEST-UNIT-011` | Password input enforces the 15-character client minimum.                                                            |
| `FE-ACCOUNT-EDIT-TEST-UNIT-012` | Password form does not enforce unsupported composition rules.                                                       |
| `FE-ACCOUNT-EDIT-TEST-UNIT-013` | Password submission uses only the supported password payload.                                                       |
| `FE-ACCOUNT-EDIT-TEST-UNIT-014` | Display name success uses the returned Account representation.                                                      |
| `FE-ACCOUNT-EDIT-TEST-UNIT-015` | Email success uses the returned Account representation.                                                             |
| `FE-ACCOUNT-EDIT-TEST-UNIT-016` | Password `204` produces a success state and clears the password field.                                              |
| `FE-ACCOUNT-EDIT-TEST-UNIT-017` | `EMAIL_ALREADY_IN_USE` renders email conflict feedback.                                                             |
| `FE-ACCOUNT-EDIT-TEST-UNIT-018` | `PASSWORD_POLICY_VIOLATION` renders password-policy feedback.                                                       |
| `FE-ACCOUNT-EDIT-TEST-UNIT-019` | `403` renders authorization feedback without logout.                                                                |
| `FE-ACCOUNT-EDIT-TEST-UNIT-020` | `404` renders Account Not Found state.                                                                              |
| `FE-ACCOUNT-EDIT-TEST-UNIT-021` | Duplicate submission is prevented.                                                                                  |
| `FE-ACCOUNT-EDIT-TEST-UNIT-022` | Lifecycle mutation controls are not rendered.                                                                       |
| `FE-ACCOUNT-EDIT-TEST-UNIT-023` | Delete controls are not rendered.                                                                                   |
| `FE-ACCOUNT-EDIT-TEST-UNIT-024` | Invalid `return_to` falls back to `/admin/accounts`.                                                                |
| `FE-ACCOUNT-EDIT-TEST-UNIT-025` | External `return_to` destinations are rejected.                                                                     |
| `FE-ACCOUNT-EDIT-TEST-UNIT-026` | `return_to` values containing unsupported query parameters are rejected.                                            |

## 17.2 Integration

| ID                             | Test                                                                    |
| ------------------------------ | ----------------------------------------------------------------------- |
| `FE-ACCOUNT-EDIT-TEST-INT-001` | Authenticated navigation to Account Edit succeeds.                      |
| `FE-ACCOUNT-EDIT-TEST-INT-002` | Account data loads through `GET /admin/accounts/{id}`.                  |
| `FE-ACCOUNT-EDIT-TEST-INT-003` | Display name update uses `PATCH /admin/accounts/{id}`.                  |
| `FE-ACCOUNT-EDIT-TEST-INT-004` | Display name update uses `application/merge-patch+json`.                |
| `FE-ACCOUNT-EDIT-TEST-INT-005` | Email change uses `PATCH /admin/accounts/{id}/email`.                   |
| `FE-ACCOUNT-EDIT-TEST-INT-006` | Password change uses `PATCH /admin/accounts/{id}/password`.             |
| `FE-ACCOUNT-EDIT-TEST-INT-007` | Password `204` is handled correctly.                                    |
| `FE-ACCOUNT-EDIT-TEST-INT-008` | Email `409 EMAIL_ALREADY_IN_USE` is rendered correctly.                 |
| `FE-ACCOUNT-EDIT-TEST-INT-009` | Display name `415` is rendered correctly.                               |
| `FE-ACCOUNT-EDIT-TEST-INT-010` | Display name and email `422` responses are rendered correctly.          |
| `FE-ACCOUNT-EDIT-TEST-INT-011` | Password `422 PASSWORD_POLICY_VIOLATION` is rendered correctly.         |
| `FE-ACCOUNT-EDIT-TEST-INT-012` | `401` delegates to shared authentication recovery.                      |
| `FE-ACCOUNT-EDIT-TEST-INT-013` | `403` preserves authentication state.                                   |
| `FE-ACCOUNT-EDIT-TEST-INT-014` | `404` renders Account Not Found.                                        |
| `FE-ACCOUNT-EDIT-TEST-INT-015` | Account response data is not written to browser persistence.            |
| `FE-ACCOUNT-EDIT-TEST-INT-016` | Password values are not placed in URLs or navigation state.             |
| `FE-ACCOUNT-EDIT-TEST-INT-017` | Each form sends only its dedicated mutation payload.                    |
| `FE-ACCOUNT-EDIT-TEST-INT-018` | Pending state prevents duplicate form submissions.                      |
| `FE-ACCOUNT-EDIT-TEST-INT-019` | Valid `return_to` navigation returns to originating Account List state. |
| `FE-ACCOUNT-EDIT-TEST-INT-020` | Invalid `return_to` navigation falls back safely.                       |
| `FE-ACCOUNT-EDIT-TEST-INT-021` | Unsupported `return_to` query parameters are rejected safely.           |

## 17.3 E2E

| ID                             | Test                                                             |
| ------------------------------ | ---------------------------------------------------------------- |
| `FE-ACCOUNT-EDIT-TEST-E2E-001` | Login → Admin Shell → Accounts → Edit works.                     |
| `FE-ACCOUNT-EDIT-TEST-E2E-002` | Edit route loads the expected administrator Account.             |
| `FE-ACCOUNT-EDIT-TEST-E2E-003` | Display name can be changed successfully.                        |
| `FE-ACCOUNT-EDIT-TEST-E2E-004` | Display name can be cleared successfully.                        |
| `FE-ACCOUNT-EDIT-TEST-E2E-005` | Email can be changed successfully.                               |
| `FE-ACCOUNT-EDIT-TEST-E2E-006` | Duplicate email displays a conflict error without false success. |
| `FE-ACCOUNT-EDIT-TEST-E2E-007` | Password can be changed successfully.                            |
| `FE-ACCOUNT-EDIT-TEST-E2E-008` | Invalid password policy response is displayed correctly.         |
| `FE-ACCOUNT-EDIT-TEST-E2E-009` | Display name, email, and password forms are independent.         |
| `FE-ACCOUNT-EDIT-TEST-E2E-010` | Lifecycle mutation controls are not present.                     |
| `FE-ACCOUNT-EDIT-TEST-E2E-011` | Delete controls are not present.                                 |
| `FE-ACCOUNT-EDIT-TEST-E2E-012` | Authentication expiry redirects to `/login`.                     |
| `FE-ACCOUNT-EDIT-TEST-E2E-013` | Unauthorized mutation remains authenticated and displays `403`.  |
| `FE-ACCOUNT-EDIT-TEST-E2E-014` | Account not found renders the unavailable state.                 |
| `FE-ACCOUNT-EDIT-TEST-E2E-015` | Return navigation preserves Account List query state.            |
| `FE-ACCOUNT-EDIT-TEST-E2E-016` | Invalid external return navigation is rejected.                  |
| `FE-ACCOUNT-EDIT-TEST-E2E-017` | Duplicate submission is prevented for each form.                 |
| `FE-ACCOUNT-EDIT-TEST-E2E-018` | Mobile layout remains usable below `768px`.                      |
| `FE-ACCOUNT-EDIT-TEST-E2E-019` | Unsupported `return_to` query parameters are rejected.           |

## 17.4 Accessibility

| ID                              | Test                                                                                                   |
| ------------------------------- | ------------------------------------------------------------------------------------------------------ |
| `FE-ACCOUNT-EDIT-TEST-A11Y-001` | One primary `h1` exists.                                                                               |
| `FE-ACCOUNT-EDIT-TEST-A11Y-002` | All editable fields have accessible labels.                                                            |
| `FE-ACCOUNT-EDIT-TEST-A11Y-003` | Validation errors are programmatically associated with fields.                                         |
| `FE-ACCOUNT-EDIT-TEST-A11Y-004` | Pending status is announced appropriately.                                                             |
| `FE-ACCOUNT-EDIT-TEST-A11Y-005` | Success status is announced appropriately.                                                             |
| `FE-ACCOUNT-EDIT-TEST-A11Y-006` | Error messages are presented as text.                                                                  |
| `FE-ACCOUNT-EDIT-TEST-A11Y-007` | Keyboard navigation works for all forms.                                                               |
| `FE-ACCOUNT-EDIT-TEST-A11Y-008` | Focus is visible.                                                                                      |
| `FE-ACCOUNT-EDIT-TEST-A11Y-009` | Focus is not obscured.                                                                                 |
| `FE-ACCOUNT-EDIT-TEST-A11Y-010` | Interactive targets satisfy the applicable WCAG 2.2 target-size requirement or a documented exception. |
| `FE-ACCOUNT-EDIT-TEST-A11Y-011` | Password field is announced as a password field through native semantics.                              |
| `FE-ACCOUNT-EDIT-TEST-A11Y-012` | Testing uses semantic queries such as role and label queries where practical.                          |
| `FE-ACCOUNT-EDIT-TEST-A11Y-013` | Automated accessibility scanning is supplemented by manual keyboard verification.                      |

## 17.5 Responsive

| ID                              | Test                                                     |
| ------------------------------- | -------------------------------------------------------- |
| `FE-ACCOUNT-EDIT-TEST-RESP-001` | Desktop layout works at `>= 1280px`.                     |
| `FE-ACCOUNT-EDIT-TEST-RESP-002` | Tablet layout works at `768px–1279px`.                   |
| `FE-ACCOUNT-EDIT-TEST-RESP-003` | Mobile layout works below `768px`.                       |
| `FE-ACCOUNT-EDIT-TEST-RESP-004` | No unintended page-level horizontal scrolling occurs.    |
| `FE-ACCOUNT-EDIT-TEST-RESP-005` | All form actions remain usable on mobile.                |
| `FE-ACCOUNT-EDIT-TEST-RESP-006` | Focused controls remain visible on mobile.               |
| `FE-ACCOUNT-EDIT-TEST-RESP-007` | Admin Shell behavior remains unchanged across viewports. |

---

# 18. Implementation Criteria

## 18.1 Route

| ID                         | Criteria                                                                                                              | Status         | Reason |
| -------------------------- | --------------------------------------------------------------------------------------------------------------------- | -------------- | ------ |
| `FE-ACCOUNT-EDIT-IMPL-001` | `/admin/accounts/:id/edit` is defined.                                                                                | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-002` | Route renders inside the Admin Shell.                                                                                 | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-003` | Unauthenticated access follows shared authentication behavior.                                                        | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-004` | `PAGE-ADM-010` remains the authoritative sitemap identifier.                                                          | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-005` | `return_to` is validated as same-origin `/admin/accounts` and preserves only supported Account List query parameters. | 🟢 Implemented |        |

## 18.2 Account Loading

| ID                         | Criteria                                               | Status         | Reason |
| -------------------------- | ------------------------------------------------------ | -------------- | ------ |
| `FE-ACCOUNT-EDIT-IMPL-006` | Account data loads through `GET /admin/accounts/{id}`. | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-007` | Server response is authoritative.                      | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-008` | Soft-deleted Accounts are treated as unavailable.      | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-009` | Account data is not persisted in browser storage.      | 🟢 Implemented |        |

## 18.3 Display Name

| ID                         | Criteria                                                     | Status         | Reason |
| -------------------------- | ------------------------------------------------------------ | -------------- | ------ |
| `FE-ACCOUNT-EDIT-IMPL-010` | `display_name` is editable.                                  | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-011` | Empty display name clears the field.                         | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-012` | Display name uses `PATCH /admin/accounts/{id}`.              | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-013` | Display name update uses `application/merge-patch+json`.     | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-014` | Server response replaces stale local Account representation. | 🟢 Implemented |        |

## 18.4 Change Email

| ID                         | Criteria                                                           | Status         | Reason |
| -------------------------- | ------------------------------------------------------------------ | -------------- | ------ |
| `FE-ACCOUNT-EDIT-IMPL-015` | Change Email form is present.                                      | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-016` | Email change uses `PATCH /admin/accounts/{id}/email`.              | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-017` | Email change requires `account:change_email` server authorization. | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-018` | `EMAIL_ALREADY_IN_USE` is handled explicitly.                      | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-019` | Successful email change uses the returned Account response.        | 🟢 Implemented |        |

## 18.5 Change Password

| ID                         | Criteria                                                                 | Status         | Reason |
| -------------------------- | ------------------------------------------------------------------------ | -------------- | ------ |
| `FE-ACCOUNT-EDIT-IMPL-020` | Change Password form is present.                                         | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-021` | Password uses `type="password"`.                                         | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-022` | Password uses `autocomplete="new-password"`.                             | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-023` | Client minimum is 15 characters.                                         | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-024` | Password change uses `PATCH /admin/accounts/{id}/password`.              | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-025` | Password change requires `account:change_password` server authorization. | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-026` | `204 No Content` is handled as successful password change.               | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-027` | Password input is cleared after successful change.                       | 🟢 Implemented |        |

## 18.6 Action Boundary

| ID                         | Criteria                                     | Status         | Reason |
| -------------------------- | -------------------------------------------- | -------------- | ------ |
| `FE-ACCOUNT-EDIT-IMPL-028` | Deactivate is absent from Account Edit.      | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-029` | Activate is absent from Account Edit.        | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-030` | Restore is absent from Account Edit.         | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-031` | Delete is absent from Account Edit.          | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-032` | Purge is absent from Account Edit.           | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-033` | Role assignment is absent from Account Edit. | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-034` | Role revocation is absent from Account Edit. | 🟢 Implemented |        |

## 18.7 Security

| ID                         | Criteria                                                        | Status         | Reason |
| -------------------------- | --------------------------------------------------------------- | -------------- | ------ |
| `FE-ACCOUNT-EDIT-IMPL-035` | Backend authentication remains authoritative.                   | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-036` | Backend authorization remains authoritative.                    | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-037` | Tokens do not appear in URLs.                                   | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-038` | Passwords do not appear in browser persistence.                 | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-039` | Passwords and tokens are not logged.                            | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-040` | `return_to` is same-origin and path-allowlisted.                | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-041` | Client-held permissions are not trusted as authorization proof. | 🟢 Implemented |        |

## 18.8 Accessibility

| ID                         | Criteria                                                                | Status         | Reason |
| -------------------------- | ----------------------------------------------------------------------- | -------------- | ------ |
| `FE-ACCOUNT-EDIT-IMPL-042` | Semantic page structure is used.                                        | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-043` | Editable fields have visible labels.                                    | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-044` | Async status is accessible.                                             | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-045` | Validation errors are associated with fields.                           | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-046` | Focus is visible and not obscured.                                      | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-047` | Password input uses native password semantics and correct autocomplete. | 🟢 Implemented |        |
| `FE-ACCOUNT-EDIT-IMPL-048` | Target size satisfies applicable WCAG 2.2 requirements.                 | 🟢 Implemented |        |

---

# 19. Traceability

## Requirements → Design → Backend → Test

| Requirement                     | Frontend Design                                       | Backend Contract                      | Test                                  |
| ------------------------------- | ----------------------------------------------------- | ------------------------------------- | ------------------------------------- |
| `ADM-AUTH-001`                  | `FE-ACCOUNT-EDIT-ROUTE-001`                           | Authentication domain                 | `FE-ACCOUNT-EDIT-TEST-E2E-001`        |
| `ADM-AUTH-005`                  | `FE-ACCOUNT-EDIT-REQ-001`                             | `AC_UC_04`, `AC_UC_10`, `AC_UC_11`    | `FE-ACCOUNT-EDIT-TEST-E2E-001`        |
| `AC_REQ_FC_06`                  | `FE-ACCOUNT-EDIT-REQ-004`                             | `AC_UC_04`, `AC_API_04`               | `FE-ACCOUNT-EDIT-TEST-INT-003`        |
| `AC_REQ_FC_12`                  | `FE-ACCOUNT-EDIT-REQ-005`, `006`                      | `AC_UC_10`, `AC_UC_11`                | `FE-ACCOUNT-EDIT-TEST-E2E-005`, `007` |
| `AC_SEC_REQ_FC_03`              | `FE-ACCOUNT-EDIT-REQ-005`                             | `AC_UC_10`, `AC_API_10`               | `FE-ACCOUNT-EDIT-TEST-INT-005`        |
| `AC_SEC_REQ_FC_04`              | `FE-ACCOUNT-EDIT-REQ-006`                             | `AC_UC_11`, `AC_API_11`               | `FE-ACCOUNT-EDIT-TEST-INT-006`        |
| `AC_UC_03`                      | `FE-ACCOUNT-EDIT-REQ-002`                             | `GET /admin/accounts/{id}`            | `FE-ACCOUNT-EDIT-TEST-INT-002`        |
| `AC_UC_04`                      | `FE-ACCOUNT-EDIT-DISPLAY-007`                         | `PATCH /admin/accounts/{id}`          | `FE-ACCOUNT-EDIT-TEST-INT-003`        |
| `AC_UC_10`                      | `FE-ACCOUNT-EDIT-EMAIL-007`                           | `PATCH /admin/accounts/{id}/email`    | `FE-ACCOUNT-EDIT-TEST-INT-005`        |
| `AC_UC_11`                      | `FE-ACCOUNT-EDIT-PASSWORD-008`                        | `PATCH /admin/accounts/{id}/password` | `FE-ACCOUNT-EDIT-TEST-INT-006`        |
| `account:view`                  | `FE-ACCOUNT-EDIT-SEC-004`                             | Authorization domain                  | `FE-ACCOUNT-EDIT-TEST-INT-002`        |
| `account:update`                | `FE-ACCOUNT-EDIT-SEC-005`                             | Authorization domain                  | `FE-ACCOUNT-EDIT-TEST-INT-003`        |
| `account:change_email`          | `FE-ACCOUNT-EDIT-SEC-006`                             | Authorization domain                  | `FE-ACCOUNT-EDIT-TEST-INT-005`        |
| `account:change_password`       | `FE-ACCOUNT-EDIT-SEC-007`                             | Authorization domain                  | `FE-ACCOUNT-EDIT-TEST-INT-006`        |
| `ACCOUNT_NOT_FOUND`             | `FE-ACCOUNT-EDIT-REQ-013`                             | Account API                           | `FE-ACCOUNT-EDIT-TEST-INT-014`        |
| `EMAIL_ALREADY_IN_USE`          | `FE-ACCOUNT-EDIT-EMAIL-*`                             | `AC_API_10`                           | `FE-ACCOUNT-EDIT-TEST-E2E-006`        |
| `PASSWORD_POLICY_VIOLATION`     | `FE-ACCOUNT-EDIT-PASSWORD-*`                          | `AC_API_11`                           | `FE-ACCOUNT-EDIT-TEST-E2E-008`        |
| Authentication `401`            | `FE-ACCOUNT-EDIT-REQ-011`                             | Authentication domain                 | `FE-ACCOUNT-EDIT-TEST-E2E-012`        |
| Authorization `403`             | `FE-ACCOUNT-EDIT-REQ-012`                             | Authorization domain                  | `FE-ACCOUNT-EDIT-TEST-E2E-013`        |
| Admin Shell                     | `FE-ACCOUNT-EDIT-ROUTE-001`, `FE-ACCOUNT-EDIT-RESP-*` | `admin_shell.md`                      | `FE-ACCOUNT-EDIT-TEST-E2E-001`        |
| Account Management return state | `FE-ACCOUNT-EDIT-ACTION-001`                          | Account Management design             | `FE-ACCOUNT-EDIT-TEST-E2E-015`        |

### Administrator Content Requirement Boundary

The application-level `CNT-ADMIN-*` requirements describe administrator account data requirements broadly.

They must not be interpreted here as Edit-page field-required rules because the Account API explicitly defines:

```text
display_name → nullable
```

and the Account Edit page intentionally allows clearing `display_name`.

The password credential requirement likewise does not mean that a password change is mandatory every time the Edit page is opened.

Therefore, the `CNT-ADMIN-*` requirements are not used as direct Edit-page field-required traceability constraints.

## Domain References

| Domain             | Relevant References                                                                                         |
| ------------------ | ----------------------------------------------------------------------------------------------------------- |
| Requirements       | `ADM-AUTH-001`, `ADM-AUTH-005`, `SCP-009`                                                                   |
| Sitemap            | `PAGE-ADM-010`, `/admin/accounts/:id/edit`                                                                  |
| Account            | `AC_UC_03`, `AC_UC_04`, `AC_UC_10`, `AC_UC_11`, `AC_API_03`, `AC_API_04`, `AC_API_10`, `AC_API_11`          |
| Authentication     | Protected route behavior, bearer authentication, `401` handling, HTTPS/TLS requirements                     |
| Authorization      | `account:view`, `account:update`, `account:change_email`, `account:change_password`, deny-by-default, `403` |
| Admin Shell        | Shell layout, route protection, Accounts navigation, responsive behavior                                    |
| Account Management | `/admin/accounts`, originating query state, lifecycle action ownership                                      |
| Account Detail     | `/admin/accounts/:id`, read-only account information                                                        |
| Account Delete     | Destructive deletion ownership                                                                              |

## Cross-Document Dependencies

```text
../../sitemap.md
    → PAGE-ADM-010
    → /admin/accounts/:id/edit

./admin_account_management_page.md
    → provides navigation into Account Edit
    → owns lifecycle actions
    → supplies originating Account List context

./admin_account_detail_page.md
    → provides read-only Account information
    → provides Edit navigation

../account.md
    → owns Account field mutation contracts
    → owns email mutation contract
    → owns password mutation contract

../authentication.md
    → owns authentication state
    → owns bearer token lifecycle
    → owns session behavior

../authorization.md
    → owns permission evaluation
    → owns role and permission policy
```

## Design Boundary

```text
Requirements
    ↓
Sitemap
    ↓
Admin Shell
    ↓
Account Management
    ↓
Account Detail
    ↓
Account Edit
    ├── Display Name
    │      ↓
    │   AC_API_04
    │
    ├── Change Email
    │      ↓
    │   AC_API_10
    │
    └── Change Password
           ↓
        AC_API_11
    ↓
Authentication
    ↓
Authorization
    ↓
Account Domain
    ↓
Tests
```

---

# 20. Standards and Technology References

The Account Edit page follows the repository Admin Application stack:

```text
React
Vite
TypeScript
Native CSS
```

## Official and Industry Guidance

| ID                        | Reference                          | Application                                                                                                                  |
| ------------------------- | ---------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-EDIT-STD-001` | React DOM Components               | Use native HTML form controls through React rather than recreating native input behavior.                                    |
| `FE-ACCOUNT-EDIT-STD-002` | React `<input>` reference          | Use semantic input types, explicit labels, controlled values where appropriate, and explicit button types.                   |
| `FE-ACCOUNT-EDIT-STD-003` | Vite TypeScript guidance           | Keep type checking in the repository's explicit `tsc --noEmit` workflow; Vite transpilation is not type checking.            |
| `FE-ACCOUNT-EDIT-STD-004` | TypeScript Handbook                | Keep Account models, API payloads, route state, and mutation results explicitly typed.                                       |
| `FE-ACCOUNT-EDIT-STD-005` | WCAG 2.2                           | Apply semantic labels, error identification, status messages, visible focus, focus visibility, and target-size requirements. |
| `FE-ACCOUNT-EDIT-STD-006` | MDN `autocomplete`                 | Use `new-password` for the new password field and native form autocomplete semantics.                                        |
| `FE-ACCOUNT-EDIT-STD-007` | URL / URLSearchParams browser APIs | Parse and validate `return_to` using URL APIs instead of manual query-string parsing.                                        |
| `FE-ACCOUNT-EDIT-STD-008` | OWASP Authentication guidance      | Treat password changes as sensitive operations, require secure transport, and avoid exposing credentials.                    |
| `FE-ACCOUNT-EDIT-STD-009` | Testing Library query guidance     | Prefer semantic queries such as roles and labels when testing accessible UI.                                                 |
| `FE-ACCOUNT-EDIT-STD-010` | Playwright accessibility guidance  | Combine automated accessibility checks with manual accessibility verification.                                               |

## Normative Rules

The Account Edit page must:

1. Render native semantic form controls.
2. Keep display-name, email, and password mutations in separate forms.
3. Use the backend API contracts exactly as defined.
4. Treat server validation and authorization as authoritative.
5. Never render or persist password credentials outside the password control's short-lived in-memory state.
6. Never put passwords, tokens, or credentials in URLs.
7. Preserve safe Account List return navigation through a validated same-origin `return_to` value containing only supported Account List query parameters.
8. Keep lifecycle and deletion operations outside Account Edit.
9. Provide accessible field labels and error associations.
10. Provide accessible loading, pending, success, and error states.
11. Keep focused controls visible across responsive layouts.
12. Use `autocomplete="new-password"` for the password-change field.
13. Avoid unsupported client-only password composition rules.
14. Use `type="email"` for semantic email input behavior without allowing native browser constraint validation to block server-supported Unicode local-parts or IDN domains.
15. Avoid reimplementing backend email normalization as an independent authority.
16. Avoid client-side permission checks as a security boundary.
17. Keep the page implementation-local to the existing React, TypeScript, Vite, and native CSS architecture.

## Implementation Status Boundary

The current frontend contains the Account Edit route and the complete Edit-page implementation required by this document.

The implementation provides:

- server-authoritative Account loading through `GET /admin/accounts/{id}`;
- editable `display_name` through `PATCH /admin/accounts/{id}`;
- dedicated Change Email through `PATCH /admin/accounts/{id}/email`;
- dedicated Change Password through `PATCH /admin/accounts/{id}/password`;
- explicit `EMAIL_ALREADY_IN_USE` handling;
- explicit `PASSWORD_POLICY_VIOLATION` handling;
- successful email updates using the returned Account representation;
- successful password changes with a cleared password field;
- validated same-origin `return_to` handling with an allowlist of supported Account List query parameters;
- lifecycle, delete, purge, and role-management operations excluded from Account Edit;
- accessible field labels, error associations, status messaging, native password semantics, and target sizing.

This document remains the frontend Account Edit source of truth.
