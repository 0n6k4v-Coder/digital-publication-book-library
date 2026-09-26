# Frontend Admin Account Detail Page

## Table of Contents

1. [Scope](#1-scope)
2. [Routes](#2-routes)
3. [Requirements](#3-requirements)
4. [Page Structure](#4-page-structure)
5. [Account Data Contract](#5-account-data-contract)
6. [Account Editing](#6-account-editing)
7. [Account Lifecycle Actions](#7-account-lifecycle-actions)
8. [API Contract](#8-api-contract)
9. [UI States](#9-ui-states)
10. [Responsive Layout](#10-responsive-layout)
11. [Security](#11-security)
12. [Accessibility](#12-accessibility)
13. [Component Structure](#13-component-structure)
14. [Testing](#14-testing)
15. [Implementation Criteria](#15-implementation-criteria)
16. [Traceability](#16-traceability)

---

# 1. Scope

This document defines the protected **Administrator Account Detail / Edit** frontend page.

The page is the detail/edit destination from the Administrator Account Management page.

The canonical route is:

```text
/admin/accounts/:id/edit
```

The page is rendered inside the existing [Admin Shell](./admin_shell.md).

The page must use the existing Account domain, Authentication domain, and Authorization domain contracts. It must not introduce a separate client-side account model, authorization model, or lifecycle implementation.

## Responsibilities

| ID | Responsibility |
|---|---|
| `FE-ACCOUNT-DETAIL-SCOPE-001` | Load one administrator account by ID. |
| `FE-ACCOUNT-DETAIL-SCOPE-002` | Display authoritative administrator account information. |
| `FE-ACCOUNT-DETAIL-SCOPE-003` | Allow the supported Account field to be edited. |
| `FE-ACCOUNT-DETAIL-SCOPE-004` | Display account lifecycle state. |
| `FE-ACCOUNT-DETAIL-SCOPE-005` | Support applicable administrator lifecycle actions. |
| `FE-ACCOUNT-DETAIL-SCOPE-006` | Refresh authoritative Account data after successful mutations. |
| `FE-ACCOUNT-DETAIL-SCOPE-007` | Preserve Admin Shell navigation and authentication behavior. |
| `FE-ACCOUNT-DETAIL-SCOPE-008` | Handle authentication, authorization, validation, not-found, conflict, server, and network errors. |
| `FE-ACCOUNT-DETAIL-SCOPE-009` | Provide keyboard-accessible and responsive interaction. |

## Out of Scope

| ID | Excluded Area |
|---|---|
| `FE-ACCOUNT-DETAIL-OOS-001` | Authentication implementation. |
| `FE-ACCOUNT-DETAIL-OOS-002` | Authorization implementation. |
| `FE-ACCOUNT-DETAIL-OOS-003` | Role management. |
| `FE-ACCOUNT-DETAIL-OOS-004` | Password hashing or password validation implementation. |
| `FE-ACCOUNT-DETAIL-OOS-005` | Database persistence or transaction management. |
| `FE-ACCOUNT-DETAIL-OOS-006` | Client-side authorization enforcement. |
| `FE-ACCOUNT-DETAIL-OOS-007` | Hard-delete UI. |
| `FE-ACCOUNT-DETAIL-OOS-008` | Search or unsupported Account filtering. |
| `FE-ACCOUNT-DETAIL-OOS-009` | Client-side Account data persistence. |

Email and password credentials have dedicated backend operations. They are not part of the primary Account field edit operation defined by `AC_UC_04`.

---

# 2. Routes

| ID | Route | Access | Behavior | References |
|---|---|---|---|---|
| `FE-ACCOUNT-DETAIL-ROUTE-001` | `/admin/accounts/:id/edit` | Authenticated | Render Account Detail / Edit page. | `PAGE-ADM-010`, `ADM-AUTH-005` |
| `FE-ACCOUNT-DETAIL-ROUTE-002` | `/admin/accounts/:id/edit` | Bootstrap | Keep route pending until authentication resolves. | Authentication domain |
| `FE-ACCOUNT-DETAIL-ROUTE-003` | `/admin/accounts/:id/edit` | Unauthenticated | Redirect to `/login`. | Admin Shell |
| `FE-ACCOUNT-DETAIL-ROUTE-004` | `/admin/accounts/:id/edit` | Authenticated + unauthorized | Render authorization error while preserving authentication state. | Authorization domain |
| `FE-ACCOUNT-DETAIL-ROUTE-005` | `/admin/accounts/:id/edit` | Invalid or unavailable Account | Render Account Not Found state. | `AC_API_03` |

The page must render inside the Admin Shell:

```text
/admin/accounts/:id/edit
    └── Admin Shell
        ├── Sidebar
        │   └── Accounts = active
        └── Main Content
            └── Administrator Account Detail / Edit
```

The `id` route parameter is the Account UUID.

Authentication tokens, refresh credentials, or other secrets must never appear in the route.

---

# 3. Requirements

| ID | Requirement | Repository Reference |
|---|---|---|
| `FE-ACCOUNT-DETAIL-REQ-001` | Provide a protected Account detail/edit route. | `ADM-AUTH-001`, `ADM-AUTH-005`, `PAGE-ADM-010` |
| `FE-ACCOUNT-DETAIL-REQ-002` | Load the Account using the Account API. | `AC_UC_03`, `AC_API_03` |
| `FE-ACCOUNT-DETAIL-REQ-003` | Display the authoritative Account representation. | Account API |
| `FE-ACCOUNT-DETAIL-REQ-004` | Allow `display_name` to be edited. | `AC_UC_04`, `AC_API_04` |
| `FE-ACCOUNT-DETAIL-REQ-005` | Reject unsupported client-side Account fields. | `AC_UC_04` |
| `FE-ACCOUNT-DETAIL-REQ-006` | Preserve the backend as the authority for validation. | Authorization and Account domains |
| `FE-ACCOUNT-DETAIL-REQ-007` | Display `active` and `inactive` lifecycle states. | `AC_REQ_FC_05` |
| `FE-ACCOUNT-DETAIL-REQ-008` | Display soft-deleted state when returned by an applicable response. | Account domain |
| `FE-ACCOUNT-DETAIL-REQ-009` | Support deactivation when applicable. | `AC_UC_05`, `AC_API_05`, `ADM-AUTH-006` |
| `FE-ACCOUNT-DETAIL-REQ-010` | Support activation when applicable. | `AC_UC_06`, `AC_API_06` |
| `FE-ACCOUNT-DETAIL-REQ-011` | Support restoration when applicable. | `AC_UC_08`, `AC_API_08` |
| `FE-ACCOUNT-DETAIL-REQ-012` | Do not expose hard delete as a routine page action. | `FE-ACCOUNT-DETAIL-OOS-007` |
| `FE-ACCOUNT-DETAIL-REQ-013` | Treat `401 Unauthorized` as an authentication failure. | Authentication domain |
| `FE-ACCOUNT-DETAIL-REQ-014` | Treat `403 Forbidden` as an authorization failure. | Authorization domain |
| `FE-ACCOUNT-DETAIL-REQ-015` | Handle Account mutation conflicts and refresh authoritative state. | `LAST_ACTIVE_ADMINISTRATOR` and Account API |
| `FE-ACCOUNT-DETAIL-REQ-016` | Prevent duplicate mutation submissions. | Frontend behavior |
| `FE-ACCOUNT-DETAIL-REQ-017` | Do not persist Account data in browser storage. | Account API `no-store` contract |
| `FE-ACCOUNT-DETAIL-REQ-018` | Do not expose password or password hash data. | Account Security |
| `FE-ACCOUNT-DETAIL-REQ-019` | Provide visible focus states and keyboard interaction. | Admin Shell accessibility |
| `FE-ACCOUNT-DETAIL-REQ-020` | Preserve the Admin Shell across viewport sizes. | Admin Shell responsive requirements |

---

# 4. Page Structure

## 4.1 Page Header

The page header should use the following structure:

```text
Administrator Accounts
← Back to Administrator Accounts

Administrator Account
Library Administrator
admin@example.com                         [Active]

[Save Changes]    [Deactivate]
```

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-UI-001` | Use `Administrator Account` as the primary page heading. |
| `FE-ACCOUNT-DETAIL-UI-002` | Provide a supporting account identifier using `display_name` when available. |
| `FE-ACCOUNT-DETAIL-UI-003` | Display the Account email in the page header summary. |
| `FE-ACCOUNT-DETAIL-UI-004` | Display lifecycle status using text and a non-color-only status treatment. |
| `FE-ACCOUNT-DETAIL-UI-005` | Provide a link back to `/admin/accounts`. |
| `FE-ACCOUNT-DETAIL-UI-006` | Place page-level actions in a clearly grouped action region. |
| `FE-ACCOUNT-DETAIL-UI-007` | Keep page actions inside the Main Content Area. |

The Back action must use a real link:

```text
/admin/accounts
```

It must not use a clickable `div` or a button that simulates navigation.

---

## 4.2 Account Information

The main Account information section should use a native HTML form.

```text
Account Information

Display name
[ Library Administrator                         ]

Email
admin@example.com

Account ID
019...

[Save Changes]
```

| ID | Field | Source | Editable |
|---|---|---|---|
| `FE-ACCOUNT-DETAIL-FIELD-001` | Display name | `display_name` | Yes |
| `FE-ACCOUNT-DETAIL-FIELD-002` | Email | `email` | No |
| `FE-ACCOUNT-DETAIL-FIELD-003` | Account ID | `id` | No |

The form must only submit Account fields supported by `AC_UC_04`.

The current Account domain defines exactly one mutable Account field for this operation:

```text
display_name
```

Email changes must use the dedicated Change Email operation and are therefore not included in the primary Account update payload.

---

## 4.3 Display Name

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-DISPLAY-001` | Render a visible label for the Display name field. |
| `FE-ACCOUNT-DETAIL-DISPLAY-002` | Populate the field from the server `display_name` value. |
| `FE-ACCOUNT-DETAIL-DISPLAY-003` | Represent a server `null` value as an empty form field. |
| `FE-ACCOUNT-DETAIL-DISPLAY-004` | Allow the administrator to clear an existing display name. |
| `FE-ACCOUNT-DETAIL-DISPLAY-005` | Do not perform client-side Unicode normalization that differs from the backend contract. |
| `FE-ACCOUNT-DETAIL-DISPLAY-006` | Preserve internal whitespace and case. |
| `FE-ACCOUNT-DETAIL-DISPLAY-007` | Reject an empty submitted value only when the backend contract requires a non-null string. |
| `FE-ACCOUNT-DETAIL-DISPLAY-008` | Rely on the backend for authoritative validation and normalization. |

The frontend may provide lightweight usability validation, but it must not replace backend validation.

The backend contract defines:

```text
trim surrounding Unicode whitespace
normalize to Unicode NFC
1–100 Unicode scalar values
```

---

## 4.4 Account Metadata

The page should display non-editable Account metadata:

```text
Account Details

Status
Active

Account ID
019...

Created
September 23, 2026, 17:00

Last updated
September 23, 2026, 17:30

Deleted
—
```

| ID | Metadata |
|---|---|
| `FE-ACCOUNT-DETAIL-META-001` | Display `status`. |
| `FE-ACCOUNT-DETAIL-META-002` | Display `id`. |
| `FE-ACCOUNT-DETAIL-META-003` | Display `created_at`. |
| `FE-ACCOUNT-DETAIL-META-004` | Display `updated_at`. |
| `FE-ACCOUNT-DETAIL-META-005` | Display `deleted_at` when non-null. |
| `FE-ACCOUNT-DETAIL-META-006` | Never display `deleted_by`, `created_by`, `updated_by`, password hashes, or authentication credentials. |

Dates must be presented in the browser's localized date/time representation.

The underlying timestamp must remain available to assistive technologies and machine-readable consumers through the HTML `datetime` value where applicable.

---

## 4.5 Lifecycle Summary

The page must communicate lifecycle state explicitly.

| Condition | Display |
|---|---|
| `status=active`, `deleted_at=null` | `Active` |
| `status=inactive`, `deleted_at=null` | `Inactive` |
| `deleted_at != null` | `Deleted` |

`Deleted` is a derived UI state and is not an Account `status` value.

The UI must never represent `deleted` as a third server-side Account status.

---

# 5. Account Data Contract

The detail page consumes the authoritative Account response.

Expected representation:

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

## Account Fields

| ID | Field | Type | Frontend Rule |
|---|---|---|---|
| `FE-ACCOUNT-DETAIL-DATA-001` | `id` | UUID string | Display as read-only identifier. |
| `FE-ACCOUNT-DETAIL-DATA-002` | `email` | string | Display as read-only Account credential identifier. |
| `FE-ACCOUNT-DETAIL-DATA-003` | `display_name` | string or null | Render as editable form field. |
| `FE-ACCOUNT-DETAIL-DATA-004` | `status` | string | Render as lifecycle state. |
| `FE-ACCOUNT-DETAIL-DATA-005` | `created_at` | RFC 3339 string | Render as localized timestamp. |
| `FE-ACCOUNT-DETAIL-DATA-006` | `updated_at` | RFC 3339 string | Render as localized timestamp. |
| `FE-ACCOUNT-DETAIL-DATA-007` | `deleted_at` | RFC 3339 string or null | Render when non-null. |

The frontend must never expect the Account response to contain:

```text
password
password_hash
```

Unknown fields must not be rendered automatically.

The server response is authoritative. The frontend must not fabricate missing values.

---

# 6. Account Editing

## 6.1 Edit Contract

The primary edit operation is:

```http
PATCH /admin/accounts/{id}
Content-Type: application/merge-patch+json
Authorization: Bearer <access-token>
```

Request body:

```json
{
  "display_name": "Library Administrator"
}
```

To clear the display name:

```json
{
  "display_name": null
}
```

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-EDIT-001` | Submit only supported Account fields. |
| `FE-ACCOUNT-DETAIL-EDIT-002` | Use `application/merge-patch+json`. |
| `FE-ACCOUNT-DETAIL-EDIT-003` | Send the Account ID only as the path parameter. |
| `FE-ACCOUNT-DETAIL-EDIT-004` | Never send credentials in the request URL. |
| `FE-ACCOUNT-DETAIL-EDIT-005` | Never send `password` or `password_hash`. |
| `FE-ACCOUNT-DETAIL-EDIT-006` | Disable duplicate submissions while the update request is pending. |
| `FE-ACCOUNT-DETAIL-EDIT-007` | Treat the successful `200 OK` response as the new authoritative Account state. |
| `FE-ACCOUNT-DETAIL-EDIT-008` | Refresh the displayed Account state after a successful mutation. |

The response is:

```text
200 OK
```

with the updated Account representation.

---

## 6.2 Dirty State

The page should distinguish:

```text
Pristine
Dirty
Submitting
Success
Error
```

| ID | Behavior |
|---|---|
| `FE-ACCOUNT-DETAIL-EDIT-009` | Disable Save Changes while no editable value has changed. |
| `FE-ACCOUNT-DETAIL-EDIT-010` | Enable Save Changes when a supported value differs from the loaded value. |
| `FE-ACCOUNT-DETAIL-EDIT-011` | Disable form mutation controls while the update request is pending. |
| `FE-ACCOUNT-DETAIL-EDIT-012` | Restore the pristine state after a successful update. |
| `FE-ACCOUNT-DETAIL-EDIT-013` | Preserve user-entered values when validation fails. |
| `FE-ACCOUNT-DETAIL-EDIT-014` | Do not optimistically replace the authoritative Account representation before the server confirms the mutation. |

No account data should be persisted into browser storage to preserve an unsaved draft.

---

## 6.3 Validation

Validation errors use the backend Problem Details response.

The frontend may provide immediate field-level feedback for obvious input problems, but backend validation remains authoritative.

| ID | Error | Presentation |
|---|---|---|
| `FE-ACCOUNT-DETAIL-VAL-001` | Invalid display name | Inline error associated with Display name. |
| `FE-ACCOUNT-DETAIL-VAL-002` | Empty unsupported patch | Preserve form and show actionable error. |
| `FE-ACCOUNT-DETAIL-VAL-003` | Unsupported field | Treat as developer/API contract error rather than displaying raw server internals. |
| `FE-ACCOUNT-DETAIL-VAL-004` | Unsupported media type | Show a generic request-format error. |

---

# 7. Account Lifecycle Actions

Lifecycle actions are displayed according to the authoritative Account state.

Routine hard deletion is not exposed.

## 7.1 Active Account

```text
Status: Active

[Save Changes]    [Deactivate]
```

| ID | Action | API |
|---|---|---|
| `FE-ACCOUNT-DETAIL-ACTION-001` | Deactivate | `POST /admin/accounts/{id}/deactivate` |

The Deactivate action is available only when:

```text
status=active
deleted_at=null
```

The server remains authoritative for whether deactivation is permitted.

---

## 7.2 Inactive Account

```text
Status: Inactive

[Save Changes]    [Activate]
```

| ID | Action | API |
|---|---|---|
| `FE-ACCOUNT-DETAIL-ACTION-002` | Activate | `POST /admin/accounts/{id}/activate` |

The Activate action is available only when:

```text
status=inactive
deleted_at=null
```

---

## 7.3 Deleted Account

If a deleted Account is returned by an authorized API response, the page must display:

```text
Status: Deleted

[Restore]
```

| ID | Action | API |
|---|---|---|
| `FE-ACCOUNT-DETAIL-ACTION-003` | Restore | `POST /admin/accounts/{id}/restore` |

Restoration results in:

```text
status = inactive
deleted_at = null
deleted_by = null
```

The page must not automatically activate the restored Account.

---

## 7.4 Mutation Rules

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-ACTION-004` | Prevent duplicate lifecycle requests. |
| `FE-ACCOUNT-DETAIL-ACTION-005` | Disable the pending lifecycle action while its request is running. |
| `FE-ACCOUNT-DETAIL-ACTION-006` | Do not use optimistic lifecycle state changes. |
| `FE-ACCOUNT-DETAIL-ACTION-007` | Use the successful API response as the authoritative new state when returned. |
| `FE-ACCOUNT-DETAIL-ACTION-008` | Refresh the Account after a successful `204 No Content` operation. |
| `FE-ACCOUNT-DETAIL-ACTION-009` | Preserve the user on the detail page after a successful mutation. |
| `FE-ACCOUNT-DETAIL-ACTION-010` | Announce mutation success through accessible status feedback. |

---

## 7.5 Last Active Administrator Conflict

Deactivation and soft-delete operations may fail with:

```text
409 Conflict
LAST_ACTIVE_ADMINISTRATOR
```

The UI must:

1. Preserve authentication state.
2. Display an actionable conflict message.
3. Refresh the Account from the server.
4. Re-evaluate the lifecycle action state from the refreshed response.
5. Never claim that the Account was deactivated when the request failed.

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-CONFLICT-001` | Handle `LAST_ACTIVE_ADMINISTRATOR`. |
| `FE-ACCOUNT-DETAIL-CONFLICT-002` | Refresh authoritative Account state after the conflict. |
| `FE-ACCOUNT-DETAIL-CONFLICT-003` | Keep the error within the page Main Content Area. |

---

# 8. API Contract

## 8.1 View Account

```http
GET /admin/accounts/{id}
Authorization: Bearer <access-token>
```

| ID | Contract |
|---|---|
| `FE-ACCOUNT-DETAIL-API-001` | Use `GET /admin/accounts/{id}`. |
| `FE-ACCOUNT-DETAIL-API-002` | Send bearer credentials through the `Authorization` header. |
| `FE-ACCOUNT-DETAIL-API-003` | Do not send tokens in query parameters. |
| `FE-ACCOUNT-DETAIL-API-004` | Do not send tokens in request bodies. |
| `FE-ACCOUNT-DETAIL-API-005` | Treat the returned Account as authoritative. |

The normal View Account endpoint does not return soft-deleted Accounts.

---

## 8.2 Authorization

Viewing an Account requires:

```text
account:view
```

Updating requires:

```text
account:update
```

Lifecycle operations require the corresponding server-side permission.

| Operation | Permission |
|---|---|
| View Account | `account:view` |
| Update Account | `account:update` |
| Deactivate | `account:deactivate` |
| Activate | `account:activate` |
| Restore | `account:restore` |

The frontend must not treat client-held roles or permissions as proof of authorization.

The backend is the authorization authority.

---

## 8.3 Update Account

```http
PATCH /admin/accounts/{id}
Authorization: Bearer <access-token>
Content-Type: application/merge-patch+json
```

Supported body:

```json
{
  "display_name": "Library Administrator"
}
```

Success:

```text
200 OK
```

| ID | Error | UI Behavior |
|---|---|---|
| `FE-ACCOUNT-DETAIL-API-006` | `400 INVALID_ACCOUNT_ID` | Render a non-editable invalid-resource error. |
| `FE-ACCOUNT-DETAIL-API-007` | `401` | Clear authentication state and redirect through Admin Shell behavior. |
| `FE-ACCOUNT-DETAIL-API-008` | `403` | Preserve authentication and show authorization error. |
| `FE-ACCOUNT-DETAIL-API-009` | `404 ACCOUNT_NOT_FOUND` | Show Not Found state. |
| `FE-ACCOUNT-DETAIL-API-010` | `415 UNSUPPORTED_MEDIA_TYPE` | Show generic request-format error. |
| `FE-ACCOUNT-DETAIL-API-011` | `422 VALIDATION_ERROR` | Show field/form validation feedback. |
| `FE-ACCOUNT-DETAIL-API-012` | `500` | Show non-sensitive server error and retry action. |
| `FE-ACCOUNT-DETAIL-API-013` | Network failure | Show network error and retry action. |

---

## 8.4 Deactivate

```http
POST /admin/accounts/{id}/deactivate
Authorization: Bearer <access-token>
```

Success:

```text
200 OK
```

Expected response:

```json
{
  "id": "019...",
  "status": "inactive"
}
```

Potential conflicts:

```text
ACCOUNT_ALREADY_INACTIVE
LAST_ACTIVE_ADMINISTRATOR
```

The UI must display the server-provided error meaning without exposing internal implementation details.

---

## 8.5 Activate

```http
POST /admin/accounts/{id}/activate
Authorization: Bearer <access-token>
```

Success:

```text
200 OK
```

Potential conflicts:

```text
ACCOUNT_ALREADY_ACTIVE
ACCOUNT_SOFT_DELETED
```

If the Account becomes unavailable during the operation, the UI must refresh or transition to the Not Found state according to the authoritative response.

---

## 8.6 Restore

```http
POST /admin/accounts/{id}/restore
Authorization: Bearer <access-token>
```

Success:

```text
200 OK
```

Expected lifecycle result:

```text
status = inactive
deleted_at = null
```

The page must not automatically activate the restored Account.

---

## 8.7 Cache

Account responses use:

```http
Cache-Control: no-store
```

The frontend must not persist Account data in:

```text
localStorage
sessionStorage
IndexedDB
```

The page must not use cached Account information as evidence for authentication or authorization.

---

## 8.8 Problem Details

Errors use:

```text
application/problem+json
```

The frontend should parse:

```json
{
  "type": "...",
  "title": "...",
  "status": 409,
  "detail": "...",
  "code": "LAST_ACTIVE_ADMINISTRATOR"
}
```

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-API-014` | Prefer the structured `code` for deterministic application behavior. |
| `FE-ACCOUNT-DETAIL-API-015` | Present user-facing messages appropriate to the operation. |
| `FE-ACCOUNT-DETAIL-API-016` | Do not render raw stack traces or internal server details. |
| `FE-ACCOUNT-DETAIL-API-017` | Preserve generic fallback messaging when the response is malformed or lacks a known code. |

---

# 9. UI States

## 9.1 Bootstrap Pending

The page must not render as unauthenticated merely because authentication state is still `unknown`.

| ID | Behavior |
|---|---|
| `FE-ACCOUNT-DETAIL-STATE-001` | Keep the protected route pending during authentication bootstrap. |
| `FE-ACCOUNT-DETAIL-STATE-002` | Do not flash the login page before authentication resolution. |
| `FE-ACCOUNT-DETAIL-STATE-003` | Preserve Admin Shell authentication behavior. |

---

## 9.2 Initial Loading

While `GET /admin/accounts/{id}` is pending:

```text
Administrator Account

[loading account summary]

Account Information
[loading fields]

Account Details
[loading metadata]
```

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-STATE-004` | Keep the Admin Shell rendered. |
| `FE-ACCOUNT-DETAIL-STATE-005` | Show an explicit loading state in Main Content. |
| `FE-ACCOUNT-DETAIL-STATE-006` | Do not display invented Account values. |
| `FE-ACCOUNT-DETAIL-STATE-007` | Do not enable mutation controls before Account data is loaded. |

---

## 9.3 Loaded

The loaded page contains:

```text
Back to Administrator Accounts

Administrator Account
Display Name
Email
Status

Account Information
Account Details
Lifecycle Actions
```

The page is interactive only when its required data is available.

---

## 9.4 Saving

During `PATCH /admin/accounts/{id}`:

```text
Display name
[ Library Administrator                  ]

[Saving...]
```

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-STATE-008` | Disable Save Changes while the request is pending. |
| `FE-ACCOUNT-DETAIL-STATE-009` | Keep the edited form value visible. |
| `FE-ACCOUNT-DETAIL-STATE-010` | Prevent duplicate form submission. |
| `FE-ACCOUNT-DETAIL-STATE-011` | Announce the pending state accessibly. |

---

## 9.5 Mutation Success

After a successful mutation:

```text
Account updated successfully.
```

The Account representation must be refreshed or replaced with the authoritative successful response.

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-STATE-012` | Announce success using an accessible status mechanism. |
| `FE-ACCOUNT-DETAIL-STATE-013` | Remove stale error messages. |
| `FE-ACCOUNT-DETAIL-STATE-014` | Recalculate available lifecycle actions from authoritative state. |

---

## 9.6 Not Found

When the Account is unavailable:

```text
Administrator Account

Account not found.
The administrator account may no longer exist or may not be available.

[Back to Administrator Accounts]
```

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-STATE-015` | Explain that the requested Account is unavailable. |
| `FE-ACCOUNT-DETAIL-STATE-016` | Provide a direct route back to `/admin/accounts`. |
| `FE-ACCOUNT-DETAIL-STATE-017` | Do not display stale Account details as current data. |

---

## 9.7 Authorization Error

For `403 Forbidden`:

```text
Administrator Account

You are not authorized to perform this action.
```

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-STATE-018` | Preserve authenticated state. |
| `FE-ACCOUNT-DETAIL-STATE-019` | Keep the error inside Main Content. |
| `FE-ACCOUNT-DETAIL-STATE-020` | Do not redirect to `/login` for `403`. |
| `FE-ACCOUNT-DETAIL-STATE-021` | Do not claim that authorization succeeded. |

---

## 9.8 Authentication Expiry

For `401 Unauthorized`:

```text
Authentication has expired. Redirecting to login.
```

The page must rely on the existing Authentication service to clear authentication state and navigate to `/login`.

The Account detail page must not implement an independent authentication system.

---

## 9.9 General Error

```text
Unable to load this administrator account.

[Try Again]
```

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-STATE-022` | Provide a retry action for retryable failures. |
| `FE-ACCOUNT-DETAIL-STATE-023` | Keep server and network errors inside Main Content. |
| `FE-ACCOUNT-DETAIL-STATE-024` | Do not expose internal diagnostics. |
| `FE-ACCOUNT-DETAIL-STATE-025` | Preserve the Admin Shell. |

---

# 10. Responsive Layout

The page must inherit the Admin Shell responsive behavior and manage its own content layout.

| ID | Viewport | Layout |
|---|---|---|
| `FE-ACCOUNT-DETAIL-RESP-001` | `>= 1280px` | Two-column detail layout may be used for metadata and Account information. |
| `FE-ACCOUNT-DETAIL-RESP-002` | `768px–1279px` | Reduce content width and maintain readable form controls. |
| `FE-ACCOUNT-DETAIL-RESP-003` | `< 768px` | Stack all sections vertically. |
| `FE-ACCOUNT-DETAIL-RESP-004` | `< 768px` | Keep primary actions reachable without horizontal scrolling. |
| `FE-ACCOUNT-DETAIL-RESP-005` | All viewports | Prevent unintended page-level horizontal scrolling. |
| `FE-ACCOUNT-DETAIL-RESP-006` | All viewports | Preserve keyboard accessibility. |
| `FE-ACCOUNT-DETAIL-RESP-007` | All viewports | Preserve authentication and authorization behavior. |

Suggested responsive structure:

```text
Desktop

┌───────────────────────────────────────────────────────────┐
│ Back to Administrator Accounts                            │
│                                                           │
│ Administrator Account                     [Actions]       │
│                                                           │
│ ┌──────────────────────────┐ ┌──────────────────────────┐ │
│ │ Account Information      │ │ Account Details          │ │
│ │                          │ │                          │ │
│ │ Display name             │ │ Status                   │ │
│ │ Email                    │ │ Account ID               │ │
│ │                          │ │ Created                  │ │
│ │ [Save Changes]           │ │ Updated                  │ │
│ └──────────────────────────┘ └──────────────────────────┘ │
└───────────────────────────────────────────────────────────┘
```

```text
Mobile

Administrator Account
Display Name
Email
Status

Account Information
Display name
Email
[Save Changes]

Account Details
Status
Account ID
Created
Updated

Lifecycle Actions
[Deactivate]
```

The exact visual styling must use the existing native CSS architecture and design conventions established by the frontend application.

---

# 11. Security

The Account Detail page must follow the Authentication, Authorization, and Account security contracts.

| ID | Security Rule | Requirement |
|---|---|---|
| `FE-ACCOUNT-DETAIL-SEC-001` | Authentication | Protected route requires authenticated Admin Shell state. |
| `FE-ACCOUNT-DETAIL-SEC-002` | Authentication authority | Backend remains authoritative for authentication. |
| `FE-ACCOUNT-DETAIL-SEC-003` | Authorization | Backend remains authoritative for authorization. |
| `FE-ACCOUNT-DETAIL-SEC-004` | Access token | Bearer credentials are sent only in the `Authorization` header. |
| `FE-ACCOUNT-DETAIL-SEC-005` | URL secrecy | Tokens never appear in URLs. |
| `FE-ACCOUNT-DETAIL-SEC-006` | UI secrecy | Tokens are never rendered. |
| `FE-ACCOUNT-DETAIL-SEC-007` | Logging | Tokens are never logged. |
| `FE-ACCOUNT-DETAIL-SEC-008` | Credential secrecy | Passwords and password hashes are never rendered. |
| `FE-ACCOUNT-DETAIL-SEC-009` | Client authorization | Client roles and permissions are not trusted as authorization proof. |
| `FE-ACCOUNT-DETAIL-SEC-010` | Account persistence | Account response data is not stored in browser persistence. |
| `FE-ACCOUNT-DETAIL-SEC-011` | Cache | Account responses are treated as `no-store`. |
| `FE-ACCOUNT-DETAIL-SEC-012` | Server confirmation | Mutations require successful server confirmation before the UI treats them as committed. |
| `FE-ACCOUNT-DETAIL-SEC-013` | Error exposure | Internal server diagnostics are not exposed to administrators. |

The frontend must not infer authorization from:

```text
role
permission
account status
route visibility
client state
```

Server-side authorization remains the security boundary.

---

# 12. Accessibility

## 12.1 Semantic Structure

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-A11Y-001` | Use semantic layout elements such as `main`, `header`, `section`, and `form`. |
| `FE-ACCOUNT-DETAIL-A11Y-002` | Provide exactly one primary `h1` for the page. |
| `FE-ACCOUNT-DETAIL-A11Y-003` | Use logical heading hierarchy for Account sections. |
| `FE-ACCOUNT-DETAIL-A11Y-004` | Use native form controls. |
| `FE-ACCOUNT-DETAIL-A11Y-005` | Associate every form control with a visible label. |

---

## 12.2 Navigation

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-A11Y-006` | Back to Accounts uses a native link. |
| `FE-ACCOUNT-DETAIL-A11Y-007` | Action controls use native buttons. |
| `FE-ACCOUNT-DETAIL-A11Y-008` | Keyboard order follows the visual and logical reading order. |
| `FE-ACCOUNT-DETAIL-A11Y-009` | Focus is visibly indicated. |
| `FE-ACCOUNT-DETAIL-A11Y-010` | Focus is not obscured by responsive UI or sticky controls. |

---

## 12.3 Form Errors

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-A11Y-011` | Inline validation errors are programmatically associated with their fields. |
| `FE-ACCOUNT-DETAIL-A11Y-012` | Error messages are understandable without relying on color. |
| `FE-ACCOUNT-DETAIL-A11Y-013` | The invalid field is identifiable without visual guessing. |
| `FE-ACCOUNT-DETAIL-A11Y-014` | Form submission errors remain available to assistive technologies. |

---

## 12.4 Async Status

Mutation and loading state must be communicated to assistive technology.

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-A11Y-015` | Loading state is announced appropriately. |
| `FE-ACCOUNT-DETAIL-A11Y-016` | Mutation success is announced appropriately. |
| `FE-ACCOUNT-DETAIL-A11Y-017` | Mutation errors are announced appropriately. |
| `FE-ACCOUNT-DETAIL-A11Y-018` | Status messages do not steal focus unnecessarily. |

A suitable status region may use:

```html
<div role="status" aria-live="polite"></div>
```

Error messaging may use an appropriate assertive live region when necessary.

---

## 12.5 Status Presentation

Status must not depend on color alone.

Example:

```text
● Active
● Inactive
● Deleted
```

The text label remains the authoritative visual meaning.

---

# 13. Component Structure

The page should remain feature-local and keep Account-specific behavior out of the Admin Shell.

Suggested structure:

```text
src/pages/accounts/
├── AccountListPage
├── AccountCreatePage
└── AccountDetailPage
```

Suggested page-local components:

```text
AccountDetailPage
├── AccountDetailHeader
├── AccountInformationSection
├── AccountDetailsSection
├── AccountLifecycleSection
├── AccountForm
├── AccountStatus
├── AccountActionGroup
├── AccountLoadingState
├── AccountErrorState
└── AccountNotFoundState
```

| ID | Component | Responsibility |
|---|---|---|
| `FE-ACCOUNT-DETAIL-COMP-001` | `AccountDetailPage` | Coordinate route state, loading, data, mutations, and page layout. |
| `FE-ACCOUNT-DETAIL-COMP-002` | `AccountDetailHeader` | Render page title, summary, status, and navigation. |
| `FE-ACCOUNT-DETAIL-COMP-003` | `AccountInformationSection` | Render editable Account fields. |
| `FE-ACCOUNT-DETAIL-COMP-004` | `AccountDetailsSection` | Render non-editable metadata. |
| `FE-ACCOUNT-DETAIL-COMP-005` | `AccountLifecycleSection` | Render applicable lifecycle actions. |
| `FE-ACCOUNT-DETAIL-COMP-006` | `AccountStatus` | Render accessible lifecycle status. |
| `FE-ACCOUNT-DETAIL-COMP-007` | `AccountLoadingState` | Render initial loading state. |
| `FE-ACCOUNT-DETAIL-COMP-008` | `AccountErrorState` | Render non-recoverable and retryable errors. |
| `FE-ACCOUNT-DETAIL-COMP-009` | `AccountNotFoundState` | Render unavailable Account state. |

API calls should remain in the existing frontend service layer:

```text
src/services/
```

Type definitions should remain in:

```text
src/types/
```

The page should use React, TypeScript, and the project's native CSS approach. No page-specific UI framework or large component dependency should be introduced solely for this page.

---

# 14. Testing

The implementation must follow the repository's test strategy:

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

## 14.1 Unit

| ID | Test |
|---|---|
| `FE-ACCOUNT-DETAIL-TEST-UNIT-001` | Account response maps to the detail view correctly. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-002` | `display_name=null` renders as an empty input. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-003` | Dirty form state is detected correctly. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-004` | Duplicate Save submissions are prevented. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-005` | Active Account shows Deactivate. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-006` | Inactive Account shows Activate. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-007` | Deleted Account shows Restore when represented by the API. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-008` | Lifecycle mutations remain pending until server confirmation. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-009` | `401` is delegated to authentication handling. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-010` | `403` preserves authentication and renders an authorization error. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-011` | `404` renders Not Found. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-012` | `LAST_ACTIVE_ADMINISTRATOR` renders the conflict state. |

---

## 14.2 Integration

| ID | Test |
|---|---|
| `FE-ACCOUNT-DETAIL-TEST-INT-001` | Authenticated navigation from Account List to Account Detail works. |
| `FE-ACCOUNT-DETAIL-TEST-INT-002` | Account detail loads from `GET /admin/accounts/{id}`. |
| `FE-ACCOUNT-DETAIL-TEST-INT-003` | Display name update uses `PATCH /admin/accounts/{id}`. |
| `FE-ACCOUNT-DETAIL-TEST-INT-004` | Correct `Content-Type` is sent for merge patch. |
| `FE-ACCOUNT-DETAIL-TEST-INT-005` | Deactivation sends the correct endpoint. |
| `FE-ACCOUNT-DETAIL-TEST-INT-006` | Activation sends the correct endpoint. |
| `FE-ACCOUNT-DETAIL-TEST-INT-007` | Restoration sends the correct endpoint. |
| `FE-ACCOUNT-DETAIL-TEST-INT-008` | Successful mutations update authoritative UI state. |
| `FE-ACCOUNT-DETAIL-TEST-INT-009` | `401` redirects to `/login`. |
| `FE-ACCOUNT-DETAIL-TEST-INT-010` | `403` preserves authentication. |
| `FE-ACCOUNT-DETAIL-TEST-INT-011` | `404` renders Account Not Found. |
| `FE-ACCOUNT-DETAIL-TEST-INT-012` | `409 LAST_ACTIVE_ADMINISTRATOR` refreshes the Account. |
| `FE-ACCOUNT-DETAIL-TEST-INT-013` | Account data is not persisted in browser storage. |
| `FE-ACCOUNT-DETAIL-TEST-INT-014` | Tokens do not appear in URLs or rendered output. |

---

## 14.3 E2E

| ID | Test |
|---|---|
| `FE-ACCOUNT-DETAIL-TEST-E2E-001` | Login → Admin Shell → Accounts works. |
| `FE-ACCOUNT-DETAIL-TEST-E2E-002` | Account row opens the correct Edit Account route. |
| `FE-ACCOUNT-DETAIL-TEST-E2E-003` | Account details render correctly. |
| `FE-ACCOUNT-DETAIL-TEST-E2E-004` | Display name can be changed successfully. |
| `FE-ACCOUNT-DETAIL-TEST-E2E-005` | Active Account can be deactivated when authorized. |
| `FE-ACCOUNT-DETAIL-TEST-E2E-006` | Inactive Account can be activated when authorized. |
| `FE-ACCOUNT-DETAIL-TEST-E2E-007` | Deleted Account can be restored when authorized. |
| `FE-ACCOUNT-DETAIL-TEST-E2E-008` | Last-active-administrator conflict is displayed. |
| `FE-ACCOUNT-DETAIL-TEST-E2E-009` | Authentication expiry redirects to `/login`. |
| `FE-ACCOUNT-DETAIL-TEST-E2E-010` | Unauthorized operations display `403` without logout. |
| `FE-ACCOUNT-DETAIL-TEST-E2E-011` | Mobile layout remains usable below `768px`. |

---

## 14.4 Accessibility

| ID | Test |
|---|---|
| `FE-ACCOUNT-DETAIL-TEST-A11Y-001` | Heading hierarchy is correct. |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-002` | All fields have accessible labels. |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-003` | Back navigation is keyboard accessible. |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-004` | Action buttons are keyboard accessible. |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-005` | Focus is visible. |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-006` | Validation errors are associated with fields. |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-007` | Async status is announced. |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-008` | Account status does not rely on color alone. |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-009` | Responsive layout remains usable with keyboard navigation. |

---

## 14.5 Responsive

| ID | Test |
|---|---|
| `FE-ACCOUNT-DETAIL-TEST-RESP-001` | Desktop layout works at `>= 1280px`. |
| `FE-ACCOUNT-DETAIL-TEST-RESP-002` | Tablet layout works at `768px–1279px`. |
| `FE-ACCOUNT-DETAIL-TEST-RESP-003` | Mobile layout works below `768px`. |
| `FE-ACCOUNT-DETAIL-TEST-RESP-004` | No unintended page-level horizontal scrolling occurs. |
| `FE-ACCOUNT-DETAIL-TEST-RESP-005` | Primary actions remain accessible on mobile. |
| `FE-ACCOUNT-DETAIL-TEST-RESP-006` | Admin Shell behavior is preserved at every viewport size. |
| `FE-ACCOUNT-DETAIL-TEST-RESP-007` | Authentication and authorization behavior is unchanged across viewports. |

---

# 15. Implementation Criteria

## 15.1 Route

| ID | Criteria |
|---|---|
| `FE-ACCOUNT-DETAIL-IMPL-001` | `/admin/accounts/:id/edit` renders inside the Admin Shell. |
| `FE-ACCOUNT-DETAIL-IMPL-002` | Unauthenticated access is handled by Admin Shell authentication behavior. |
| `FE-ACCOUNT-DETAIL-IMPL-003` | Accounts navigation remains active while viewing the detail page. |
| `FE-ACCOUNT-DETAIL-IMPL-004` | Back navigation returns to `/admin/accounts`. |

---

## 15.2 Data

| ID | Criteria |
|---|---|
| `FE-ACCOUNT-DETAIL-IMPL-005` | Detail data comes from `GET /admin/accounts/{id}`. |
| `FE-ACCOUNT-DETAIL-IMPL-006` | Server Account response is authoritative. |
| `FE-ACCOUNT-DETAIL-IMPL-007` | Password credentials are never expected or rendered. |
| `FE-ACCOUNT-DETAIL-IMPL-008` | Account data is not stored in browser persistence. |
| `FE-ACCOUNT-DETAIL-IMPL-009` | Account response cache policy remains `no-store`. |

---

## 15.3 Editing

| ID | Criteria |
|---|---|
| `FE-ACCOUNT-DETAIL-IMPL-010` | `display_name` is the only field submitted through the primary Account update form. |
| `FE-ACCOUNT-DETAIL-IMPL-011` | Update uses `PATCH /admin/accounts/{id}`. |
| `FE-ACCOUNT-DETAIL-IMPL-012` | Update uses `application/merge-patch+json`. |
| `FE-ACCOUNT-DETAIL-IMPL-013` | Duplicate submissions are prevented. |
| `FE-ACCOUNT-DETAIL-IMPL-014` | Successful updates replace or refresh authoritative Account state. |

---

## 15.4 Lifecycle

| ID | Criteria |
|---|---|
| `FE-ACCOUNT-DETAIL-IMPL-015` | Deactivate is available for active, non-deleted Accounts. |
| `FE-ACCOUNT-DETAIL-IMPL-016` | Activate is available for inactive, non-deleted Accounts. |
| `FE-ACCOUNT-DETAIL-IMPL-017` | Restore is available for soft-deleted Accounts returned by the API. |
| `FE-ACCOUNT-DETAIL-IMPL-018` | Hard delete is not exposed. |
| `FE-ACCOUNT-DETAIL-IMPL-019` | Lifecycle actions are confirmed by the server before the UI commits the new state. |
| `FE-ACCOUNT-DETAIL-IMPL-020` | `LAST_ACTIVE_ADMINISTRATOR` is handled as an explicit conflict. |

---

## 15.5 Security

| ID | Criteria |
|---|---|
| `FE-ACCOUNT-DETAIL-IMPL-021` | Backend authentication remains authoritative. |
| `FE-ACCOUNT-DETAIL-IMPL-022` | Backend authorization remains authoritative. |
| `FE-ACCOUNT-DETAIL-IMPL-023` | Tokens are only sent through the `Authorization` header for protected Account APIs. |
| `FE-ACCOUNT-DETAIL-IMPL-024` | Tokens never appear in URLs. |
| `FE-ACCOUNT-DETAIL-IMPL-025` | Tokens never render in the UI. |
| `FE-ACCOUNT-DETAIL-IMPL-026` | Tokens never appear in logs. |
| `FE-ACCOUNT-DETAIL-IMPL-027` | Passwords and password hashes never render. |

---

## 15.6 Accessibility

| ID | Criteria |
|---|---|
| `FE-ACCOUNT-DETAIL-IMPL-028` | Semantic page structure is used. |
| `FE-ACCOUNT-DETAIL-IMPL-029` | Primary page heading is present. |
| `FE-ACCOUNT-DETAIL-IMPL-030` | Form controls have visible labels. |
| `FE-ACCOUNT-DETAIL-IMPL-031` | Navigation and actions are keyboard accessible. |
| `FE-ACCOUNT-DETAIL-IMPL-032` | Focus is visible and not obscured. |
| `FE-ACCOUNT-DETAIL-IMPL-033` | Async status is accessible. |
| `FE-ACCOUNT-DETAIL-IMPL-034` | Account lifecycle status does not rely only on color. |

---

# 16. Traceability

## Requirements → Design → Backend → Test

| Requirement | Frontend Design | Backend Contract | Test |
|---|---|---|---|
| `ADM-AUTH-001` | `FE-ACCOUNT-DETAIL-ROUTE-001` | Authentication domain | `FE-ACCOUNT-DETAIL-TEST-E2E-001` |
| `ADM-AUTH-004` | `FE-ACCOUNT-DETAIL-REQ-002` | `AC_UC_03`, `AC_API_03` | `FE-ACCOUNT-DETAIL-TEST-INT-002` |
| `ADM-AUTH-005` | `FE-ACCOUNT-DETAIL-REQ-004` | `AC_UC_04`, `AC_API_04` | `FE-ACCOUNT-DETAIL-TEST-E2E-004` |
| `ADM-AUTH-006` | `FE-ACCOUNT-DETAIL-ACTION-001` | `AC_UC_05`, `AC_API_05` | `FE-ACCOUNT-DETAIL-TEST-E2E-005` |
| `AC_UC_03` | `FE-ACCOUNT-DETAIL-API-001` | `GET /admin/accounts/{id}` | `FE-ACCOUNT-DETAIL-TEST-INT-002` |
| `AC_UC_04` | `FE-ACCOUNT-DETAIL-EDIT-001` | `PATCH /admin/accounts/{id}` | `FE-ACCOUNT-DETAIL-TEST-E2E-004` |
| `AC_UC_05` | `FE-ACCOUNT-DETAIL-ACTION-001` | Deactivate API | `FE-ACCOUNT-DETAIL-TEST-E2E-005` |
| `AC_UC_06` | `FE-ACCOUNT-DETAIL-ACTION-002` | Activate API | `FE-ACCOUNT-DETAIL-TEST-E2E-006` |
| `AC_UC_08` | `FE-ACCOUNT-DETAIL-ACTION-003` | Restore API | `FE-ACCOUNT-DETAIL-TEST-E2E-007` |
| `account:view` | `FE-ACCOUNT-DETAIL-API-001` | Authorization domain | `FE-ACCOUNT-DETAIL-TEST-INT-010` |
| `account:update` | `FE-ACCOUNT-DETAIL-EDIT-006` | Authorization domain | `FE-ACCOUNT-DETAIL-TEST-INT-010` |
| `account:deactivate` | `FE-ACCOUNT-DETAIL-ACTION-001` | Authorization domain | `FE-ACCOUNT-DETAIL-TEST-E2E-005` |
| `account:activate` | `FE-ACCOUNT-DETAIL-ACTION-002` | Authorization domain | `FE-ACCOUNT-DETAIL-TEST-E2E-006` |
| `account:restore` | `FE-ACCOUNT-DETAIL-ACTION-003` | Authorization domain | `FE-ACCOUNT-DETAIL-TEST-E2E-007` |
| Authentication `401` | `FE-ACCOUNT-DETAIL-API-007` | Authentication domain | `FE-ACCOUNT-DETAIL-TEST-E2E-009` |
| Authorization `403` | `FE-ACCOUNT-DETAIL-API-008` | Authorization domain | `FE-ACCOUNT-DETAIL-TEST-INT-010` |
| `LAST_ACTIVE_ADMINISTRATOR` | `FE-ACCOUNT-DETAIL-CONFLICT-001` | Account invariant | `FE-ACCOUNT-DETAIL-TEST-INT-012` |
| Admin Shell | `FE-ACCOUNT-DETAIL-ROUTE-001`, `FE-ACCOUNT-DETAIL-RESP-*` | `admin_shell.md` | `FE-ACCOUNT-DETAIL-TEST-E2E-001` |

## Domain References

| Domain | Relevant References |
|---|---|
| Requirements | `ADM-AUTH-001` to `ADM-AUTH-007`, `CNT-ADMIN-*`, `SCP-009` |
| Sitemap | `PAGE-ADM-008`, `PAGE-ADM-010` |
| Account | `AC_UC_03`, `AC_UC_04`, `AC_UC_05`, `AC_UC_06`, `AC_UC_08`, `AC_API_03` to `AC_API_08` |
| Authentication | Protected route behavior, bearer authentication, `401` handling |
| Authorization | `account:view`, `account:update`, `account:deactivate`, `account:activate`, `account:restore`, `403` |
| Admin Shell | Shell layout, route protection, active Accounts navigation, responsive behavior |
| Account Management | `/admin/accounts`, Edit Account navigation, lifecycle action semantics |

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
Account Detail / Edit
    ↓
Account API
    ↓
Authentication
    ↓
Authorization
    ↓
Tests
```

The Account Detail / Edit page must remain a frontend representation of the existing backend contracts. It must not create new Account lifecycle rules, authentication rules, authorization rules, or persistence semantics.
