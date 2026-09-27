# Frontend Admin Account Detail Page

## Table of Contents

1. [Scope](#1-scope)
2. [Routes](#2-routes)
3. [Requirements](#3-requirements)
4. [Page Structure](#4-page-structure)
5. [Account Data Contract](#5-account-data-contract)
6. [Edit Navigation](#6-edit-navigation)
7. [Account Detail Action Boundary](#7-account-detail-action-boundary)
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
* Soft Delete controls.
* Hard Delete controls.

The page provides an **Edit** navigation action that leads to:

```text
/admin/accounts/:id/edit
```

## Responsibilities

| ID | Responsibility |
|---|---|
| `FE-ACCOUNT-DETAIL-SCOPE-001` | Load one administrator account by ID. |
| `FE-ACCOUNT-DETAIL-SCOPE-002` | Display authoritative administrator account information. |
| `FE-ACCOUNT-DETAIL-SCOPE-003` | Display the account lifecycle state. |
| `FE-ACCOUNT-DETAIL-SCOPE-004` | Provide navigation back to Account Management. |
| `FE-ACCOUNT-DETAIL-SCOPE-005` | Provide an Edit action that navigates to the Account Edit page. |
| `FE-ACCOUNT-DETAIL-SCOPE-006` | Handle authentication, authorization, not-found, server, and network errors. |
| `FE-ACCOUNT-DETAIL-SCOPE-007` | Provide keyboard-accessible and responsive interaction. |
| `FE-ACCOUNT-DETAIL-SCOPE-008` | Preserve Admin Shell navigation and authentication behavior. |

## Out of Scope

| ID | Excluded Area |
|---|---|
| `FE-ACCOUNT-DETAIL-OOS-001` | Authentication implementation. |
| `FE-ACCOUNT-DETAIL-OOS-002` | Authorization implementation. |
| `FE-ACCOUNT-DETAIL-OOS-003` | Role management. |
| `FE-ACCOUNT-DETAIL-OOS-004` | Password hashing or password validation implementation. |
| `FE-ACCOUNT-DETAIL-OOS-005` | Database persistence or transaction management. |
| `FE-ACCOUNT-DETAIL-OOS-006` | Client-side authorization enforcement. |
| `FE-ACCOUNT-DETAIL-OOS-007` | Account field editing. |
| `FE-ACCOUNT-DETAIL-OOS-008` | Account lifecycle mutations. |
| `FE-ACCOUNT-DETAIL-OOS-009` | Hard-delete UI. |
| `FE-ACCOUNT-DETAIL-OOS-010` | Search or unsupported Account filtering. |
| `FE-ACCOUNT-DETAIL-OOS-011` | Client-side Account data persistence. |

---

# 2. Routes

| ID | Route | Access | Behavior | References |
|---|---|---|---|---|
| `FE-ACCOUNT-DETAIL-ROUTE-001` | `/admin/accounts/:id` | Authenticated | Render read-only Account Detail page. | `PAGE-ADM-010`, `ADM-AUTH-004` |
| `FE-ACCOUNT-DETAIL-ROUTE-002` | `/admin/accounts/:id` | Bootstrap | Keep route pending until authentication resolves. | Authentication domain |
| `FE-ACCOUNT-DETAIL-ROUTE-003` | `/admin/accounts/:id` | Unauthenticated | Redirect to `/login`. | Admin Shell |
| `FE-ACCOUNT-DETAIL-ROUTE-004` | `/admin/accounts/:id` | Authenticated + unauthorized | Render authorization error while preserving authentication state. | Authorization domain |
| `FE-ACCOUNT-DETAIL-ROUTE-005` | `/admin/accounts/:id` | Invalid or unavailable Account | Render Account Not Found state. | `AC_API_03` |

The page must render inside the Admin Shell:

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

Authentication tokens, refresh credentials, or other secrets must never appear in the route.

---

# 3. Requirements

| ID | Requirement | Repository Reference |
|---|---|---|
| `FE-ACCOUNT-DETAIL-REQ-001` | Provide a protected Account Detail route. | `ADM-AUTH-001`, `ADM-AUTH-004`, `PAGE-ADM-010` |
| `FE-ACCOUNT-DETAIL-REQ-002` | Load the Account using the Account API. | `AC_UC_03`, `AC_API_03` |
| `FE-ACCOUNT-DETAIL-REQ-003` | Display the authoritative Account representation. | Account API |
| `FE-ACCOUNT-DETAIL-REQ-004` | Render Account fields as read-only values. | Account domain |
| `FE-ACCOUNT-DETAIL-REQ-005` | Display active, inactive, and deleted lifecycle state when represented by the authoritative response. | Account domain |
| `FE-ACCOUNT-DETAIL-REQ-006` | Provide navigation back to `/admin/accounts`. | Frontend navigation |
| `FE-ACCOUNT-DETAIL-REQ-007` | Provide an Edit action that navigates to `/admin/accounts/:id/edit`. | `ADM-AUTH-005`, `AC_UC_04` |
| `FE-ACCOUNT-DETAIL-REQ-008` | Do not provide editable fields on the Detail page. | Page boundary |
| `FE-ACCOUNT-DETAIL-REQ-009` | Do not provide Save or update controls on the Detail page. | Page boundary |
| `FE-ACCOUNT-DETAIL-REQ-010` | Do not provide Deactivate, Activate, or Restore controls on the Detail page. | Page boundary |
| `FE-ACCOUNT-DETAIL-REQ-011` | Treat `401 Unauthorized` as an authentication failure. | Authentication domain |
| `FE-ACCOUNT-DETAIL-REQ-012` | Treat `403 Forbidden` as an authorization failure. | Authorization domain |
| `FE-ACCOUNT-DETAIL-REQ-013` | Do not persist Account data in browser storage. | Account API `no-store` contract |
| `FE-ACCOUNT-DETAIL-REQ-014` | Do not expose password or password hash data. | Account Security |
| `FE-ACCOUNT-DETAIL-REQ-015` | Provide visible focus states and keyboard interaction. | Admin Shell accessibility |
| `FE-ACCOUNT-DETAIL-REQ-016` | Preserve the Admin Shell across viewport sizes. | Admin Shell responsive requirements |

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

[Edit]
```

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-UI-001` | Use `Administrator Account` as the primary page heading. |
| `FE-ACCOUNT-DETAIL-UI-002` | Display the account `display_name` when available. |
| `FE-ACCOUNT-DETAIL-UI-003` | Display the Account email in the page header summary. |
| `FE-ACCOUNT-DETAIL-UI-004` | Display lifecycle status using text and a non-color-only status treatment. |
| `FE-ACCOUNT-DETAIL-UI-005` | Provide a link back to `/admin/accounts`. |
| `FE-ACCOUNT-DETAIL-UI-006` | Provide an Edit action in the page header action region. |
| `FE-ACCOUNT-DETAIL-UI-007` | Keep page actions inside the Main Content Area. |
| `FE-ACCOUNT-DETAIL-UI-008` | Do not provide editable Account fields on this page. |

The Back action must use a real link:

```text
/admin/accounts
```

It must not use a clickable `div` or a button that simulates navigation.

The Edit action must navigate to:

```text
/admin/accounts/:id/edit
```

The Edit action is a navigation control, not a form submission control.

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

| ID | Field | Source | Editable |
|---|---|---|---|
| `FE-ACCOUNT-DETAIL-FIELD-001` | Display name | `display_name` | No |
| `FE-ACCOUNT-DETAIL-FIELD-002` | Email | `email` | No |
| `FE-ACCOUNT-DETAIL-FIELD-003` | Account ID | `id` | No |

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

## 4.4 Lifecycle Summary

The page must communicate lifecycle state explicitly.

| Condition | Display |
|---|---|
| `status=active`, `deleted_at=null` | `Active` |
| `status=inactive`, `deleted_at=null` | `Inactive` |
| `deleted_at != null` | `Deleted` |

`Deleted` is a derived UI state and is not an Account `status` value.

The page displays lifecycle state but does not provide lifecycle mutation controls.

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

| ID | Field | Type | Frontend Rule |
|---|---|---|---|
| `FE-ACCOUNT-DETAIL-DATA-001` | `id` | UUID string | Display as read-only identifier. |
| `FE-ACCOUNT-DETAIL-DATA-002` | `email` | string | Display as read-only Account identifier. |
| `FE-ACCOUNT-DETAIL-DATA-003` | `display_name` | string or null | Display as read-only text. |
| `FE-ACCOUNT-DETAIL-DATA-004` | `status` | string | Display as lifecycle state. |
| `FE-ACCOUNT-DETAIL-DATA-005` | `created_at` | RFC 3339 string | Display as localized timestamp. |
| `FE-ACCOUNT-DETAIL-DATA-006` | `updated_at` | RFC 3339 string | Display as localized timestamp. |
| `FE-ACCOUNT-DETAIL-DATA-007` | `deleted_at` | RFC 3339 string or null | Display when non-null. |

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

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-EDIT-001` | Provide an Edit action on the Account Detail page. |
| `FE-ACCOUNT-DETAIL-EDIT-002` | Navigate to `/admin/accounts/:id/edit` when Edit is activated. |
| `FE-ACCOUNT-DETAIL-EDIT-003` | Implement Edit as navigation rather than form submission. |
| `FE-ACCOUNT-DETAIL-EDIT-004` | Do not render editable Account fields on the Detail page. |
| `FE-ACCOUNT-DETAIL-EDIT-005` | Do not expose `PATCH /admin/accounts/{id}` from the Detail page. |
| `FE-ACCOUNT-DETAIL-EDIT-006` | Do not expose Save or Save Changes controls on the Detail page. |

The Edit action must remain clearly separate from the read-only Account information.

---

# 7. Account Detail Action Boundary

The Account Detail page contains only navigation actions required to view the account and reach the dedicated editing page.

## 7.1 Back Navigation

```text
← Back to Administrator Accounts
```

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-ACTION-001` | Navigate to `/admin/accounts`. |
| `FE-ACCOUNT-DETAIL-ACTION-002` | Use a native link. |
| `FE-ACCOUNT-DETAIL-ACTION-003` | Preserve normal browser navigation semantics. |

---

## 7.2 Edit Navigation

```text
[Edit]
```

Target:

```text
/admin/accounts/:id/edit
```

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-ACTION-004` | Provide the Edit action when Account details are available. |
| `FE-ACCOUNT-DETAIL-ACTION-005` | Navigate to the Account Edit route. |
| `FE-ACCOUNT-DETAIL-ACTION-006` | Use a native link or equivalent accessible navigation control. |

---

## 7.3 Actions Not Provided by This Page

The Account Detail page must not provide:

```text
Save Changes
Deactivate
Activate
Restore
Soft Delete
Hard Delete
```

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-ACTION-007` | Do not expose Account update mutations from the Detail page. |
| `FE-ACCOUNT-DETAIL-ACTION-008` | Do not expose Deactivate from the Detail page. |
| `FE-ACCOUNT-DETAIL-ACTION-009` | Do not expose Activate from the Detail page. |
| `FE-ACCOUNT-DETAIL-ACTION-010` | Do not expose Restore from the Detail page. |
| `FE-ACCOUNT-DETAIL-ACTION-011` | Do not expose Soft Delete from the Detail page. |
| `FE-ACCOUNT-DETAIL-ACTION-012` | Do not expose Hard Delete from the Detail page. |

Lifecycle mutations remain Account Management actions.

Account editing remains an Account Edit page responsibility.

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

The Account Detail page is a read-only consumer of the Account representation.

The page does not perform Account update or lifecycle API calls.

---

## 8.2 Authorization

Viewing an Account requires:

```text
account:view
```

| Operation | Permission |
|---|---|
| View Account | `account:view` |

The frontend must not treat client-held roles or permissions as proof of authorization.

The backend is the authorization authority.

---

## 8.3 Error Handling

| ID | Error | UI Behavior |
|---|---|---|
| `FE-ACCOUNT-DETAIL-API-006` | `400 INVALID_ACCOUNT_ID` | Render a non-editable invalid-resource error. |
| `FE-ACCOUNT-DETAIL-API-007` | `401` | Clear authentication state and redirect through Admin Shell behavior. |
| `FE-ACCOUNT-DETAIL-API-008` | `403` | Preserve authentication and show authorization error. |
| `FE-ACCOUNT-DETAIL-API-009` | `404 ACCOUNT_NOT_FOUND` | Show Not Found state. |
| `FE-ACCOUNT-DETAIL-API-010` | `500` | Show non-sensitive server error and retry action. |
| `FE-ACCOUNT-DETAIL-API-011` | Network failure | Show network error and retry action. |

The Account Detail page must not convert a read operation into a mutation when handling an error.

---

## 8.4 Cache

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

## 8.5 Problem Details

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

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-API-012` | Prefer the structured `code` for deterministic application behavior. |
| `FE-ACCOUNT-DETAIL-API-013` | Present user-facing messages appropriate to the read operation. |
| `FE-ACCOUNT-DETAIL-API-014` | Do not render raw stack traces or internal server details. |
| `FE-ACCOUNT-DETAIL-API-015` | Preserve generic fallback messaging when the response is malformed or lacks a known code. |

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
[loading account details]
```

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-STATE-004` | Keep the Admin Shell rendered. |
| `FE-ACCOUNT-DETAIL-STATE-005` | Show an explicit loading state in Main Content. |
| `FE-ACCOUNT-DETAIL-STATE-006` | Do not display invented Account values. |
| `FE-ACCOUNT-DETAIL-STATE-007` | Do not display or enable the Edit action before Account data is loaded. |

---

## 9.3 Loaded

The loaded page contains:

```text
Back to Administrator Accounts

Administrator Account
Display Name
Email
Status

Account Details

[Edit]
```

The page displays read-only Account information.

The page does not expose editable fields or lifecycle mutation actions.

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-STATE-008` | Display authoritative Account information. |
| `FE-ACCOUNT-DETAIL-STATE-009` | Render Account fields as read-only. |
| `FE-ACCOUNT-DETAIL-STATE-010` | Display the Edit action after Account data is available. |
| `FE-ACCOUNT-DETAIL-STATE-011` | Do not display Save or lifecycle mutation controls. |

---

## 9.4 Not Found

When the Account is unavailable:

```text
Administrator Account

Account not found.
The administrator account may no longer exist or may not be available.

[Back to Administrator Accounts]
```

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-STATE-012` | Explain that the requested Account is unavailable. |
| `FE-ACCOUNT-DETAIL-STATE-013` | Provide a direct route back to `/admin/accounts`. |
| `FE-ACCOUNT-DETAIL-STATE-014` | Do not display stale Account details as current data. |
| `FE-ACCOUNT-DETAIL-STATE-015` | Do not display the Edit action when Account data is unavailable. |

---

## 9.5 Authorization Error

For `403 Forbidden`:

```text
Administrator Account

You are not authorized to view this administrator account.
```

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-STATE-016` | Preserve authenticated state. |
| `FE-ACCOUNT-DETAIL-STATE-017` | Keep the error inside Main Content. |
| `FE-ACCOUNT-DETAIL-STATE-018` | Do not redirect to `/login` for `403`. |
| `FE-ACCOUNT-DETAIL-STATE-019` | Do not display Account details that were not authorized by the backend. |

---

## 9.6 Authentication Expiry

For `401 Unauthorized`:

```text
Authentication has expired. Redirecting to login.
```

The page must rely on the existing Authentication service to clear authentication state and navigate to `/login`.

The Account Detail page must not implement an independent authentication system.

---

## 9.7 General Error

```text
Unable to load this administrator account.

[Try Again]
```

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-STATE-020` | Provide a retry action for retryable failures. |
| `FE-ACCOUNT-DETAIL-STATE-021` | Keep server and network errors inside Main Content. |
| `FE-ACCOUNT-DETAIL-STATE-022` | Do not expose internal diagnostics. |
| `FE-ACCOUNT-DETAIL-STATE-023` | Preserve the Admin Shell. |

---

# 10. Responsive Layout

The page must inherit the Admin Shell responsive behavior and manage its own content layout.

| ID | Viewport | Layout |
|---|---|---|
| `FE-ACCOUNT-DETAIL-RESP-001` | `>= 1280px` | Two-column detail layout may be used for metadata and Account information. |
| `FE-ACCOUNT-DETAIL-RESP-002` | `768px–1279px` | Reduce content width and maintain readable content. |
| `FE-ACCOUNT-DETAIL-RESP-003` | `< 768px` | Stack all sections vertically. |
| `FE-ACCOUNT-DETAIL-RESP-004` | `< 768px` | Keep the Edit action reachable without horizontal scrolling. |
| `FE-ACCOUNT-DETAIL-RESP-005` | All viewports | Prevent unintended page-level horizontal scrolling. |
| `FE-ACCOUNT-DETAIL-RESP-006` | All viewports | Preserve keyboard accessibility. |
| `FE-ACCOUNT-DETAIL-RESP-007` | All viewports | Preserve authentication and authorization behavior. |

Suggested responsive structure:

```text
Desktop

┌───────────────────────────────────────────────────────────┐
│ Back to Administrator Accounts                            │
│                                                           │
│ Administrator Account                         [Edit]      │
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
| `FE-ACCOUNT-DETAIL-SEC-012` | Mutation boundary | The Detail page does not perform Account mutations. |
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
| `FE-ACCOUNT-DETAIL-A11Y-001` | Use semantic layout elements such as `main`, `header`, and `section`. |
| `FE-ACCOUNT-DETAIL-A11Y-002` | Provide exactly one primary `h1` for the page. |
| `FE-ACCOUNT-DETAIL-A11Y-003` | Use logical heading hierarchy for Account sections. |
| `FE-ACCOUNT-DETAIL-A11Y-004` | Use semantic read-only content elements rather than editable controls. |
| `FE-ACCOUNT-DETAIL-A11Y-005` | Provide accessible names for displayed Account data. |

---

## 12.2 Navigation

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-A11Y-006` | Back to Accounts uses a native link. |
| `FE-ACCOUNT-DETAIL-A11Y-007` | Edit navigation uses a native link or equivalent accessible navigation control. |
| `FE-ACCOUNT-DETAIL-A11Y-008` | Keyboard order follows the visual and logical reading order. |
| `FE-ACCOUNT-DETAIL-A11Y-009` | Focus is visibly indicated. |
| `FE-ACCOUNT-DETAIL-A11Y-010` | Focus is not obscured by responsive UI or sticky controls. |

---

## 12.3 Async Status

Loading and error states must be communicated to assistive technology.

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-DETAIL-A11Y-011` | Loading state is announced appropriately. |
| `FE-ACCOUNT-DETAIL-A11Y-012` | Read failures are announced appropriately. |
| `FE-ACCOUNT-DETAIL-A11Y-013` | Status messages do not steal focus unnecessarily. |

A suitable status region may use:

```html
<div role="status" aria-live="polite"></div>
```

Error messaging may use an appropriate assertive live region when necessary.

---

## 12.4 Status Presentation

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

| ID | Component | Responsibility |
|---|---|---|
| `FE-ACCOUNT-DETAIL-COMP-001` | `AccountDetailPage` | Coordinate route state, loading, Account data, navigation, and page layout. |
| `FE-ACCOUNT-DETAIL-COMP-002` | `AccountDetailHeader` | Render page title, summary, status, Back navigation, and Edit navigation. |
| `FE-ACCOUNT-DETAIL-COMP-003` | `AccountInformationSection` | Render read-only Account information. |
| `FE-ACCOUNT-DETAIL-COMP-004` | `AccountDetailsSection` | Render read-only metadata. |
| `FE-ACCOUNT-DETAIL-COMP-005` | `AccountStatus` | Render accessible lifecycle status. |
| `FE-ACCOUNT-DETAIL-COMP-006` | `AccountActionGroup` | Render Back and Edit navigation controls only. |
| `FE-ACCOUNT-DETAIL-COMP-007` | `AccountLoadingState` | Render initial loading state. |
| `FE-ACCOUNT-DETAIL-COMP-008` | `AccountErrorState` | Render non-recoverable and retryable errors. |
| `FE-ACCOUNT-DETAIL-COMP-009` | `AccountNotFoundState` | Render unavailable Account state. |

The page must not introduce:

```text
AccountForm
AccountSaveButton
AccountLifecycleSection
```

because Account editing and lifecycle mutation controls do not belong on the Detail page.

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
| `FE-ACCOUNT-DETAIL-TEST-UNIT-002` | `display_name=null` renders as a read-only value. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-003` | Email renders as read-only content. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-004` | Account ID renders as read-only content. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-005` | Active Account renders `Active`. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-006` | Inactive Account renders `Inactive`. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-007` | Deleted Account renders `Deleted` when represented by the API. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-008` | Edit action targets `/admin/accounts/:id/edit`. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-009` | Back action targets `/admin/accounts`. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-010` | No editable form controls are rendered. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-011` | No lifecycle mutation controls are rendered. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-012` | `401` is delegated to authentication handling. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-013` | `403` preserves authentication and renders an authorization error. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-014` | `404` renders Not Found. |

---

## 14.2 Integration

| ID | Test |
|---|---|
| `FE-ACCOUNT-DETAIL-TEST-INT-001` | Authenticated navigation from Account List to Account Detail works. |
| `FE-ACCOUNT-DETAIL-TEST-INT-002` | Account detail loads from `GET /admin/accounts/{id}`. |
| `FE-ACCOUNT-DETAIL-TEST-INT-003` | Account fields render as read-only values. |
| `FE-ACCOUNT-DETAIL-TEST-INT-004` | Edit action navigates to `/admin/accounts/{id}/edit`. |
| `FE-ACCOUNT-DETAIL-TEST-INT-005` | Back action returns to `/admin/accounts`. |
| `FE-ACCOUNT-DETAIL-TEST-INT-006` | `401` redirects to `/login`. |
| `FE-ACCOUNT-DETAIL-TEST-INT-007` | `403` preserves authentication. |
| `FE-ACCOUNT-DETAIL-TEST-INT-008` | `404` renders Account Not Found. |
| `FE-ACCOUNT-DETAIL-TEST-INT-009` | Account data is not persisted in browser storage. |
| `FE-ACCOUNT-DETAIL-TEST-INT-010` | Tokens do not appear in URLs or rendered output. |
| `FE-ACCOUNT-DETAIL-TEST-INT-011` | Account Detail performs no update request. |
| `FE-ACCOUNT-DETAIL-TEST-INT-012` | Account Detail performs no lifecycle mutation request. |

---

## 14.3 E2E

| ID | Test |
|---|---|
| `FE-ACCOUNT-DETAIL-TEST-E2E-001` | Login → Admin Shell → Accounts works. |
| `FE-ACCOUNT-DETAIL-TEST-E2E-002` | Account row opens the correct Account Detail route. |
| `FE-ACCOUNT-DETAIL-TEST-E2E-003` | Account details render correctly. |
| `FE-ACCOUNT-DETAIL-TEST-E2E-004` | Account detail fields remain read-only. |
| `FE-ACCOUNT-DETAIL-TEST-E2E-005` | Edit action navigates to the Account Edit route. |
| `FE-ACCOUNT-DETAIL-TEST-E2E-006` | Back navigation returns to Account Management. |
| `FE-ACCOUNT-DETAIL-TEST-E2E-007` | Authentication expiry redirects to `/login`. |
| `FE-ACCOUNT-DETAIL-TEST-E2E-008` | Unauthorized detail access displays `403` without logout. |
| `FE-ACCOUNT-DETAIL-TEST-E2E-009` | Mobile layout remains usable below `768px`. |

---

## 14.4 Accessibility

| ID | Test |
|---|---|
| `FE-ACCOUNT-DETAIL-TEST-A11Y-001` | Heading hierarchy is correct. |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-002` | Account information has accessible names. |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-003` | Back navigation is keyboard accessible. |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-004` | Edit navigation is keyboard accessible. |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-005` | Focus is visible. |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-006` | Async status is announced. |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-007` | Account status does not rely on color alone. |

---

## 14.5 Responsive

| ID | Test |
|---|---|
| `FE-ACCOUNT-DETAIL-TEST-RESP-001` | Desktop layout works at `>= 1280px`. |
| `FE-ACCOUNT-DETAIL-TEST-RESP-002` | Tablet layout works at `768px–1279px`. |
| `FE-ACCOUNT-DETAIL-TEST-RESP-003` | Mobile layout works below `768px`. |
| `FE-ACCOUNT-DETAIL-TEST-RESP-004` | No unintended page-level horizontal scrolling occurs. |
| `FE-ACCOUNT-DETAIL-TEST-RESP-005` | Back and Edit actions remain accessible on mobile. |
| `FE-ACCOUNT-DETAIL-TEST-RESP-006` | Admin Shell behavior is preserved at every viewport size. |
| `FE-ACCOUNT-DETAIL-TEST-RESP-007` | Authentication and authorization behavior is unchanged across viewports. |

---

# 15. Implementation Criteria

## 15.1 Route

| ID                           | Criteria                                                                  | Status |
| ---------------------------- | ------------------------------------------------------------------------- | ------ |
| `FE-ACCOUNT-DETAIL-IMPL-001` | `/admin/accounts/:id` renders inside the Admin Shell.                    | 🟡 Defined |
| `FE-ACCOUNT-DETAIL-IMPL-002` | Unauthenticated access is handled by Admin Shell authentication behavior. | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-003` | Accounts navigation remains active while viewing the detail page.       | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-004` | Back navigation returns to `/admin/accounts`.                            | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-005` | Edit action navigates to `/admin/accounts/:id/edit`.                     | 🟡 Defined |

---

## 15.2 Data

| ID                           | Criteria                                             | Status |
| ---------------------------- | ---------------------------------------------------- | ------ |
| `FE-ACCOUNT-DETAIL-IMPL-006` | Detail data comes from `GET /admin/accounts/{id}`.   | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-007` | Server Account response is authoritative.            | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-008` | Password credentials are never expected or rendered. | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-009` | Account data is not stored in browser persistence.   | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-010` | Account response cache policy remains `no-store`.    | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-011` | Account fields render as read-only values.            | 🟡 Defined |

---

## 15.3 Editing Boundary

| ID                           | Criteria                                                                  | Status |
| ---------------------------- | ------------------------------------------------------------------------- | ------ |
| `FE-ACCOUNT-DETAIL-IMPL-012` | No editable Account fields are rendered on the Detail page.               | 🟡 Defined |
| `FE-ACCOUNT-DETAIL-IMPL-013` | No Save or Save Changes control is rendered on the Detail page.           | 🟡 Defined |
| `FE-ACCOUNT-DETAIL-IMPL-014` | The Detail page does not submit `PATCH /admin/accounts/{id}`.             | 🟡 Defined |
| `FE-ACCOUNT-DETAIL-IMPL-015` | Edit navigation is provided from the Detail page.                         | 🟡 Defined |

---

## 15.4 Lifecycle Boundary

| ID                           | Criteria                                                                  | Status |
| ---------------------------- | ------------------------------------------------------------------------- | ------ |
| `FE-ACCOUNT-DETAIL-IMPL-016` | Deactivate is not exposed on the Detail page.                             | 🟡 Defined |
| `FE-ACCOUNT-DETAIL-IMPL-017` | Activate is not exposed on the Detail page.                               | 🟡 Defined |
| `FE-ACCOUNT-DETAIL-IMPL-018` | Restore is not exposed on the Detail page.                                | 🟡 Defined |
| `FE-ACCOUNT-DETAIL-IMPL-019` | Hard delete is not exposed.                                                | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-020` | Lifecycle mutation APIs are not called by the Detail page.                 | 🟡 Defined |

---

## 15.5 Security

| ID                           | Criteria                                                                            | Status |
| ---------------------------- | ----------------------------------------------------------------------------------- | ------ |
| `FE-ACCOUNT-DETAIL-IMPL-021` | Backend authentication remains authoritative.                                       | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-022` | Backend authorization remains authoritative.                                        | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-023` | Tokens are only sent through the `Authorization` header for protected Account APIs. | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-024` | Tokens never appear in URLs.                                                        | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-025` | Tokens never render in the UI.                                                      | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-026` | Tokens never appear in logs.                                                        | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-027` | Passwords and password hashes never render.                                         | 🟢 Implemented |

---

## 15.6 Accessibility

| ID                           | Criteria                                              | Status |
| ---------------------------- | ----------------------------------------------------- | ------ |
| `FE-ACCOUNT-DETAIL-IMPL-028` | Semantic page structure is used.                      | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-029` | Primary page heading is present.                      | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-030` | Account information has accessible labels or names.   | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-031` | Navigation and actions are keyboard accessible.       | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-032` | Focus is visible and not obscured.                    | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-033` | Async status is accessible.                           | 🟢 Implemented |
| `FE-ACCOUNT-DETAIL-IMPL-034` | Account lifecycle status does not rely only on color. | 🟢 Implemented |

---

# 16. Traceability

## Requirements → Design → Backend → Test

| Requirement | Frontend Design | Backend Contract | Test |
|---|---|---|---|
| `ADM-AUTH-001` | `FE-ACCOUNT-DETAIL-ROUTE-001` | Authentication domain | `FE-ACCOUNT-DETAIL-TEST-E2E-001` |
| `ADM-AUTH-004` | `FE-ACCOUNT-DETAIL-REQ-002` | `AC_UC_03`, `AC_API_03` | `FE-ACCOUNT-DETAIL-TEST-INT-002` |
| `ADM-AUTH-005` | `FE-ACCOUNT-DETAIL-EDIT-001` | `AC_UC_04`, `AC_API_04` | `FE-ACCOUNT-DETAIL-TEST-E2E-005` |
| `AC_UC_03` | `FE-ACCOUNT-DETAIL-API-001` | `GET /admin/accounts/{id}` | `FE-ACCOUNT-DETAIL-TEST-INT-002` |
| `account:view` | `FE-ACCOUNT-DETAIL-API-001` | Authorization domain | `FE-ACCOUNT-DETAIL-TEST-INT-007` |
| Authentication `401` | `FE-ACCOUNT-DETAIL-API-007` | Authentication domain | `FE-ACCOUNT-DETAIL-TEST-E2E-007` |
| Authorization `403` | `FE-ACCOUNT-DETAIL-API-008` | Authorization domain | `FE-ACCOUNT-DETAIL-TEST-INT-007` |
| Account Detail navigation | `FE-ACCOUNT-DETAIL-ACTION-001` | Frontend navigation | `FE-ACCOUNT-DETAIL-TEST-INT-005` |
| Edit navigation | `FE-ACCOUNT-DETAIL-EDIT-001` | `PAGE-ADM-010` | `FE-ACCOUNT-DETAIL-TEST-INT-004` |
| Admin Shell | `FE-ACCOUNT-DETAIL-ROUTE-001`, `FE-ACCOUNT-DETAIL-RESP-*` | `admin_shell.md` | `FE-ACCOUNT-DETAIL-TEST-E2E-001` |

## Domain References

| Domain | Relevant References |
|---|---|
| Requirements | `ADM-AUTH-001` to `ADM-AUTH-005`, `CNT-ADMIN-*`, `SCP-009` |
| Sitemap | `PAGE-ADM-008`, `PAGE-ADM-010` |
| Account | `AC_UC_03`, `AC_API_03`, `AC_UC_04`, `AC_API_04` |
| Authentication | Protected route behavior, bearer authentication, `401` handling |
| Authorization | `account:view`, deny-by-default, `403` |
| Admin Shell | Shell layout, route protection, active Accounts navigation, responsive behavior |
| Account Management | `/admin/accounts`, View navigation, Edit navigation, lifecycle action separation |

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
    └── Edit navigation
            ↓
        Account Edit
    ↓
Account API
    ↓
Authentication
    ↓
Authorization
    ↓
Tests
```
