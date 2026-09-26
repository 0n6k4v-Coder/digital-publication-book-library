# Frontend Admin Account Detail Page

## Table of Contents

1. [Scope](#1-scope)
2. [Route](#2-route)
3. [Requirements](#3-requirements)
4. [Page Structure](#4-page-structure)
5. [Account Data Contract](#5-account-data-contract)
6. [Navigation](#6-navigation)
7. [UI States](#7-ui-states)
8. [Responsive Layout](#8-responsive-layout)
9. [Security](#9-security)
10. [Accessibility](#10-accessibility)
11. [Component Structure](#11-component-structure)
12. [Testing](#12-testing)
13. [Implementation Checklist](#13-implementation-checklist)

---

# 1. Scope

This document defines the protected **Administrator Account Detail** frontend page.

The page is a read-only view of one administrator account.

The Account Detail page must not contain account mutation functionality.

The separate Account Edit page is responsible for:

- Editing Account fields.
- Changing Account credentials.
- Activating an Account.
- Deactivating an Account.
- Restoring a soft-deleted Account.
- Other Account mutations defined by the Account domain.

## Responsibilities

| ID | Responsibility |
|---|---|
| `FE-ACCOUNT-DETAIL-SCOPE-001` | Load one administrator Account. |
| `FE-ACCOUNT-DETAIL-SCOPE-002` | Display the authoritative Account representation. |
| `FE-ACCOUNT-DETAIL-SCOPE-003` | Display Account lifecycle state. |
| `FE-ACCOUNT-DETAIL-SCOPE-004` | Display Account metadata. |
| `FE-ACCOUNT-DETAIL-SCOPE-005` | Provide navigation to Account Edit when editing is available to the user. |
| `FE-ACCOUNT-DETAIL-SCOPE-006` | Provide navigation back to Administrator Accounts. |
| `FE-ACCOUNT-DETAIL-SCOPE-007` | Handle loading, empty, authentication, authorization, not-found, server, and network states. |
| `FE-ACCOUNT-DETAIL-SCOPE-008` | Preserve Admin Shell behavior and navigation state. |

## Out of Scope

| ID | Excluded Area |
|---|---|
| `FE-ACCOUNT-DETAIL-OOS-001` | Account field editing. |
| `FE-ACCOUNT-DETAIL-OOS-002` | Email changes. |
| `FE-ACCOUNT-DETAIL-OOS-003` | Password changes. |
| `FE-ACCOUNT-DETAIL-OOS-004` | Account activation. |
| `FE-ACCOUNT-DETAIL-OOS-005` | Account deactivation. |
| `FE-ACCOUNT-DETAIL-OOS-006` | Account soft deletion. |
| `FE-ACCOUNT-DETAIL-OOS-007` | Account restoration. |
| `FE-ACCOUNT-DETAIL-OOS-008` | Account hard deletion. |
| `FE-ACCOUNT-DETAIL-OOS-009` | Form state and mutation state. |
| `FE-ACCOUNT-DETAIL-OOS-010` | Authentication implementation. |
| `FE-ACCOUNT-DETAIL-OOS-011` | Authorization implementation. |
| `FE-ACCOUNT-DETAIL-OOS-012` | Account persistence and database behavior. |

The Account Detail page is intentionally read-only.

---

# 2. Route

The canonical Account Detail route is:

```text
/admin/accounts/:id
````

The canonical Account Edit route is:

```text id="ke15gj"
/admin/accounts/:id/edit
```

The two pages have separate responsibilities:

```text id="0f94ws"
/admin/accounts
    │
    ├── /admin/accounts/:id
    │      └── Account Detail
    │
    └── /admin/accounts/:id/edit
           └── Account Edit
```

## Route Contract

| ID                            | Route                 | Access                         | Behavior                                          |
| ----------------------------- | --------------------- | ------------------------------ | ------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-ROUTE-001` | `/admin/accounts/:id` | Authenticated                  | Render Account Detail.                            |
| `FE-ACCOUNT-DETAIL-ROUTE-002` | `/admin/accounts/:id` | Bootstrap                      | Keep route pending until authentication resolves. |
| `FE-ACCOUNT-DETAIL-ROUTE-003` | `/admin/accounts/:id` | Unauthenticated                | Redirect to `/login`.                             |
| `FE-ACCOUNT-DETAIL-ROUTE-004` | `/admin/accounts/:id` | Authenticated but unauthorized | Render authorization error.                       |
| `FE-ACCOUNT-DETAIL-ROUTE-005` | `/admin/accounts/:id` | Invalid or unavailable Account | Render Account Not Found.                         |

The Account ID is represented by the route parameter:

```text id="9fyars"
:id
```

The route must never contain:

```text id="lxgapg"
access_token
refresh_token
password
password_hash
```

or any other authentication credential.

## Route Relationships

```text id="jdem5p"
Administrator Accounts
        │
        ├── View Account
        │      └── /admin/accounts/:id
        │
        └── Edit Account
               └── /admin/accounts/:id/edit
```

---

# 3. Requirements

| ID                          | Requirement                                                      | Repository Reference                     |
| --------------------------- | ---------------------------------------------------------------- | ---------------------------------------- |
| `FE-ACCOUNT-DETAIL-REQ-001` | Provide a protected Account Detail route.                        | `PAGE-ADM-010` extension, `ADM-AUTH-004` |
| `FE-ACCOUNT-DETAIL-REQ-002` | Load one Account from the Account API.                           | `AC_UC_03`, `AC_API_03`                  |
| `FE-ACCOUNT-DETAIL-REQ-003` | Display the authoritative Account representation.                | Account domain                           |
| `FE-ACCOUNT-DETAIL-REQ-004` | Display Account lifecycle status.                                | `AC_REQ_FC_05`                           |
| `FE-ACCOUNT-DETAIL-REQ-005` | Display Account metadata.                                        | Account domain                           |
| `FE-ACCOUNT-DETAIL-REQ-006` | Provide navigation to Account Edit without implementing editing. | `ADM-AUTH-005`                           |
| `FE-ACCOUNT-DETAIL-REQ-007` | Provide navigation back to Account Management.                   | `PAGE-ADM-008`                           |
| `FE-ACCOUNT-DETAIL-REQ-008` | Treat backend authentication as authoritative.                   | Authentication domain                    |
| `FE-ACCOUNT-DETAIL-REQ-009` | Treat backend authorization as authoritative.                    | Authorization domain                     |
| `FE-ACCOUNT-DETAIL-REQ-010` | Handle `401 Unauthorized`.                                       | Authentication domain                    |
| `FE-ACCOUNT-DETAIL-REQ-011` | Handle `403 Forbidden`.                                          | Authorization domain                     |
| `FE-ACCOUNT-DETAIL-REQ-012` | Handle `404 ACCOUNT_NOT_FOUND`.                                  | Account API                              |
| `FE-ACCOUNT-DETAIL-REQ-013` | Do not expose credentials or password data.                      | Account Security                         |
| `FE-ACCOUNT-DETAIL-REQ-014` | Do not persist Account data in browser storage.                  | Account API                              |
| `FE-ACCOUNT-DETAIL-REQ-015` | Provide keyboard-accessible navigation.                          | Admin Shell                              |
| `FE-ACCOUNT-DETAIL-REQ-016` | Provide visible focus states.                                    | Admin Shell                              |
| `FE-ACCOUNT-DETAIL-REQ-017` | Provide accessible loading and error feedback.                   | Frontend accessibility                   |
| `FE-ACCOUNT-DETAIL-REQ-018` | Prevent unintended page-level horizontal scrolling.              | Admin Shell                              |

---

# 4. Page Structure

## 4.1 Page Header

The page header should use the following structure:

```text id="bvk514"
Administrator Account

Library Administrator
admin@example.com
[Active]

[Back to Accounts]    [Edit Account]
```

| ID                         | Requirement                                                                                        |
| -------------------------- | -------------------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-UI-001` | Use `Administrator Account` as the primary page heading.                                           |
| `FE-ACCOUNT-DETAIL-UI-002` | Display `display_name` when available.                                                             |
| `FE-ACCOUNT-DETAIL-UI-003` | Display the Account email.                                                                         |
| `FE-ACCOUNT-DETAIL-UI-004` | Display lifecycle status using text.                                                               |
| `FE-ACCOUNT-DETAIL-UI-005` | Provide navigation to `/admin/accounts`.                                                           |
| `FE-ACCOUNT-DETAIL-UI-006` | Provide navigation to `/admin/accounts/:id/edit` when the current user may access Account editing. |
| `FE-ACCOUNT-DETAIL-UI-007` | Keep the page read-only.                                                                           |

The page must not include:

```text id="uycozs"
Save
Update
Submit
Activate
Deactivate
Restore
Delete
Change Email
Change Password
```

as executable controls.

The only mutation-related control on the page is navigation to the separate Account Edit page.

---

## 4.2 Account Information

The primary content area should display Account information in a read-only presentation.

```text id="pg3r7x"
Account Information

Display name
Library Administrator

Email
admin@example.com

Status
Active
```

| ID                            | Field        | Source                 | Editable |
| ----------------------------- | ------------ | ---------------------- | -------- |
| `FE-ACCOUNT-DETAIL-FIELD-001` | Display name | `display_name`         | No       |
| `FE-ACCOUNT-DETAIL-FIELD-002` | Email        | `email`                | No       |
| `FE-ACCOUNT-DETAIL-FIELD-003` | Status       | `status`, `deleted_at` | No       |

Values should be presented as readable text rather than disabled form fields.

Disabled form controls should not be used simply to simulate read-only content.

---

## 4.3 Account Metadata

The page should display non-editable Account metadata:

```text id="sy22u2"
Account Details

Account ID
019...

Created
September 23, 2026, 17:00

Last updated
September 23, 2026, 17:30

Deleted
—
```

| ID                           | Metadata                                             |
| ---------------------------- | ---------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-META-001` | Display `id`.                                        |
| `FE-ACCOUNT-DETAIL-META-002` | Display `created_at`.                                |
| `FE-ACCOUNT-DETAIL-META-003` | Display `updated_at`.                                |
| `FE-ACCOUNT-DETAIL-META-004` | Display `deleted_at` when non-null.                  |
| `FE-ACCOUNT-DETAIL-META-005` | Never display `password` or `password_hash`.         |
| `FE-ACCOUNT-DETAIL-META-006` | Never expose internal credential or security fields. |

Account timestamps should be localized for display while retaining their underlying timestamp value.

---

## 4.4 Status

The UI maps the Account response to a display state:

| Condition                               | UI Status  |
| --------------------------------------- | ---------- |
| `status=active` and `deleted_at=null`   | `Active`   |
| `status=inactive` and `deleted_at=null` | `Inactive` |
| `deleted_at != null`                    | `Deleted`  |

`Deleted` is a derived presentation state.

It must not be treated as an additional Account `status` value.

Status must not rely on color alone.

---

## 4.5 Edit Navigation

The Edit Account control navigates to:

```text id="574inx"
/admin/accounts/:id/edit
```

| ID                         | Requirement                                               |
| -------------------------- | --------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-UI-008` | Use a real link for Edit Account navigation.              |
| `FE-ACCOUNT-DETAIL-UI-009` | Preserve the current Account ID in the destination route. |
| `FE-ACCOUNT-DETAIL-UI-010` | Do not mutate Account state from the Detail page.         |
| `FE-ACCOUNT-DETAIL-UI-011` | Do not place unsaved Account state in the URL.            |

Authorization remains server-side.

The presence or absence of the Edit link is a usability decision only and is not itself an authorization boundary.

---

# 5. Account Data Contract

The page consumes the Account response defined by the Account domain.

Expected representation:

```json id="orrslz"
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

| ID                           | Field          | Type                    | Detail Page Rule             |
| ---------------------------- | -------------- | ----------------------- | ---------------------------- |
| `FE-ACCOUNT-DETAIL-DATA-001` | `id`           | UUID string             | Display read-only.           |
| `FE-ACCOUNT-DETAIL-DATA-002` | `email`        | string                  | Display read-only.           |
| `FE-ACCOUNT-DETAIL-DATA-003` | `display_name` | string or null          | Display read-only.           |
| `FE-ACCOUNT-DETAIL-DATA-004` | `status`       | string                  | Display lifecycle state.     |
| `FE-ACCOUNT-DETAIL-DATA-005` | `created_at`   | RFC 3339 string         | Display localized timestamp. |
| `FE-ACCOUNT-DETAIL-DATA-006` | `updated_at`   | RFC 3339 string         | Display localized timestamp. |
| `FE-ACCOUNT-DETAIL-DATA-007` | `deleted_at`   | RFC 3339 string or null | Display when non-null.       |

The following fields must never be expected or rendered:

```text id="xpyoj1"
password
password_hash
```

Unknown response fields must not automatically become UI fields.

The server response is authoritative.

The frontend must not fabricate missing Account values.

---

# 6. Navigation

## 6.1 Back to Account Management

The page provides:

```text id="izwhvp"
Back to Accounts
```

Destination:

```text id="pgtnxc"
/admin/accounts
```

| ID                          | Requirement                                             |
| --------------------------- | ------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-NAV-001` | Use a native link.                                      |
| `FE-ACCOUNT-DETAIL-NAV-002` | Return to the Account Management page.                  |
| `FE-ACCOUNT-DETAIL-NAV-003` | Do not require Account mutation state to navigate back. |

---

## 6.2 Edit Account

The page provides:

```text id="0aedfx"
Edit Account
```

Destination:

```text id="i6glzq"
/admin/accounts/:id/edit
```

| ID                          | Requirement                                         |
| --------------------------- | --------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-NAV-004` | Use a native link.                                  |
| `FE-ACCOUNT-DETAIL-NAV-005` | Preserve the current Account ID.                    |
| `FE-ACCOUNT-DETAIL-NAV-006` | Do not perform an update before navigation.         |
| `FE-ACCOUNT-DETAIL-NAV-007` | Treat the edit page as a separate feature boundary. |

---

# 7. UI States

## 7.1 Authentication Bootstrap

When authentication state is:

```text id="ye3sxc"
unknown
```

the route must remain pending.

The page must not prematurely redirect to `/login`.

| ID                            | Requirement                                             |
| ----------------------------- | ------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-STATE-001` | Preserve Admin Shell authentication bootstrap behavior. |
| `FE-ACCOUNT-DETAIL-STATE-002` | Do not treat `unknown` as unauthenticated.              |

---

## 7.2 Loading

While loading the Account:

```text id="izj204"
Administrator Account

[loading]

Account Information
[loading]

Account Details
[loading]
```

| ID                            | Requirement                                                      |
| ----------------------------- | ---------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-STATE-003` | Keep the Admin Shell rendered.                                   |
| `FE-ACCOUNT-DETAIL-STATE-004` | Show a clear Main Content loading state.                         |
| `FE-ACCOUNT-DETAIL-STATE-005` | Do not display fabricated Account values.                        |
| `FE-ACCOUNT-DETAIL-STATE-006` | Do not render mutation controls as active actions while loading. |

---

## 7.3 Loaded

The loaded state displays:

```text id="y4nrw3"
Administrator Account

Library Administrator
admin@example.com
Active

Account Information
...

Account Details
...

[Back to Accounts]    [Edit Account]
```

The page remains read-only.

---

## 7.4 Not Found

For:

```text id="y4ckyx"
404 ACCOUNT_NOT_FOUND
```

display:

```text id="qallhv"
Administrator Account

Account not found.

The administrator account is no longer available.

[Back to Accounts]
```

| ID                            | Requirement                                                |
| ----------------------------- | ---------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-STATE-007` | Do not display stale Account information as current.       |
| `FE-ACCOUNT-DETAIL-STATE-008` | Provide navigation back to Account Management.             |
| `FE-ACCOUNT-DETAIL-STATE-009` | Do not offer mutation controls for an unavailable Account. |

---

## 7.5 Authorization Error

For:

```text id="wyt2n3"
403 Forbidden
```

display:

```text id="tejp9f"
Administrator Account

You are not authorized to view this administrator account.
```

The authenticated session must remain intact.

The page must not redirect to `/login` for `403`.

---

## 7.6 Authentication Expiry

For:

```text id="tp7fes"
401 Unauthorized
```

the page must rely on the existing Authentication service behavior.

Expected result:

```text id="2qdywi"
clear authentication state
        ↓
redirect to /login
```

The Account Detail page must not implement a separate authentication mechanism.

---

## 7.7 General Error

For network or unexpected server errors:

```text id="zqcs9z"
Unable to load this administrator account.

[Try Again]
```

| ID                            | Requirement                               |
| ----------------------------- | ----------------------------------------- |
| `FE-ACCOUNT-DETAIL-STATE-010` | Keep errors inside Main Content.          |
| `FE-ACCOUNT-DETAIL-STATE-011` | Provide retry behavior where appropriate. |
| `FE-ACCOUNT-DETAIL-STATE-012` | Do not expose internal diagnostics.       |
| `FE-ACCOUNT-DETAIL-STATE-013` | Preserve the Admin Shell.                 |

---

# 8. Responsive Layout

The page inherits the Admin Shell responsive behavior and adapts its Account content accordingly.

| ID                           | Viewport       | Layout                                                  |
| ---------------------------- | -------------- | ------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-RESP-001` | `>= 1280px`    | Two-column information presentation may be used.        |
| `FE-ACCOUNT-DETAIL-RESP-002` | `768px–1279px` | Reduce content width while preserving readable spacing. |
| `FE-ACCOUNT-DETAIL-RESP-003` | `< 768px`      | Stack Account sections vertically.                      |
| `FE-ACCOUNT-DETAIL-RESP-004` | `< 768px`      | Keep navigation controls reachable and readable.        |
| `FE-ACCOUNT-DETAIL-RESP-005` | All viewports  | Prevent unintended page-level horizontal scrolling.     |
| `FE-ACCOUNT-DETAIL-RESP-006` | All viewports  | Preserve keyboard navigation.                           |
| `FE-ACCOUNT-DETAIL-RESP-007` | All viewports  | Preserve authentication and authorization behavior.     |

Suggested layout:

```text id="qz3vwe"
Desktop

┌───────────────────────────────────────────────────────┐
│ Administrator Account                    [Edit]       │
│ Library Administrator                                 │
│ admin@example.com                         Active      │
│                                                       │
│ ┌────────────────────────┐ ┌────────────────────────┐ │
│ │ Account Information    │ │ Account Details        │ │
│ │                        │ │                        │ │
│ │ Display name           │ │ Account ID             │ │
│ │ Email                  │ │ Created                │ │
│ │ Status                 │ │ Last updated           │ │
│ │                        │ │ Deleted                │ │
│ └────────────────────────┘ └────────────────────────┘ │
└───────────────────────────────────────────────────────┘
```

```text id="6ag2f2"
Mobile

Administrator Account
Library Administrator
admin@example.com
Active

Account Information
Display name
Email
Status

Account Details
Account ID
Created
Last updated
Deleted

[Back to Accounts]
[Edit Account]
```

---

# 9. Security

The Account Detail page must follow the Authentication, Authorization, and Account security contracts.

| ID                          | Security Rule            | Requirement                                                      |
| --------------------------- | ------------------------ | ---------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-SEC-001` | Protected Route          | Route requires authenticated Admin Shell state.                  |
| `FE-ACCOUNT-DETAIL-SEC-002` | Authentication Authority | Backend remains authoritative for authentication.                |
| `FE-ACCOUNT-DETAIL-SEC-003` | Authorization Authority  | Backend remains authoritative for authorization.                 |
| `FE-ACCOUNT-DETAIL-SEC-004` | Access Token             | Bearer credentials are sent only through `Authorization`.        |
| `FE-ACCOUNT-DETAIL-SEC-005` | URL Security             | Tokens never appear in URLs.                                     |
| `FE-ACCOUNT-DETAIL-SEC-006` | UI Security              | Tokens are never rendered.                                       |
| `FE-ACCOUNT-DETAIL-SEC-007` | Logging                  | Tokens are never logged.                                         |
| `FE-ACCOUNT-DETAIL-SEC-008` | Credential Security      | Passwords and password hashes are never rendered.                |
| `FE-ACCOUNT-DETAIL-SEC-009` | Client Authorization     | Client-side roles or permissions are not authorization evidence. |
| `FE-ACCOUNT-DETAIL-SEC-010` | Persistence              | Account data is not stored in browser persistence.               |
| `FE-ACCOUNT-DETAIL-SEC-011` | Cache                    | Account responses remain `no-store`.                             |
| `FE-ACCOUNT-DETAIL-SEC-012` | Error Exposure           | Internal server diagnostics are not exposed.                     |

The Edit link is not a security boundary.

A user who can reach the Account Detail page must still be authorized by the backend for any operation performed on the Edit page.

---

# 10. Accessibility

## 10.1 Semantic Structure

| ID                           | Requirement                                                                     |
| ---------------------------- | ------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-A11Y-001` | Use semantic layout elements.                                                   |
| `FE-ACCOUNT-DETAIL-A11Y-002` | Provide one primary `h1`.                                                       |
| `FE-ACCOUNT-DETAIL-A11Y-003` | Use logical heading hierarchy.                                                  |
| `FE-ACCOUNT-DETAIL-A11Y-004` | Use semantic definition-style or equivalent read-only information presentation. |

---

## 10.2 Navigation

| ID                           | Requirement                                       |
| ---------------------------- | ------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-A11Y-005` | Back to Accounts is keyboard accessible.          |
| `FE-ACCOUNT-DETAIL-A11Y-006` | Edit Account is keyboard accessible.              |
| `FE-ACCOUNT-DETAIL-A11Y-007` | Links use descriptive accessible names.           |
| `FE-ACCOUNT-DETAIL-A11Y-008` | Keyboard order follows the logical reading order. |
| `FE-ACCOUNT-DETAIL-A11Y-009` | Focus is visibly indicated.                       |
| `FE-ACCOUNT-DETAIL-A11Y-010` | Focus is not obscured by responsive UI.           |

---

## 10.3 Status and Async Feedback

| ID                           | Requirement                                                          |
| ---------------------------- | -------------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-A11Y-011` | Loading state is communicated accessibly.                            |
| `FE-ACCOUNT-DETAIL-A11Y-012` | Error state is communicated accessibly.                              |
| `FE-ACCOUNT-DETAIL-A11Y-013` | Account status does not rely on color alone.                         |
| `FE-ACCOUNT-DETAIL-A11Y-014` | Not-found and authorization messages are programmatically available. |

A status region may use:

```html id="v3glbe"
<div role="status" aria-live="polite"></div>
```

Error presentation should use an appropriate semantic mechanism and should not rely on visual styling alone.

---

# 11. Component Structure

The page should remain feature-local and separate from the Account Edit implementation.

Suggested structure:

```text id="gp50ma"
src/pages/accounts/
├── AccountListPage
├── AccountDetailPage
├── AccountCreatePage
└── AccountEditPage
```

Suggested Account Detail components:

```text id="v7kyr4"
AccountDetailPage
├── AccountDetailHeader
├── AccountSummary
├── AccountInformation
├── AccountMetadata
├── AccountStatus
├── AccountLoadingState
├── AccountErrorState
└── AccountNotFoundState
```

| ID                           | Component              | Responsibility                                               |
| ---------------------------- | ---------------------- | ------------------------------------------------------------ |
| `FE-ACCOUNT-DETAIL-COMP-001` | `AccountDetailPage`    | Coordinate route, loading, data, and read-only presentation. |
| `FE-ACCOUNT-DETAIL-COMP-002` | `AccountDetailHeader`  | Render title, Account summary, status, and navigation.       |
| `FE-ACCOUNT-DETAIL-COMP-003` | `AccountInformation`   | Render core Account fields.                                  |
| `FE-ACCOUNT-DETAIL-COMP-004` | `AccountMetadata`      | Render Account metadata and timestamps.                      |
| `FE-ACCOUNT-DETAIL-COMP-005` | `AccountStatus`        | Render lifecycle state accessibly.                           |
| `FE-ACCOUNT-DETAIL-COMP-006` | `AccountLoadingState`  | Render loading UI.                                           |
| `FE-ACCOUNT-DETAIL-COMP-007` | `AccountErrorState`    | Render generic and authorization errors.                     |
| `FE-ACCOUNT-DETAIL-COMP-008` | `AccountNotFoundState` | Render unavailable Account state.                            |

Account mutations must not be implemented inside these components.

API communication should remain in:

```text id="twll0b"
src/services/
```

Type definitions should remain in:

```text id="i0mw9m"
src/types/
```

The page should use the existing React, TypeScript, Vite, and native CSS architecture.

---

# 12. Testing

## 12.1 Unit

| ID                                | Test                                                  |
| --------------------------------- | ----------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-001` | Account response renders correctly.                   |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-002` | Null `display_name` renders correctly.                |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-003` | Active Account displays `Active`.                     |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-004` | Inactive Account displays `Inactive`.                 |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-005` | Deleted Account displays `Deleted`.                   |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-006` | Edit navigation contains the correct Account ID.      |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-007` | No mutation controls are rendered by the Detail page. |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-008` | `404` produces the Not Found state.                   |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-009` | `403` produces the authorization state.               |
| `FE-ACCOUNT-DETAIL-TEST-UNIT-010` | Loading state does not fabricate Account data.        |

---

## 12.2 Integration

| ID                               | Test                                                        |
| -------------------------------- | ----------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-TEST-INT-001` | Authenticated Account List navigation opens Account Detail. |
| `FE-ACCOUNT-DETAIL-TEST-INT-002` | Account Detail requests `GET /admin/accounts/{id}`.         |
| `FE-ACCOUNT-DETAIL-TEST-INT-003` | Bearer credentials are sent using `Authorization`.          |
| `FE-ACCOUNT-DETAIL-TEST-INT-004` | `401` delegates to authentication handling.                 |
| `FE-ACCOUNT-DETAIL-TEST-INT-005` | `403` preserves authentication.                             |
| `FE-ACCOUNT-DETAIL-TEST-INT-006` | `404` renders Account Not Found.                            |
| `FE-ACCOUNT-DETAIL-TEST-INT-007` | Account data is not persisted in browser storage.           |
| `FE-ACCOUNT-DETAIL-TEST-INT-008` | Tokens do not appear in URLs.                               |
| `FE-ACCOUNT-DETAIL-TEST-INT-009` | Edit navigation opens `/admin/accounts/:id/edit`.           |

---

## 12.3 E2E

| ID                               | Test                                                          |
| -------------------------------- | ------------------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-TEST-E2E-001` | Login → Admin Shell → Accounts works.                         |
| `FE-ACCOUNT-DETAIL-TEST-E2E-002` | Account row opens the correct Detail page.                    |
| `FE-ACCOUNT-DETAIL-TEST-E2E-003` | Account information renders correctly.                        |
| `FE-ACCOUNT-DETAIL-TEST-E2E-004` | Account metadata renders correctly.                           |
| `FE-ACCOUNT-DETAIL-TEST-E2E-005` | Edit Account navigation opens the separate edit route.        |
| `FE-ACCOUNT-DETAIL-TEST-E2E-006` | Back navigation returns to Account Management.                |
| `FE-ACCOUNT-DETAIL-TEST-E2E-007` | Authentication expiry redirects to `/login`.                  |
| `FE-ACCOUNT-DETAIL-TEST-E2E-008` | Unauthorized Account access displays the authorization state. |
| `FE-ACCOUNT-DETAIL-TEST-E2E-009` | Mobile layout remains usable below `768px`.                   |

---

## 12.4 Accessibility

| ID                                | Test                                      |
| --------------------------------- | ----------------------------------------- |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-001` | Heading hierarchy is correct.             |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-002` | Navigation links are keyboard accessible. |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-003` | Focus is visible.                         |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-004` | Loading state is accessible.              |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-005` | Errors are accessible.                    |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-006` | Status does not rely on color alone.      |
| `FE-ACCOUNT-DETAIL-TEST-A11Y-007` | Mobile layout remains accessible.         |

---

## 12.5 Responsive

| ID                                | Test                                                  |
| --------------------------------- | ----------------------------------------------------- |
| `FE-ACCOUNT-DETAIL-TEST-RESP-001` | Desktop layout works at `>= 1280px`.                  |
| `FE-ACCOUNT-DETAIL-TEST-RESP-002` | Tablet layout works at `768px–1279px`.                |
| `FE-ACCOUNT-DETAIL-TEST-RESP-003` | Mobile layout works below `768px`.                    |
| `FE-ACCOUNT-DETAIL-TEST-RESP-004` | No unintended page-level horizontal scrolling occurs. |
| `FE-ACCOUNT-DETAIL-TEST-RESP-005` | Navigation remains usable at all viewport sizes.      |
| `FE-ACCOUNT-DETAIL-TEST-RESP-006` | Admin Shell behavior remains unchanged.               |

---

# 13. Implementation Checklist

## 13.1 Route

| ID                           | Checklist Item                                                       | Status | Reason |
| ---------------------------- | -------------------------------------------------------------------- | ------ | ------ |
| `FE-ACCOUNT-DETAIL-IMPL-001` | `/admin/accounts/:id` renders inside the Admin Shell.                | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-002` | Authentication bootstrap behavior is inherited from the Admin Shell. | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-003` | Accounts navigation remains active.                                  | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-004` | Back navigation returns to `/admin/accounts`.                        | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-005` | Edit navigation targets `/admin/accounts/:id/edit`.                  | ⬜     |        |

---

## 13.2 Data

| ID                           | Checklist Item                                        | Status | Reason |
| ---------------------------- | ----------------------------------------------------- | ------ | ------ |
| `FE-ACCOUNT-DETAIL-IMPL-006` | Data is loaded through `GET /admin/accounts/{id}`.    | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-007` | Server response is authoritative.                     | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-008` | Password and password-hash fields are never rendered. | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-009` | Account data is not stored in browser persistence.    | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-010` | Account API responses remain `no-store`.              | ⬜     |        |

---

## 13.3 Concern Separation

| ID                           | Checklist Item                                                                          | Status | Reason |
| ---------------------------- | --------------------------------------------------------------------------------------- | ------ | ------ |
| `FE-ACCOUNT-DETAIL-IMPL-011` | Account Detail contains no form submission logic.                                       | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-012` | Account Detail contains no Account mutation API calls.                                  | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-013` | Account Detail contains no lifecycle mutation controls.                                 | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-014` | Account Edit is implemented as a separate page and concern.                             | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-015` | Credential changes are implemented outside Account Detail.                              | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-016` | Detail-page read state and edit-page mutation state are not shared as one page concern. | ⬜     |        |

---

## 13.4 Security

| ID                           | Checklist Item                                           | Status | Reason |
| ---------------------------- | -------------------------------------------------------- | ------ | ------ |
| `FE-ACCOUNT-DETAIL-IMPL-017` | Backend authentication remains authoritative.            | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-018` | Backend authorization remains authoritative.             | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-019` | Tokens are only sent through the `Authorization` header. | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-020` | Tokens never appear in URLs.                             | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-021` | Tokens never render.                                     | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-022` | Tokens never log.                                        | ⬜     |        |

---

## 13.5 Accessibility

| ID                           | Checklist Item                            | Status | Reason |
| ---------------------------- | ----------------------------------------- | ------ | ------ |
| `FE-ACCOUNT-DETAIL-IMPL-023` | Semantic page structure is implemented.   | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-024` | Primary `h1` is implemented.              | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-025` | Navigation is keyboard accessible.        | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-026` | Visible focus is implemented.             | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-027` | Loading and error states are accessible.  | ⬜     |        |
| `FE-ACCOUNT-DETAIL-IMPL-028` | Status is not represented by color alone. | ⬜     |        |
