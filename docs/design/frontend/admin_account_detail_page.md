# Frontend Admin Account Detail Page

## Table of Contents

1. [Scope](#1-scope)
2. [Routes](#2-routes)
3. [Requirements](#3-requirements)
4. [Page Structure](#4-page-structure)
5. [Account Data Contract](#5-account-data-contract)
6. [Edit Navigation](#6-edit-navigation)
7. [Delete Navigation](#7-delete-navigation)
8. [Account Detail Action Boundary](#8-account-detail-action-boundary)
9. [API Contract](#9-api-contract)
10. [UI States](#10-ui-states)
11. [Responsive Layout](#11-responsive-layout)
12. [Security](#12-security)
13. [Accessibility](#13-accessibility)
14. [Component Structure](#14-component-structure)
15. [Testing](#15-testing)
16. [Implementation Criteria](#16-implementation-criteria)
17. [Traceability](#17-traceability)
18. [Standards and Technology References](#18-standards-and-technology-references)

---

# 1. Scope

This document defines the protected **Administrator Account Detail** frontend page.

The page displays the authoritative details of one administrator account.

The canonical route is:

```text
/admin/accounts/:id
```

The page is rendered inside the existing [Admin Shell](./admin_shell.md).

The page is read-only.

It does not provide:

* Account field editing.
* Save or update controls.
* Deactivate controls.
* Activate controls.
* Restore controls.
* Soft-delete mutation controls.
* Hard-delete mutation controls.

The page provides:

* Back navigation to Account Management.
* Edit navigation to Account Edit.
* Delete Account navigation to the dedicated Account Delete page.

Delete Account navigation does not perform a deletion mutation.

The Edit navigation leads to:

```text
/admin/accounts/:id/edit
```

The Delete Account navigation leads to:

```text
/admin/accounts/:id/delete
```

## Responsibilities

| ID                            | Responsibility                                                                                 |
| ----------------------------- | ---------------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-SCOPE-001` | Load one administrator account by ID.                                                          |
| `FE-ACCOUNT-DETAIL-SCOPE-002` | Display authoritative administrator account information.                                       |
| `FE-ACCOUNT-DETAIL-SCOPE-003` | Display the account lifecycle state returned by the Account API.                               |
| `FE-ACCOUNT-DETAIL-SCOPE-004` | Provide navigation back to Account Management.                                                 |
| `FE-ACCOUNT-DETAIL-SCOPE-005` | Provide an Edit action that navigates to the Account Edit page.                                |
| `FE-ACCOUNT-DETAIL-SCOPE-006` | Provide a Delete Account action that navigates to the Account Delete page.                     |
| `FE-ACCOUNT-DETAIL-SCOPE-007` | Handle authentication, authorization, not-found, server, and network errors.                   |
| `FE-ACCOUNT-DETAIL-SCOPE-008` | Provide keyboard-accessible and responsive interaction.                                        |
| `FE-ACCOUNT-DETAIL-SCOPE-009` | Preserve Admin Shell navigation and authentication behavior.                                   |
| `FE-ACCOUNT-DETAIL-SCOPE-010` | Preserve originating Account List query state for child-page navigation and return navigation. |

## Out of Scope

| ID                          | Excluded Area                                           |
| --------------------------- | ------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-OOS-001` | Authentication implementation.                          |
| `FE-ACCOUNT-DETAIL-OOS-002` | Authorization implementation.                           |
| `FE-ACCOUNT-DETAIL-OOS-003` | Role management.                                        |
| `FE-ACCOUNT-DETAIL-OOS-004` | Password hashing or password validation implementation. |
| `FE-ACCOUNT-DETAIL-OOS-005` | Database persistence or transaction management.         |
| `FE-ACCOUNT-DETAIL-OOS-006` | Client-side authorization enforcement.                  |
| `FE-ACCOUNT-DETAIL-OOS-007` | Account field editing.                                  |
| `FE-ACCOUNT-DETAIL-OOS-008` | Account lifecycle mutations.                            |
| `FE-ACCOUNT-DETAIL-OOS-009` | Delete confirmation UI.                                 |
| `FE-ACCOUNT-DETAIL-OOS-010` | Delete or purge API invocation.                         |
| `FE-ACCOUNT-DETAIL-OOS-011` | Search or unsupported Account filtering.                |
| `FE-ACCOUNT-DETAIL-OOS-012` | Client-side Account data persistence.                   |

---

# 2. Routes

| ID                            | Route                        | Access                         | Behavior                                                                                 | References                     |
| ----------------------------- | ---------------------------- | ------------------------------ | ---------------------------------------------------------------------------------------- | ------------------------------ |
| `FE-ACCOUNT-DETAIL-ROUTE-001` | `/admin/accounts/:id`        | Authenticated                  | Render read-only Account Detail page.                                                    | `ADM-AUTH-004`                 |
| `FE-ACCOUNT-DETAIL-ROUTE-002` | `/admin/accounts/:id`        | Bootstrap                      | Keep route pending until authentication resolves.                                        | Authentication domain          |
| `FE-ACCOUNT-DETAIL-ROUTE-003` | `/admin/accounts/:id`        | Unauthenticated                | Redirect to `/login`.                                                                    | Admin Shell                    |
| `FE-ACCOUNT-DETAIL-ROUTE-004` | `/admin/accounts/:id`        | Authenticated + unauthorized   | Render authorization error while preserving authentication state.                        | Authorization domain           |
| `FE-ACCOUNT-DETAIL-ROUTE-005` | `/admin/accounts/:id`        | Invalid or unavailable Account | Render Account Not Found state.                                                          | `AC_API_03`                    |
| `FE-ACCOUNT-DETAIL-ROUTE-006` | `/admin/accounts/:id/delete` | Authenticated                  | Navigation target for the dedicated Account Delete page; not rendered by Account Detail. | Account Delete frontend design |

The Account Detail page must render inside the Admin Shell:

```text
/admin/accounts/:id
    └── Admin Shell
        ├── Sidebar
        │   └── Accounts = active
        └── Main Content
            └── Administrator Account Detail
```

The `id` route parameter is the Account UUID.

The page provides an Edit action targeting:

```text
/admin/accounts/:id/edit
```

The page provides a Delete Account action targeting:

```text
/admin/accounts/:id/delete
```

The Delete Account target is a dedicated page dependency. Account Detail does not render the Delete page and does not execute its mutation operations.

Authentication tokens, refresh credentials, or other secrets must never appear in the route.

## Query State Preservation

When navigation to Account Detail originates from a filtered or paginated Account List, the originating Account List query state must remain available for return navigation.

Supported Account List query state:

```text
page
page_size
status
include_deleted
```

Example:

```text
/admin/accounts?page=2&page_size=20&status=active
```

The originating query state is navigation context only.

It must not be interpreted as Account authorization state and must not be used as evidence of Account permissions.

The Account Detail page must preserve the originating query state when navigating to:

```text
/admin/accounts
/admin/accounts/:id/edit
/admin/accounts/:id/delete
```

When returning to Account Management, the originating query state must be restored and authoritative Account List data must be re-fetched.

---

# 3. Requirements

| ID                          | Requirement                                                                          | Repository Reference                       |
| --------------------------- | ------------------------------------------------------------------------------------ | ------------------------------------------ |
| `FE-ACCOUNT-DETAIL-REQ-001` | Provide a protected Account Detail route.                                            | `ADM-AUTH-001`, `ADM-AUTH-004`             |
| `FE-ACCOUNT-DETAIL-REQ-002` | Load the Account using the Account API.                                              | `AC_UC_03`, `AC_API_03`                    |
| `FE-ACCOUNT-DETAIL-REQ-003` | Display the authoritative Account representation.                                    | Account API                                |
| `FE-ACCOUNT-DETAIL-REQ-004` | Render Account fields as read-only values.                                           | Account domain                             |
| `FE-ACCOUNT-DETAIL-REQ-005` | Display the Account lifecycle state returned by the authoritative response.          | Account domain                             |
| `FE-ACCOUNT-DETAIL-REQ-006` | Provide navigation back to `/admin/accounts`.                                        | Frontend navigation                        |
| `FE-ACCOUNT-DETAIL-REQ-007` | Provide an Edit action that navigates to `/admin/accounts/:id/edit`.                 | `ADM-AUTH-005`, `AC_UC_04`                 |
| `FE-ACCOUNT-DETAIL-REQ-008` | Provide a Delete Account action that navigates to `/admin/accounts/:id/delete`.      | `AC_UC_07`, Account Delete frontend design |
| `FE-ACCOUNT-DETAIL-REQ-009` | Preserve originating Account List query state during Account Detail navigation.      | Account Management frontend design         |
| `FE-ACCOUNT-DETAIL-REQ-010` | Preserve originating Account List query state when navigating from Detail to Edit.   | Account Management frontend design         |
| `FE-ACCOUNT-DETAIL-REQ-011` | Preserve originating Account List query state when navigating from Detail to Delete. | Account Management frontend design         |
| `FE-ACCOUNT-DETAIL-REQ-012` | Do not provide editable fields on the Detail page.                                   | Page boundary                              |
| `FE-ACCOUNT-DETAIL-REQ-013` | Do not provide Save or update controls on the Detail page.                           | Page boundary                              |
| `FE-ACCOUNT-DETAIL-REQ-014` | Do not provide Deactivate, Activate, or Restore controls on the Detail page.         | Page boundary                              |
| `FE-ACCOUNT-DETAIL-REQ-015` | Do not perform soft-delete or hard-delete mutations from the Detail page.            | Account Delete frontend design             |
| `FE-ACCOUNT-DETAIL-REQ-016` | Treat `401 Unauthorized` as an authentication failure.                               | Authentication domain                      |
| `FE-ACCOUNT-DETAIL-REQ-017` | Treat `403 Forbidden` as an authorization failure.                                   | Authorization domain                       |
| `FE-ACCOUNT-DETAIL-REQ-018` | Do not persist Account data in browser storage.                                      | Account API `no-store` contract            |
| `FE-ACCOUNT-DETAIL-REQ-019` | Do not expose password or password hash data.                                        | Account Security                           |
| `FE-ACCOUNT-DETAIL-REQ-020` | Provide visible focus states and keyboard interaction.                               | Admin Shell accessibility                  |
| `FE-ACCOUNT-DETAIL-REQ-021` | Preserve the Admin Shell across viewport sizes.                                      | Admin Shell responsive requirements        |

---

# 4. Page Structure

## 4.1 Page Header

The page header should use the following structure:

```text
Administrator Account
← Back to Administrator Accounts

Library Administrator
admin@example.com
Active

[Edit] [Delete Account]
```

| ID                         | Requirement                                                                                                         |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-UI-001` | Use `Administrator Account` as the primary page heading.                                                            |
| `FE-ACCOUNT-DETAIL-UI-002` | Display the account `display_name` when available.                                                                  |
| `FE-ACCOUNT-DETAIL-UI-003` | Display the Account email in the page header summary.                                                               |
| `FE-ACCOUNT-DETAIL-UI-004` | Display lifecycle status using text and a non-color-only status treatment.                                          |
| `FE-ACCOUNT-DETAIL-UI-005` | Provide a link back to `/admin/accounts`.                                                                           |
| `FE-ACCOUNT-DETAIL-UI-006` | Provide an Edit action in the page header action region.                                                            |
| `FE-ACCOUNT-DETAIL-UI-007` | Provide a Delete Account action in the page header action region when the loaded Account is available for deletion. |
| `FE-ACCOUNT-DETAIL-UI-008` | Keep page actions inside the Main Content Area.                                                                     |
| `FE-ACCOUNT-DETAIL-UI-009` | Do not provide editable Account fields on this page.                                                                |
| `FE-ACCOUNT-DETAIL-UI-010` | Do not execute deletion from the Delete Account navigation action.                                                  |

The Back action must use a real link:

```text
/admin/accounts
```

The Edit action must navigate to:

```text
/admin/accounts/:id/edit
```

The Delete Account action must navigate to:

```text
/admin/accounts/:id/delete
```

The Delete Account action is a navigation control, not a deletion control.

The page must not use a clickable `div` or a button that simulates navigation where a native link is appropriate.

---

## 4.2 Account Information

Account information is displayed as read-only content.

```text
Account Information

Display name
Library Administrator

Email
admin@example.com

Account ID
019...
```

| ID                            | Field        | Source         | Editable |
| ----------------------------- | ------------ | -------------- | -------- |
| `FE-ACCOUNT-DETAIL-FIELD-001` | Display name | `display_name` | No       |
| `FE-ACCOUNT-DETAIL-FIELD-002` | Email        | `email`        | No       |
| `FE-ACCOUNT-DETAIL-FIELD-003` | Account ID   | `id`           | No       |

The page must not render these values as editable form controls.

The page must not provide:

```text
Save
Save Changes
Submit
Reset
```

Account editing belongs to:

```text
/admin/accounts/:id/edit
```

---

## 4.3 Account Metadata

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
```

| ID                           | Metadata                                                                                                |
| ---------------------------- | ------------------------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-META-001` | Display `status`.                                                                                       |
| `FE-ACCOUNT-DETAIL-META-002` | Display `id`.                                                                                           |
| `FE-ACCOUNT-DETAIL-META-003` | Display `created_at`.                                                                                   |
| `FE-ACCOUNT-DETAIL-META-004` | Display `updated_at`.                                                                                   |
| `FE-ACCOUNT-DETAIL-META-005` | Never display `deleted_by`, `created_by`, `updated_by`, password hashes, or authentication credentials. |

Dates must be presented in the browser's localized date/time representation.

The underlying timestamp must remain available to assistive technologies and machine-readable consumers through the HTML `datetime` value where applicable.

---

## 4.4 Lifecycle Summary

The page must communicate lifecycle state explicitly.

| Condition         | Display    |
| ----------------- | ---------- |
| `status=active`   | `Active`   |
| `status=inactive` | `Inactive` |

The Account Detail API does not return soft-deleted Accounts through normal lookup.

A soft-deleted Account must therefore be handled as unavailable by the Account Detail page rather than rendered as a normal `Deleted` Detail state.

The page displays lifecycle state but does not provide lifecycle mutation controls.

The Delete Account navigation entry point is distinct from lifecycle mutation controls.

---

# 5. Account Data Contract

The Account Detail page consumes the authoritative Account response.

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

| ID                           | Field          | Type                    | Frontend Rule                                                  |
| ---------------------------- | -------------- | ----------------------- | -------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-DATA-001` | `id`           | UUID string             | Display as read-only identifier.                               |
| `FE-ACCOUNT-DETAIL-DATA-002` | `email`        | string                  | Display as read-only Account identifier.                       |
| `FE-ACCOUNT-DETAIL-DATA-003` | `display_name` | string or null          | Display as read-only text.                                     |
| `FE-ACCOUNT-DETAIL-DATA-004` | `status`       | string                  | Display as lifecycle state.                                    |
| `FE-ACCOUNT-DETAIL-DATA-005` | `created_at`   | RFC 3339 string         | Display as localized timestamp.                                |
| `FE-ACCOUNT-DETAIL-DATA-006` | `updated_at`   | RFC 3339 string         | Display as localized timestamp.                                |
| `FE-ACCOUNT-DETAIL-DATA-007` | `deleted_at`   | RFC 3339 string or null | Expected to be `null` for a successful normal Detail response. |

The frontend must never expect the Account response to contain:

```text
password
password_hash
```

Unknown fields must not be rendered automatically.

The server response is authoritative.

The frontend must not fabricate missing values.

Account data displayed by this page must not become editable through local form controls.

---

# 6. Edit Navigation

The Account Detail page does not perform Account updates.

The page provides an Edit action that navigates to the dedicated Account Edit page:

```text
/admin/accounts/:id/edit
```

Example:

```text
Administrator Account

Library Administrator
admin@example.com
Active

[Edit]
```

| ID                           | Requirement                                                                |
| ---------------------------- | -------------------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-EDIT-001` | Provide an Edit action on the Account Detail page.                         |
| `FE-ACCOUNT-DETAIL-EDIT-002` | Navigate to `/admin/accounts/:id/edit` when Edit is activated.             |
| `FE-ACCOUNT-DETAIL-EDIT-003` | Implement Edit as navigation rather than form submission.                  |
| `FE-ACCOUNT-DETAIL-EDIT-004` | Preserve the originating Account List query state when navigating to Edit. |
| `FE-ACCOUNT-DETAIL-EDIT-005` | Do not render editable Account fields on the Detail page.                  |
| `FE-ACCOUNT-DETAIL-EDIT-006` | Do not expose `PATCH /admin/accounts/{id}` from the Detail page.           |
| `FE-ACCOUNT-DETAIL-EDIT-007` | Do not expose Save or Save Changes controls on the Detail page.            |

The Edit action must remain clearly separate from the read-only Account information.

---

# 7. Delete Navigation

The Account Detail page provides a **Delete Account** navigation entry point for a loaded Account that is not soft-deleted.

The Delete Account action navigates to:

```text
/admin/accounts/:id/delete
```

The Account Delete page owns:

```text
Delete Account
    → DELETE /admin/accounts/{id}
```

and, for soft-deleted Accounts:

```text
Permanently Delete Account
    → DELETE /admin/accounts/{id}/purge
```

The Account Detail page does not execute either mutation.

| ID                             | Requirement                                                                                         |
| ------------------------------ | --------------------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-DELETE-001` | Provide a `Delete Account` navigation entry point for a loaded Account that is not soft-deleted.    |
| `FE-ACCOUNT-DETAIL-DELETE-002` | Navigate to `/admin/accounts/:id/delete`.                                                           |
| `FE-ACCOUNT-DETAIL-DELETE-003` | Implement Delete Account as native navigation, not as a mutation control.                           |
| `FE-ACCOUNT-DETAIL-DELETE-004` | Preserve the originating Account List query state when navigating to Account Delete.                |
| `FE-ACCOUNT-DETAIL-DELETE-005` | Do not call `DELETE /admin/accounts/{id}` from Account Detail.                                      |
| `FE-ACCOUNT-DETAIL-DELETE-006` | Do not call `DELETE /admin/accounts/{id}/purge` from Account Detail.                                |
| `FE-ACCOUNT-DETAIL-DELETE-007` | Do not render deletion confirmation on Account Detail.                                              |
| `FE-ACCOUNT-DETAIL-DELETE-008` | Do not render Permanently Delete controls on Account Detail.                                        |
| `FE-ACCOUNT-DETAIL-DELETE-009` | Treat Delete Account navigation as insufficient evidence of deletion authorization.                 |
| `FE-ACCOUNT-DETAIL-DELETE-010` | Keep deletion authorization and mutation handling on the dedicated Account Delete page and backend. |

Because soft-deleted Accounts are not returned by normal Account Detail lookup, Account Detail does not expose a Permanently Delete navigation action.

---

# 8. Account Detail Action Boundary

The Account Detail page contains navigation actions only.

## 8.1 Back Navigation

```text
← Back to Administrator Accounts
```

| ID                             | Requirement                                                      |
| ------------------------------ | ---------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-ACTION-001` | Navigate to `/admin/accounts`.                                   |
| `FE-ACCOUNT-DETAIL-ACTION-002` | Restore the originating Account List query state when available. |
| `FE-ACCOUNT-DETAIL-ACTION-003` | Use a native link.                                               |
| `FE-ACCOUNT-DETAIL-ACTION-004` | Preserve normal browser navigation semantics.                    |

---

## 8.2 Edit Navigation

```text
[Edit]
```

Target:

```text
/admin/accounts/:id/edit
```

| ID                             | Requirement                                                    |
| ------------------------------ | -------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-ACTION-005` | Provide the Edit action when Account details are available.    |
| `FE-ACCOUNT-DETAIL-ACTION-006` | Navigate to the Account Edit route.                            |
| `FE-ACCOUNT-DETAIL-ACTION-007` | Preserve the originating Account List query state.             |
| `FE-ACCOUNT-DETAIL-ACTION-008` | Use a native link or equivalent accessible navigation control. |

---

## 8.3 Delete Navigation

```text
[Delete Account]
```

Target:

```text
/admin/accounts/:id/delete
```

| ID                             | Requirement                                                                      |
| ------------------------------ | -------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-ACTION-009` | Provide Delete Account navigation for a loaded Account that is not soft-deleted. |
| `FE-ACCOUNT-DETAIL-ACTION-010` | Navigate to the dedicated Account Delete route.                                  |
| `FE-ACCOUNT-DETAIL-ACTION-011` | Preserve the originating Account List query state.                               |
| `FE-ACCOUNT-DETAIL-ACTION-012` | Use a native link.                                                               |
| `FE-ACCOUNT-DETAIL-ACTION-013` | Do not perform a delete request from the Detail page.                            |

---

## 8.4 Actions Not Provided by This Page

The Account Detail page must not provide:

```text
Save Changes
Deactivate
Activate
Restore
Soft Delete Mutation
Hard Delete Mutation
Delete Confirmation
Permanently Delete
```

| ID                             | Requirement                                                       |
| ------------------------------ | ----------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-ACTION-014` | Do not expose Account update mutations from the Detail page.      |
| `FE-ACCOUNT-DETAIL-ACTION-015` | Do not expose Deactivate from the Detail page.                    |
| `FE-ACCOUNT-DETAIL-ACTION-016` | Do not expose Activate from the Detail page.                      |
| `FE-ACCOUNT-DETAIL-ACTION-017` | Do not expose Restore from the Detail page.                       |
| `FE-ACCOUNT-DETAIL-ACTION-018` | Do not expose Soft Delete mutation controls from the Detail page. |
| `FE-ACCOUNT-DETAIL-ACTION-019` | Do not expose Hard Delete mutation controls from the Detail page. |
| `FE-ACCOUNT-DETAIL-ACTION-020` | Do not expose Delete confirmation from the Detail page.           |
| `FE-ACCOUNT-DETAIL-ACTION-021` | Do not expose Permanently Delete from the Detail page.            |

Lifecycle mutations remain Account Management or Account Delete responsibilities according to the Account Management and Account Delete frontend designs.

Account editing remains an Account Edit page responsibility.

---

# 9. API Contract

## 9.1 View Account

```http
GET /admin/accounts/{id}
Authorization: Bearer <access-token>
```

| ID                          | Contract                                                    |
| --------------------------- | ----------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-API-001` | Use `GET /admin/accounts/{id}`.                             |
| `FE-ACCOUNT-DETAIL-API-002` | Send bearer credentials through the `Authorization` header. |
| `FE-ACCOUNT-DETAIL-API-003` | Do not send tokens in query parameters.                     |
| `FE-ACCOUNT-DETAIL-API-004` | Do not send tokens in request bodies.                       |
| `FE-ACCOUNT-DETAIL-API-005` | Treat the returned Account as authoritative.                |

The Account Detail page is a read-only consumer of the Account representation.

The page does not perform Account update or lifecycle API calls.

The Delete Account action changes page location only:

```text
Account Detail
    ↓
Account Delete
```

---

## 9.2 Authorization

Viewing an Account requires:

```text
account:view
```

| Operation    | Permission     |
| ------------ | -------------- |
| View Account | `account:view` |

The frontend must not treat client-held roles or permissions as proof of authorization.

The backend is the authorization authority.

Delete Account navigation does not establish deletion authorization.

---

## 9.3 Error Handling

| ID                          | Error                    | UI Behavior                                                           |
| --------------------------- | ------------------------ | --------------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-API-006` | `400 INVALID_ACCOUNT_ID` | Render a non-editable invalid-resource error.                         |
| `FE-ACCOUNT-DETAIL-API-007` | `401`                    | Clear authentication state and redirect through Admin Shell behavior. |
| `FE-ACCOUNT-DETAIL-API-008` | `403`                    | Preserve authentication and show authorization error.                 |
| `FE-ACCOUNT-DETAIL-API-009` | `404 ACCOUNT_NOT_FOUND`  | Show Not Found state.                                                 |
| `FE-ACCOUNT-DETAIL-API-010` | `500`                    | Show non-sensitive server error and retry action.                     |
| `FE-ACCOUNT-DETAIL-API-011` | Network failure          | Show network error and retry action.                                  |

The Account Detail page must not convert a read operation into a mutation when handling an error.

Delete navigation errors belong to the Account Delete page once navigation occurs.

---

## 9.4 Cache

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

## 9.5 Problem Details

Errors use:

```text
application/problem+json
```

The frontend should parse:

```json
{
  "type": "...",
  "title": "...",
  "status": 404,
  "detail": "...",
  "code": "ACCOUNT_NOT_FOUND"
}
```

| ID                          | Requirement                                                                               |
| --------------------------- | ----------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-API-012` | Prefer the structured `code` for deterministic application behavior.                      |
| `FE-ACCOUNT-DETAIL-API-013` | Present user-facing messages appropriate to the read operation.                           |
| `FE-ACCOUNT-DETAIL-API-014` | Do not render raw stack traces or internal server details.                                |
| `FE-ACCOUNT-DETAIL-API-015` | Preserve generic fallback messaging when the response is malformed or lacks a known code. |

---

# 10. UI States

## 10.1 Bootstrap Pending

The page must not render as unauthenticated merely because authentication state is still `unknown`.

| ID                            | Behavior                                                          |
| ----------------------------- | ----------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-STATE-001` | Keep the protected route pending during authentication bootstrap. |
| `FE-ACCOUNT-DETAIL-STATE-002` | Do not flash the login page before authentication resolution.     |
| `FE-ACCOUNT-DETAIL-STATE-003` | Preserve Admin Shell authentication behavior.                     |

---

## 10.2 Initial Loading

While `GET /admin/accounts/{id}` is pending:

```text
Administrator Account

[loading account summary]

Account Information
[loading account details]
```

| ID                            | Requirement                                                                               |
| ----------------------------- | ----------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-STATE-004` | Keep the Admin Shell rendered.                                                            |
| `FE-ACCOUNT-DETAIL-STATE-005` | Show an explicit loading state in Main Content.                                           |
| `FE-ACCOUNT-DETAIL-STATE-006` | Do not display invented Account values.                                                   |
| `FE-ACCOUNT-DETAIL-STATE-007` | Do not display or enable Edit or Delete Account navigation before Account data is loaded. |

---

## 10.3 Loaded

The loaded page contains:

```text
Back to Administrator Accounts

Administrator Account
Display Name
Email
Status

Account Details

[Edit] [Delete Account]
```

The page displays read-only Account information.

The page does not expose editable fields or lifecycle mutation actions.

| ID                            | Requirement                                                                          |
| ----------------------------- | ------------------------------------------------------------------------------------ |
| `FE-ACCOUNT-DETAIL-STATE-008` | Display authoritative Account information.                                           |
| `FE-ACCOUNT-DETAIL-STATE-009` | Render Account fields as read-only.                                                  |
| `FE-ACCOUNT-DETAIL-STATE-010` | Display the Edit action after Account data is available.                             |
| `FE-ACCOUNT-DETAIL-STATE-011` | Display Delete Account navigation when the loaded Account is available for deletion. |
| `FE-ACCOUNT-DETAIL-STATE-012` | Do not display Save or lifecycle mutation controls.                                  |

---

## 10.4 Not Found

When the Account is unavailable:

```text
Administrator Account

Account not found.
The administrator account may no longer exist or may not be available.

[Back to Administrator Accounts]
```

| ID                            | Requirement                                                                        |
| ----------------------------- | ---------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-STATE-013` | Explain that the requested Account is unavailable.                                 |
| `FE-ACCOUNT-DETAIL-STATE-014` | Provide a direct route back to `/admin/accounts`.                                  |
| `FE-ACCOUNT-DETAIL-STATE-015` | Do not display stale Account details as current data.                              |
| `FE-ACCOUNT-DETAIL-STATE-016` | Do not display Edit or Delete Account navigation when Account data is unavailable. |

---

## 10.5 Authorization Error

For `403 Forbidden`:

```text
Administrator Account

You are not authorized to view this administrator account.
```

| ID                            | Requirement                                                                                      |
| ----------------------------- | ------------------------------------------------------------------------------------------------ |
| `FE-ACCOUNT-DETAIL-STATE-017` | Preserve authenticated state.                                                                    |
| `FE-ACCOUNT-DETAIL-STATE-018` | Keep the error inside Main Content.                                                              |
| `FE-ACCOUNT-DETAIL-STATE-019` | Do not redirect to `/login` for `403`.                                                           |
| `FE-ACCOUNT-DETAIL-STATE-020` | Do not display Account details that were not authorized by the backend.                          |
| `FE-ACCOUNT-DETAIL-STATE-021` | Do not display Delete Account navigation when the Account itself was not authorized for viewing. |

---

## 10.6 Authentication Expiry

For `401 Unauthorized`:

```text
Authentication has expired. Redirecting to login.
```

The page must rely on the existing Authentication service to clear authentication state and navigate to `/login`.

The Account Detail page must not implement an independent authentication system.

---

## 10.7 General Error

```text
Unable to load this administrator account.

[Try Again]
```

| ID                            | Requirement                                                                         |
| ----------------------------- | ----------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-STATE-022` | Provide a retry action for retryable failures.                                      |
| `FE-ACCOUNT-DETAIL-STATE-023` | Keep server and network errors inside Main Content.                                 |
| `FE-ACCOUNT-DETAIL-STATE-024` | Do not expose internal diagnostics.                                                 |
| `FE-ACCOUNT-DETAIL-STATE-025` | Preserve the Admin Shell.                                                           |
| `FE-ACCOUNT-DETAIL-STATE-026` | Do not display Edit or Delete Account navigation while Account data is unavailable. |

---

# 11. Responsive Layout

The page must inherit the Admin Shell responsive behavior and manage its own content layout.

| ID                           | Viewport       | Layout                                                                      |
| ---------------------------- | -------------- | --------------------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-RESP-001` | `>= 1280px`    | Two-column detail layout may be used for metadata and Account information.  |
| `FE-ACCOUNT-DETAIL-RESP-002` | `768px–1279px` | Single-column detail layout is acceptable when required by available width. |
| `FE-ACCOUNT-DETAIL-RESP-003` | `< 768px`      | Single-column stacked detail layout.                                        |
| `FE-ACCOUNT-DETAIL-RESP-004` | All viewports  | Delete Account and Edit navigation remain accessible and usable.            |
| `FE-ACCOUNT-DETAIL-RESP-005` | All viewports  | Prevent unintended page-level horizontal scrolling.                         |
| `FE-ACCOUNT-DETAIL-RESP-006` | All viewports  | Preserve keyboard accessibility.                                            |
| `FE-ACCOUNT-DETAIL-RESP-007` | All viewports  | Preserve authentication and authorization behavior.                         |

Suggested responsive structure:

```text
Desktop

┌───────────────────────────────────────────────────────────┐
│ Back to Administrator Accounts                            │
│                                                           │
│ Administrator Account                [Edit] [Delete]      │
│                                                           │
│ ┌──────────────────────────┐ ┌──────────────────────────┐ │
│ │ Account Information      │ │ Account Details          │ │
│ │                          │ │                          │ │
│ │ Display name             │ │ Status                   │ │
│ │ Email                    │ │ Account ID               │ │
│ │                          │ │ Created                  │ │
│ │                          │ │ Updated                  │ │
│ └──────────────────────────┘ └──────────────────────────┘ │
└───────────────────────────────────────────────────────────┘
```

```text
Mobile

Administrator Account

[Edit]
[Delete Account]

Account Information
Display name
Email

Account Details
Status
Account ID
Created
Updated
```

The exact visual styling must use the existing native CSS architecture and design conventions established by the frontend application.

---

# 12. Security

The Account Detail page must follow the Authentication, Authorization, and Account security contracts.

| ID                          | Security Rule            | Requirement                                                                                            |
| --------------------------- | ------------------------ | ------------------------------------------------------------------------------------------------------ |
| `FE-ACCOUNT-DETAIL-SEC-001` | Authentication           | Protected route requires authenticated Admin Shell state.                                              |
| `FE-ACCOUNT-DETAIL-SEC-002` | Authentication authority | Backend remains authoritative for authentication.                                                      |
| `FE-ACCOUNT-DETAIL-SEC-003` | Authorization            | Backend remains authoritative for authorization.                                                       |
| `FE-ACCOUNT-DETAIL-SEC-004` | Access token             | Bearer credentials are sent only in the `Authorization` header.                                        |
| `FE-ACCOUNT-DETAIL-SEC-005` | URL secrecy              | Tokens never appear in URLs.                                                                           |
| `FE-ACCOUNT-DETAIL-SEC-006` | UI secrecy               | Tokens are never rendered.                                                                             |
| `FE-ACCOUNT-DETAIL-SEC-007` | Logging                  | Tokens are never logged.                                                                               |
| `FE-ACCOUNT-DETAIL-SEC-008` | Credential secrecy       | Passwords and password hashes are never rendered.                                                      |
| `FE-ACCOUNT-DETAIL-SEC-009` | Client authorization     | Client roles and permissions are not trusted as authorization proof.                                   |
| `FE-ACCOUNT-DETAIL-SEC-010` | Account persistence      | Account response data is not stored in browser persistence.                                            |
| `FE-ACCOUNT-DETAIL-SEC-011` | Cache                    | Account responses are treated as `no-store`.                                                           |
| `FE-ACCOUNT-DETAIL-SEC-012` | Mutation boundary        | The Detail page does not perform Account mutations.                                                    |
| `FE-ACCOUNT-DETAIL-SEC-013` | Delete navigation        | Delete Account visibility is navigation convenience only and is not a deletion authorization boundary. |
| `FE-ACCOUNT-DETAIL-SEC-014` | Error exposure           | Internal server diagnostics are not exposed to administrators.                                         |

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

# 13. Accessibility

## 13.1 Semantic Structure

| ID                           | Requirement                                                            |
| ---------------------------- | ---------------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-A11Y-001` | Use semantic layout elements such as `main`, `header`, and `section`.  |
| `FE-ACCOUNT-DETAIL-A11Y-002` | Provide exactly one primary `h1` for the page.                         |
| `FE-ACCOUNT-DETAIL-A11Y-003` | Use logical heading hierarchy for Account sections.                    |
| `FE-ACCOUNT-DETAIL-A11Y-004` | Use semantic read-only content elements rather than editable controls. |
| `FE-ACCOUNT-DETAIL-A11Y-005` | Provide accessible names for displayed Account data.                   |

---

## 13.2 Navigation

| ID                           | Requirement                                                  |
| ---------------------------- | ------------------------------------------------------------ |
| `FE-ACCOUNT-DETAIL-A11Y-006` | Back to Accounts uses a native link.                         |
| `FE-ACCOUNT-DETAIL-A11Y-007` | Edit navigation uses a native link.                          |
| `FE-ACCOUNT-DETAIL-A11Y-008` | Delete Account navigation uses a native link.                |
| `FE-ACCOUNT-DETAIL-A11Y-009` | Keyboard order follows the visual and logical reading order. |
| `FE-ACCOUNT-DETAIL-A11Y-010` | Focus is visibly indicated.                                  |
| `FE-ACCOUNT-DETAIL-A11Y-011` | Focus is not obscured by responsive UI or sticky controls.   |
| `FE-ACCOUNT-DETAIL-A11Y-012` | Navigation link names identify their purpose in context.     |

Suggested accessible names:

```text
Back to administrator accounts
Edit account: Library Administrator
Delete account: Library Administrator
```

---

## 13.3 Async Status

Loading and error states must be communicated to assistive technology.

| ID                           | Requirement                                       |
| ---------------------------- | ------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-A11Y-013` | Loading state is announced appropriately.         |
| `FE-ACCOUNT-DETAIL-A11Y-014` | Read failures are announced appropriately.        |
| `FE-ACCOUNT-DETAIL-A11Y-015` | Status messages do not steal focus unnecessarily. |

A suitable status region may use:

```html
<div role="status" aria-live="polite"></div>
```

Error messaging may use an appropriate assertive live region when necessary.

---

## 13.4 Status Presentation

Status must not depend on color alone.

Example:

```text
Active
Inactive
```

The text label remains the authoritative visual meaning.

---

## 13.5 Target Size

Interactive targets must meet the applicable WCAG 2.2 Target Size (Minimum) requirement or a documented exception must apply.

Delete Account, Edit, and Back links must remain independently targetable and must not require precise pointer interaction.

---

# 14. Component Structure

The page should remain feature-local and keep Account-specific behavior out of the Admin Shell.

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
AccountDetailPage
├── AccountDetailHeader
├── AccountInformationSection
├── AccountDetailsSection
├── AccountStatus
├── AccountActionGroup
├── AccountLoadingState
├── AccountErrorState
└── AccountNotFoundState
```

| ID                           | Component                   | Responsibility                                                                                       |
| ---------------------------- | --------------------------- | ---------------------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-COMP-001` | `AccountDetailPage`         | Coordinate route state, loading, Account data, navigation, and page layout.                          |
| `FE-ACCOUNT-DETAIL-COMP-002` | `AccountDetailHeader`       | Render page title, summary, status, Back navigation, Edit navigation, and Delete Account navigation. |
| `FE-ACCOUNT-DETAIL-COMP-003` | `AccountInformationSection` | Render read-only Account information.                                                                |
| `FE-ACCOUNT-DETAIL-COMP-004` | `AccountDetailsSection`     | Render read-only metadata.                                                                           |
| `FE-ACCOUNT-DETAIL-COMP-005` | `AccountStatus`             | Render accessible lifecycle status.                                                                  |
| `FE-ACCOUNT-DETAIL-COMP-006` | `AccountActionGroup`        | Render Back, Edit, and Delete Account navigation controls only.                                      |
| `FE-ACCOUNT-DETAIL-COMP-007` | `AccountLoadingState`       | Render initial loading state.                                                                        |
| `FE-ACCOUNT-DETAIL-COMP-008` | `AccountErrorState`         | Render non-recoverable and retryable errors.                                                         |
| `FE-ACCOUNT-DETAIL-COMP-009` | `AccountNotFoundState`      | Render unavailable Account state.                                                                    |

The page must not introduce:

```text
AccountForm
AccountSaveButton
AccountLifecycleSection
AccountDeleteDialog
```

because Account editing, lifecycle mutation, and deletion confirmation do not belong on the Detail page.

The Delete Account action is a navigation control only.

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

# 15. Testing

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

## 15.1 Unit

| ID                                | Test                                                               |
| --------------------------------- | ------------------------------------------------------------------ |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-001` | Account response maps to the detail view correctly.                |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-002` | `display_name=null` renders as a read-only value.                  |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-003` | Email renders as read-only content.                                |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-004` | Account ID renders as read-only content.                           |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-005` | Active Account renders `Active`.                                   |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-006` | Inactive Account renders `Inactive`.                               |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-007` | Unavailable or soft-deleted Account renders Not Found.             |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-008` | Edit action targets `/admin/accounts/:id/edit`.                    |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-009` | Delete Account action targets `/admin/accounts/:id/delete`.        |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-010` | Back action targets `/admin/accounts`.                             |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-011` | Child navigation preserves originating Account List query state.   |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-012` | No editable form controls are rendered.                            |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-013` | No lifecycle mutation controls are rendered.                       |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-014` | Delete Account does not invoke a delete or purge API.              |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-015` | `401` is delegated to authentication handling.                     |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-016` | `403` preserves authentication and renders an authorization error. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-017` | `404` renders Not Found.                                           |

---

## 15.2 Integration

| ID                               | Test                                                                                         |
| -------------------------------- | -------------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-TEST-INT-001` | Authenticated navigation from Account List to Account Detail works.                          |
| `FE-ACCOUNT-DETAIL-TEST-INT-002` | Account detail loads from `GET /admin/accounts/{id}`.                                        |
| `FE-ACCOUNT-DETAIL-TEST-INT-003` | Account fields render as read-only values.                                                   |
| `FE-ACCOUNT-DETAIL-TEST-INT-004` | Edit action navigates to `/admin/accounts/{id}/edit` and preserves query state.              |
| `FE-ACCOUNT-DETAIL-TEST-INT-005` | Back action returns to `/admin/accounts` with originating query state.                       |
| `FE-ACCOUNT-DETAIL-TEST-INT-006` | Delete Account action navigates to `/admin/accounts/{id}/delete` with preserved query state. |
| `FE-ACCOUNT-DETAIL-TEST-INT-007` | `401` redirects to `/login`.                                                                 |
| `FE-ACCOUNT-DETAIL-TEST-INT-008` | `403` preserves authentication.                                                              |
| `FE-ACCOUNT-DETAIL-TEST-INT-009` | `404` renders Account Not Found.                                                             |
| `FE-ACCOUNT-DETAIL-TEST-INT-010` | Account data is not persisted in browser storage.                                            |
| `FE-ACCOUNT-DETAIL-TEST-INT-011` | Tokens do not appear in URLs or rendered output.                                             |
| `FE-ACCOUNT-DETAIL-TEST-INT-012` | Account Detail performs no update request.                                                   |
| `FE-ACCOUNT-DETAIL-TEST-INT-013` | Account Detail performs no lifecycle mutation request.                                       |
| `FE-ACCOUNT-DETAIL-TEST-INT-014` | Account Detail performs no delete or purge request.                                          |

---

## 15.3 E2E

| ID                               | Test                                                                                         |
| -------------------------------- | -------------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-TEST-E2E-001` | Login → Admin Shell → Accounts works.                                                        |
| `FE-ACCOUNT-DETAIL-TEST-E2E-002` | Account row opens the correct Account Detail route.                                          |
| `FE-ACCOUNT-DETAIL-TEST-E2E-003` | Account details render correctly.                                                            |
| `FE-ACCOUNT-DETAIL-TEST-E2E-004` | Account detail fields remain read-only.                                                      |
| `FE-ACCOUNT-DETAIL-TEST-E2E-005` | Edit action navigates to the Account Edit route with preserved list state.                   |
| `FE-ACCOUNT-DETAIL-TEST-E2E-006` | Back navigation returns to Account Management with preserved list state.                     |
| `FE-ACCOUNT-DETAIL-TEST-E2E-007` | Delete Account navigation opens the dedicated Account Delete page with preserved list state. |
| `FE-ACCOUNT-DETAIL-TEST-E2E-008` | Authentication expiry redirects to `/login`.                                                 |
| `FE-ACCOUNT-DETAIL-TEST-E2E-009` | Unauthorized detail access displays `403` without logout.                                    |
| `FE-ACCOUNT-DETAIL-TEST-E2E-010` | Account Detail does not execute a deletion mutation.                                         |
| `FE-ACCOUNT-DETAIL-TEST-E2E-011` | Mobile layout remains usable below `768px`.                                                  |

---

## 15.4 Accessibility

| ID                                | Test                                                                                 |
| --------------------------------- | ------------------------------------------------------------------------------------ |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-001` | Heading hierarchy is correct.                                                        |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-002` | Account information has accessible names.                                            |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-003` | Back navigation is keyboard accessible.                                              |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-004` | Edit navigation is keyboard accessible.                                              |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-005` | Delete Account navigation is keyboard accessible.                                    |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-006` | Focus is visible.                                                                    |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-007` | Focus is not obscured.                                                               |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-008` | Async status is announced.                                                           |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-009` | Account status does not rely on color alone.                                         |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-010` | Delete Account has a descriptive accessible name.                                    |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-011` | Interactive targets meet the required minimum target size or a documented exception. |

---

## 15.5 Responsive

| ID                                | Test                                                                     |
| --------------------------------- | ------------------------------------------------------------------------ |
| `FE-ACCOUNT-DETAIL-TEST-RESP-001` | Desktop layout works at `>= 1280px`.                                     |
| `FE-ACCOUNT-DETAIL-TEST-RESP-002` | Tablet layout works at `768px–1279px`.                                   |
| `FE-ACCOUNT-DETAIL-TEST-RESP-003` | Mobile layout works below `768px`.                                       |
| `FE-ACCOUNT-DETAIL-TEST-RESP-004` | No unintended page-level horizontal scrolling occurs.                    |
| `FE-ACCOUNT-DETAIL-TEST-RESP-005` | Back, Edit, and Delete Account actions remain accessible on mobile.      |
| `FE-ACCOUNT-DETAIL-TEST-RESP-006` | Admin Shell behavior is preserved at every viewport size.                |
| `FE-ACCOUNT-DETAIL-TEST-RESP-007` | Authentication and authorization behavior is unchanged across viewports. |

---

# 16. Implementation Criteria

## 16.1 Route

| ID                           | Criteria                                                                        | Status         |
| ---------------------------- | ------------------------------------------------------------------------------- | -------------- |
| `FE-ACCOUNT-DETAIL-IMPL-001` | `/admin/accounts/:id` renders inside the Admin Shell.                           | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-002` | Unauthenticated access is handled by Admin Shell authentication behavior.       | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-003` | Accounts navigation remains active while viewing the detail page.               | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-004` | Back navigation returns to `/admin/accounts`.                                   | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-005` | Edit action navigates to `/admin/accounts/:id/edit`.                            | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-006` | Delete Account action navigates to `/admin/accounts/:id/delete`.                | 🟡 Defined     |
| `FE-ACCOUNT-DETAIL-IMPL-007` | Originating Account List query state is preserved across child-page navigation. | 🟡 Defined     |

---

## 16.2 Data

| ID                           | Criteria                                             | Status         |
| ---------------------------- | ---------------------------------------------------- | -------------- |
| `FE-ACCOUNT-DETAIL-IMPL-008` | Detail data comes from `GET /admin/accounts/{id}`.   | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-009` | Server Account response is authoritative.            | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-010` | Password credentials are never expected or rendered. | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-011` | Account data is not stored in browser persistence.   | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-012` | Account response cache policy remains `no-store`.    | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-013` | Account fields render as read-only values.           | 🟢 Implemented |

---

## 16.3 Editing Boundary

| ID                           | Criteria                                                        | Status         |
| ---------------------------- | --------------------------------------------------------------- | -------------- |
| `FE-ACCOUNT-DETAIL-IMPL-014` | No editable Account fields are rendered on the Detail page.     | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-015` | No Save or Save Changes control is rendered on the Detail page. | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-016` | The Detail page does not submit `PATCH /admin/accounts/{id}`.   | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-017` | Edit navigation is provided from the Detail page.               | 🟢 Implemented |

---

## 16.4 Delete Navigation Boundary

| ID                           | Criteria                                                                  | Status         |
| ---------------------------- | ------------------------------------------------------------------------- | -------------- |
| `FE-ACCOUNT-DETAIL-IMPL-018` | Delete Account navigation is provided from the Detail page.               | 🟡 Defined     |
| `FE-ACCOUNT-DETAIL-IMPL-019` | Delete Account navigation targets `/admin/accounts/:id/delete`.           | 🟡 Defined     |
| `FE-ACCOUNT-DETAIL-IMPL-020` | Delete Account navigation preserves originating Account List query state. | 🟡 Defined     |
| `FE-ACCOUNT-DETAIL-IMPL-021` | Detail page does not execute `DELETE /admin/accounts/{id}`.               | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-022` | Detail page does not execute `DELETE /admin/accounts/{id}/purge`.         | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-023` | Delete confirmation is owned by the dedicated Account Delete page.        | 🟢 Implemented |

---

## 16.5 Lifecycle Boundary

| ID                           | Criteria                                                   | Status         |
| ---------------------------- | ---------------------------------------------------------- | -------------- |
| `FE-ACCOUNT-DETAIL-IMPL-024` | Deactivate is not exposed on the Detail page.              | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-025` | Activate is not exposed on the Detail page.                | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-026` | Restore is not exposed on the Detail page.                 | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-027` | Soft-delete mutation is not exposed on the Detail page.    | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-028` | Hard-delete mutation is not exposed on the Detail page.    | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-029` | Lifecycle mutation APIs are not called by the Detail page. | 🟢 Implemented |

---

## 16.6 Security

| ID                           | Criteria                                                                            | Status         |
| ---------------------------- | ----------------------------------------------------------------------------------- | -------------- |
| `FE-ACCOUNT-DETAIL-IMPL-030` | Backend authentication remains authoritative.                                       | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-031` | Backend authorization remains authoritative.                                        | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-032` | Tokens are only sent through the `Authorization` header for protected Account APIs. | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-033` | Tokens never appear in URLs.                                                        | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-034` | Tokens never render in the UI.                                                      | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-035` | Tokens never appear in logs.                                                        | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-036` | Passwords and password hashes never render.                                         | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-037` | Delete Account visibility is not an authorization boundary.                         | 🟡 Defined     |

---

## 16.7 Accessibility

| ID                           | Criteria                                                                                               | Status         |
| ---------------------------- | ------------------------------------------------------------------------------------------------------ | -------------- |
| `FE-ACCOUNT-DETAIL-IMPL-038` | Semantic page structure is used.                                                                       | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-039` | Primary page heading is present.                                                                       | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-040` | Account information has accessible labels or names.                                                    | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-041` | Navigation and actions are keyboard accessible.                                                        | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-042` | Focus is visible and not obscured.                                                                     | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-043` | Async status is accessible.                                                                            | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-044` | Account lifecycle status does not rely only on color.                                                  | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-045` | Delete Account navigation uses a native accessible link.                                               | 🟡 Defined     |
| `FE-ACCOUNT-DETAIL-IMPL-046` | Delete Account target size satisfies the applicable accessibility requirement or documented exception. | 🟡 Defined     |

---

# 17. Traceability

## Requirements → Design → Backend → Test

| Requirement               | Frontend Design                                           | Backend Contract                   | Test                                    |
| ------------------------- | --------------------------------------------------------- | ---------------------------------- | --------------------------------------- |
| `ADM-AUTH-001`            | `FE-ACCOUNT-DETAIL-ROUTE-001`                             | Authentication domain              | `FE-ACCOUNT-DETAIL-TEST-E2E-001`        |
| `ADM-AUTH-004`            | `FE-ACCOUNT-DETAIL-REQ-002`                               | `AC_UC_03`, `AC_API_03`            | `FE-ACCOUNT-DETAIL-TEST-INT-002`        |
| `ADM-AUTH-005`            | `FE-ACCOUNT-DETAIL-EDIT-001`                              | `AC_UC_04`, `AC_API_04`            | `FE-ACCOUNT-DETAIL-TEST-E2E-005`        |
| `AC_UC_03`                | `FE-ACCOUNT-DETAIL-API-001`                               | `GET /admin/accounts/{id}`         | `FE-ACCOUNT-DETAIL-TEST-INT-002`        |
| `AC_UC_07`                | `FE-ACCOUNT-DETAIL-DELETE-001`                            | `AC_API_07`                        | `FE-ACCOUNT-DETAIL-TEST-E2E-007`        |
| `account:view`            | `FE-ACCOUNT-DETAIL-API-001`                               | Authorization domain               | `FE-ACCOUNT-DETAIL-TEST-INT-008`        |
| Authentication `401`      | `FE-ACCOUNT-DETAIL-API-007`                               | Authentication domain              | `FE-ACCOUNT-DETAIL-TEST-E2E-008`        |
| Authorization `403`       | `FE-ACCOUNT-DETAIL-API-008`                               | Authorization domain               | `FE-ACCOUNT-DETAIL-TEST-INT-009`        |
| Account Detail navigation | `FE-ACCOUNT-DETAIL-ACTION-001`                            | Frontend navigation                | `FE-ACCOUNT-DETAIL-TEST-INT-005`        |
| Edit navigation           | `FE-ACCOUNT-DETAIL-EDIT-001`                              | `AC_UC_04`, `AC_API_04`            | `FE-ACCOUNT-DETAIL-TEST-INT-004`        |
| Delete navigation         | `FE-ACCOUNT-DETAIL-DELETE-001`                            | Account Delete frontend design     | `FE-ACCOUNT-DETAIL-TEST-INT-006`        |
| Delete mutation boundary  | `FE-ACCOUNT-DETAIL-DELETE-005`, `006`                     | `AC_API_07`, `AC_API_09`           | `FE-ACCOUNT-DETAIL-TEST-INT-014`        |
| Query state preservation  | `FE-ACCOUNT-DETAIL-ACTION-002`, `007`, `011`              | Account Management frontend design | `FE-ACCOUNT-DETAIL-TEST-E2E-005`, `007` |
| Admin Shell               | `FE-ACCOUNT-DETAIL-ROUTE-001`, `FE-ACCOUNT-DETAIL-RESP-*` | `admin_shell.md`                   | `FE-ACCOUNT-DETAIL-TEST-E2E-001`        |

## Domain References

| Domain             | Relevant References                                                                                        |
| ------------------ | ---------------------------------------------------------------------------------------------------------- |
| Requirements       | `ADM-AUTH-001` to `ADM-AUTH-005`, `CNT-ADMIN-*`, `SCP-009`                                                 |
| Sitemap            | `/admin/accounts`, `/admin/accounts/:id`, `/admin/accounts/:id/delete`; page IDs require sitemap alignment |
| Account            | `AC_UC_03`, `AC_API_03`, `AC_UC_04`, `AC_API_04`, `AC_UC_07`, `AC_API_07`, `AC_UC_09`, `AC_API_09`         |
| Authentication     | Protected route behavior, bearer authentication, `401` handling                                            |
| Authorization      | `account:view`, deny-by-default, `403`                                                                     |
| Admin Shell        | Shell layout, route protection, active Accounts navigation, responsive behavior                            |
| Account Management | `/admin/accounts`, View navigation, Edit navigation, lifecycle action separation, originating query state  |
| Account Delete     | `/admin/accounts/:id/delete`, destructive confirmation, delete and purge mutations                         |

## Cross-Document Dependencies

| ID                           | Dependency         | Requirement                                                  |
| ---------------------------- | ------------------ | ------------------------------------------------------------ |
| `FE-ACCOUNT-DETAIL-XDOC-001` | Account Management | Supplies originating Account List context and query state.   |
| `FE-ACCOUNT-DETAIL-XDOC-002` | Account Edit       | Owns Account editing and update mutations.                   |
| `FE-ACCOUNT-DETAIL-XDOC-003` | Account Delete     | Owns Delete Account confirmation and deletion mutations.     |
| `FE-ACCOUNT-DETAIL-XDOC-004` | Sitemap            | Must represent the Account Detail and Account Delete routes. |

Required repository alignment:

```text
docs/docs/sitemap.md
    → represent /admin/accounts/:id
    → represent /admin/accounts/:id/delete

docs/docs/design/frontend/admin_account_management_page.md
    → preserve Account List query state into Account Detail
    → preserve Account List query state into Account Delete
    → keep destructive mutations outside Account Management

docs/docs/design/frontend/admin_account_delete_page.md
    → define Delete Account confirmation
    → define DELETE /admin/accounts/{id}
    → define DELETE /admin/accounts/{id}/purge
```

The Account Detail page does not own the Account Edit or Account Delete page implementations.

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
    ├── Read-only account information
    ├── Back navigation
    ├── Edit navigation
    │       ↓
    │   Account Edit
    │
    └── Delete Account navigation
            ↓
        Account Delete
            ├── Delete Account
            └── Permanently Delete Account
    ↓
Account API
    ↓
Authentication
    ↓
Authorization
    ↓
Tests
```

---

# 18. Standards and Technology References

The Account Detail page follows the repository Admin Application stack:

```text
React
Vite
TypeScript
Native CSS
```

The design follows current official guidance for semantic HTML, React DOM usage, TypeScript/Vite workflows, URL query APIs, accessibility, and server-side authorization.

| ID                          | Reference                                | Application                                                                                                                        |
| --------------------------- | ---------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-STD-001` | React official DOM component guidance    | Use native browser elements through React and standard DOM props/events rather than recreating native browser behavior.            |
| `FE-ACCOUNT-DETAIL-STD-002` | Vite official TypeScript guidance        | Keep type checking in the repository's explicit TypeScript workflow; Vite transpilation alone is not the type-checking step.       |
| `FE-ACCOUNT-DETAIL-STD-003` | TypeScript 6.0 documentation             | Keep route state, API responses, Account models, and navigation builders explicitly typed.                                         |
| `FE-ACCOUNT-DETAIL-STD-004` | WCAG 2.2                                 | Preserve keyboard access, visible focus, focus visibility, target sizing, meaningful headings, link purpose, and status messaging. |
| `FE-ACCOUNT-DETAIL-STD-005` | WAI-ARIA Authoring Practices Guide       | Prefer native HTML semantics and native links whenever equivalent HTML semantics exist.                                            |
| `FE-ACCOUNT-DETAIL-STD-006` | URL and URLSearchParams browser APIs     | Preserve supported originating Account List query state through Account Detail child-page navigation.                              |
| `FE-ACCOUNT-DETAIL-STD-007` | OWASP server-side authorization guidance | Treat backend authorization as authoritative; client-side visibility is never the security boundary.                               |

## Normative Rules

The Account Detail page must:

1. Use semantic HTML for headings, sections, links, and displayed Account information.
2. Use native links for Back, Edit, and Delete Account navigation.
3. Keep the page read-only.
4. Keep lifecycle mutation controls outside the Detail page.
5. Keep destructive confirmation and deletion mutations on the dedicated Account Delete page.
6. Preserve originating Account List query state without treating it as authorization state.
7. Ensure interactive targets meet applicable WCAG 2.2 requirements.
8. Expose asynchronous loading and error states to assistive technologies without unnecessary focus movement.
9. Never rely on client-side roles, permissions, or navigation visibility as authorization proof.
10. Never expose credentials, password hashes, access tokens, or internal server diagnostics.

The Account Detail page provides a **Delete Account navigation entry point** but does not perform deletion.

The dedicated Account Delete page owns:

```text
Delete confirmation
    ↓
DELETE /admin/accounts/{id}

Permanently Delete confirmation
    ↓
DELETE /admin/accounts/{id}/purge
```

The Account Detail page remains read-only throughout the entire navigation flow.
