# Frontend Admin Account List

## Table of Contents

1. [Scope](#scope)
2. [Route](#route)
3. [Requirements](#requirements)
4. [Layout](#layout)
5. [API Contract](#api-contract)
6. [Account Actions](#account-actions)
7. [UI States](#ui-states)
8. [Security](#security)
9. [Accessibility](#accessibility)
10. [Testing](#testing)
11. [Implementation Criteria](#implementation-criteria)
12. [Contract Notes](#contract-notes)

---

# 1. Scope

This document defines the **Administrator Account List** at `/admin/accounts`.

The page is rendered inside the existing [Admin Shell](./admin_shell.md).

### Responsibilities

* Display administrator accounts.
* Filter accounts by status.
* Optionally include soft-deleted accounts.
* Paginate server-provided results.
* Navigate to Create Account and Edit Account.
* Support account activation, deactivation, and restoration where permitted.
* Handle loading, empty, error, authentication, authorization, and conflict states.
* Preserve the Admin Shell and its navigation behavior.

### Out of Scope

* Authentication implementation.
* Authorization implementation.
* Role management.
* Password hashing or credential validation.
* Account persistence.
* Database transactions and lifecycle invariants.
* Create Account form implementation.
* Edit Account form implementation.
* Hard-delete UI.

---

# 2. Route

| ID                    | Route                      | Access          | Behavior             |
| --------------------- | -------------------------- | --------------- | -------------------- |
| `FE_ACCOUNT_ROUTE_01` | `/admin/accounts`          | Authenticated   | Render Account List  |
| `FE_ACCOUNT_ROUTE_02` | `/admin/accounts`          | Unauthenticated | Redirect to `/login` |
| `FE_ACCOUNT_ROUTE_03` | `/admin/accounts/create`   | Authenticated   | Create Account page  |
| `FE_ACCOUNT_ROUTE_04` | `/admin/accounts/:id/edit` | Authenticated   | Edit Account page    |

The page must render inside the Admin Shell:

```text
/admin/accounts
    └── Admin Shell
        ├── Sidebar
        │   └── Accounts = active
        └── Main Content
            └── Account List
```

---

# 3. Requirements

| ID              | Requirement                                                                        |
| --------------- | ---------------------------------------------------------------------------------- |
| `FE_ACCOUNT_01` | Provide a protected `/admin/accounts` route.                                       |
| `FE_ACCOUNT_02` | Render the Account List inside the Admin Shell Main Content Area.                  |
| `FE_ACCOUNT_03` | Mark Accounts navigation as active.                                                |
| `FE_ACCOUNT_04` | Load accounts from `GET /admin/accounts`.                                          |
| `FE_ACCOUNT_05` | Exclude soft-deleted accounts by default.                                          |
| `FE_ACCOUNT_06` | Support `page` pagination.                                                         |
| `FE_ACCOUNT_07` | Support `page_size` pagination.                                                    |
| `FE_ACCOUNT_08` | Support `status=active`.                                                           |
| `FE_ACCOUNT_09` | Support `status=inactive`.                                                         |
| `FE_ACCOUNT_10` | Support `include_deleted=true` for restoration workflows.                          |
| `FE_ACCOUNT_11` | Display account name (`display_name`).                                             |
| `FE_ACCOUNT_12` | Display email.                                                                     |
| `FE_ACCOUNT_13` | Display account state.                                                             |
| `FE_ACCOUNT_14` | Display `created_at`.                                                              |
| `FE_ACCOUNT_15` | Display `updated_at`.                                                              |
| `FE_ACCOUNT_16` | Provide navigation to `/admin/accounts/create`.                                    |
| `FE_ACCOUNT_17` | Provide navigation to `/admin/accounts/:id/edit` for non-deleted accounts.         |
| `FE_ACCOUNT_18` | Support activation and deactivation actions where authorized.                      |
| `FE_ACCOUNT_19` | Support restoration of soft-deleted accounts when deleted accounts are requested.  |
| `FE_ACCOUNT_20` | Treat `401 Unauthorized` as an invalid authentication state.                       |
| `FE_ACCOUNT_21` | Treat `403 Forbidden` as an authorization failure.                                 |
| `FE_ACCOUNT_22` | Refresh authoritative account data after successful mutations.                     |
| `FE_ACCOUNT_23` | Prevent duplicate submissions for the same mutation.                               |
| `FE_ACCOUNT_24` | Preserve list state when returning from Edit Account.                              |
| `FE_ACCOUNT_25` | Do not introduce client-side sorting unsupported by the API.                       |
| `FE_ACCOUNT_26` | Do not introduce search or filtering unsupported by the API.                       |
| `FE_ACCOUNT_27` | Keep access tokens and passwords out of UI, URLs, logs, and persisted client data. |
| `FE_ACCOUNT_28` | Keep backend authentication and authorization authoritative.                       |
| `FE_ACCOUNT_29` | Provide keyboard-accessible interaction and visible focus.                         |
| `FE_ACCOUNT_30` | Provide accessible loading, success, and error feedback.                           |

---

# 4. Layout

## Page Header

```text
Administrator Accounts
Manage administrator accounts and access.

[Create Account]
```

Requirements:

| ID                 | Requirement                                       |
| ------------------ | ------------------------------------------------- |
| `FE_ACCOUNT_UI_01` | Use `Administrator Accounts` as the page heading. |
| `FE_ACCOUNT_UI_02` | Provide a short supporting description.           |
| `FE_ACCOUNT_UI_03` | Provide a Create Account action.                  |

Create Account links to:

```text
/admin/accounts/create
```

## List Controls

```text
Status        [All ▼]
Include deleted [ ]
Page size     [20 ▼]
              [Refresh]
```

Supported controls:

| Control         | Behavior                           |
| --------------- | ---------------------------------- |
| Status          | `All`, `Active`, `Inactive`        |
| Include deleted | Sends `include_deleted=true`       |
| Page size       | Must not exceed `100`              |
| Refresh         | Re-fetches the current query state |

Do not add search or sort controls.

### URL State

The current list state may be represented in the query string:

```text
/admin/accounts
/admin/accounts?page=2&page_size=20
/admin/accounts?page=2&page_size=20&status=active
/admin/accounts?page=2&page_size=20&status=inactive&include_deleted=true
```

Changing a filter or page size resets `page` to `1`.

---

## Account Table

Use a native HTML table.

| Column  | Source                 | Display                                |
| ------- | ---------------------- | -------------------------------------- |
| Name    | `display_name`         | Name or `—`                            |
| Email   | `email`                | Email address                          |
| Status  | `status`, `deleted_at` | Active / Inactive / Deleted            |
| Created | `created_at`           | Localized date/time                    |
| Updated | `updated_at`           | Localized date/time                    |
| Actions | Derived                | Edit / Activate / Deactivate / Restore |

The table must not be implemented as an ARIA `grid` unless the interaction model later requires grid-specific keyboard behavior.

### Status Mapping

```text
status = active   && deleted_at = null → Active
status = inactive && deleted_at = null → Inactive
deleted_at != null                    → Deleted
```

`Deleted` is a UI state derived from `deleted_at`; it is not an Account status value.

### Row Actions

| Account State         | Actions          |
| --------------------- | ---------------- |
| Active, non-deleted   | Edit, Deactivate |
| Inactive, non-deleted | Edit, Activate   |
| Soft-deleted          | Restore          |

Do not expose Hard Delete as a routine row action.

### Account Count

The API returns:

```json
{
  "items": [],
  "page": 1,
  "page_size": 20,
  "total": 0
}
```

Display:

```text
0 accounts
```

when `total = 0`.

Otherwise:

```text
{start}–{end} of {total} accounts
```

where:

```text
start = ((page - 1) * page_size) + 1
end   = min(page * page_size, total)
```

---

## Pagination

Use the server-side `page` and `page_size` contract.

Rules:

* First page is `1`.
* Previous is disabled on page `1`.
* Next is disabled on the last page.
* Filter changes reset to page `1`.
* Page-size changes reset to page `1`.
* Client-side pagination is not permitted.
* The frontend must preserve the server-provided row order.

The backend defines deterministic ordering as:

```text
id ASC
```

The frontend must not re-sort the response.

---

## Responsive Layout

The Account List follows the Admin Shell breakpoints:

| Viewport       | Layout                                       |
| -------------- | -------------------------------------------- |
| `≥ 1280px`     | Full table                                   |
| `768px–1279px` | Full table with local horizontal overflow    |
| `< 768px`      | Compact table with local horizontal overflow |

Horizontal scrolling must be contained within the table region. The page itself must not acquire unintended horizontal scrolling.

The following remain unchanged across viewport sizes:

* Authentication behavior.
* Authorization behavior.
* API contract.
* Account state semantics.

---

# 5. API Contract

## List Request

```http
GET /admin/accounts
Authorization: Bearer <access-token>
```

Supported query parameters:

| Parameter         | Default | Rule                            |
| ----------------- | ------- | ------------------------------- |
| `page`            | `1`     | Must be `>= 1`                  |
| `page_size`       | `20`    | Maximum `100`                   |
| `status`          | omitted | `active` or `inactive`          |
| `include_deleted` | `false` | Explicitly enable when required |

## Response

```json
{
  "items": [
    {
      "id": "019...",
      "email": "admin@example.com",
      "display_name": "Library Administrator",
      "status": "active",
      "created_at": "2026-09-23T10:00:00Z",
      "updated_at": "2026-09-23T10:00:00Z",
      "deleted_at": null
    }
  ],
  "page": 1,
  "page_size": 20,
  "total": 1
}
```

The frontend must:

* Ignore unknown response fields.
* Never expect `password` or `password_hash`.
* Use the response as the source of truth.
* Preserve the response order.

## Authorization

For:

```text
include_deleted = false
```

the request requires:

```text
account:view
```

For:

```text
include_deleted = true
```

the request requires:

```text
account:view
account:view_deleted
```

Authorization is evaluated by the backend.

The frontend must not determine authorization from:

* Client-supplied account IDs.
* Client-supplied roles.
* Client-supplied permissions.
* Hidden UI controls.

## Response Handling

### `200 OK`

Replace the current list and pagination state with the response.

### `401 Unauthorized`

1. Clear client authentication state.
2. Navigate to `/login`.
3. Do not retry indefinitely.

### `403 Forbidden`

1. Preserve authentication state.
2. Do not navigate to `/login`.
3. Display an authorization error.
4. Do not treat the request as an empty result.

When `include_deleted=true` is rejected with `403`, reset the filter to `false` and reload the normal list.

### `404 Not Found`

For a row mutation:

1. Display a non-blocking message.
2. Refresh the list.
3. Remove the stale row if no longer returned.

### `409 Conflict`

Refresh the list and display the domain conflict.

For:

```text
LAST_ACTIVE_ADMINISTRATOR
```

display:

```text
This action cannot be completed because it would leave the system without an active administrator.
```

### `422 Unprocessable Content`

Display the server-provided validation result without exposing implementation details.

### `5xx`

Display:

```text
We could not complete the request.
Please try again.
```

Do not expose stack traces or infrastructure details.

### Network Failure

Keep the current rendered data and provide a retry action.

---

## Problem Details

Account API errors use:

```text
application/problem+json
```

The frontend should use the stable `code` field when supplied rather than matching free-form error text.

Relevant codes include:

```text
ACCOUNT_NOT_FOUND
ACCOUNT_ALREADY_ACTIVE
ACCOUNT_ALREADY_INACTIVE
ACCOUNT_ALREADY_DELETED
ACCOUNT_NOT_DELETED
LAST_ACTIVE_ADMINISTRATOR
VALIDATION_ERROR
```

## Cache

Account API responses are defined as:

```http
Cache-Control: no-store
```

The frontend must not persist Account List data in:

* `localStorage`
* `sessionStorage`
* IndexedDB
* Other persistent application storage

The frontend must not use cached account data as authorization evidence.

---

# 6. Account Actions

## Edit

Navigate to:

```text
/admin/accounts/:id/edit
```

Edit is available only for non-deleted accounts.

The Account List does not implement edit fields.

When returning to the list, preserve the previous list state where possible:

```text
/admin/accounts?page=2&page_size=20&status=active
```

## Deactivate

Endpoint:

```http
POST /admin/accounts/{id}/deactivate
```

Available when:

```text
status = active
deleted_at = null
```

The action should require confirmation.

After `200 OK`:

```text
GET /admin/accounts
```

to refresh authoritative state.

If the backend returns:

```text
409 LAST_ACTIVE_ADMINISTRATOR
```

show the conflict and refresh the list.

## Activate

Endpoint:

```http
POST /admin/accounts/{id}/activate
```

Available when:

```text
status = inactive
deleted_at = null
```

After success, refresh the list.

## Restore

Endpoint:

```http
POST /admin/accounts/{id}/restore
```

Available only for soft-deleted accounts.

After successful restoration, the backend returns the Account as:

```text
status = inactive
deleted_at = null
```

The frontend must display the restored account as `Inactive`.

## Soft Delete

The Account domain supports:

```http
DELETE /admin/accounts/{id}
```

but the current frontend requirements do not require Soft Delete as a primary Account List action.

Do not expose it unless the product scope explicitly enables it.

## Hard Delete

The Account domain supports:

```http
DELETE /admin/accounts/{id}/purge
```

Do not expose Hard Delete as a routine Account List action.

It is a destructive cross-domain operation and requires a separate product decision.

## Mutation Behavior

For all row mutations:

```text
User action
    ↓
Disable initiating action
    ↓
Send request
    ↓
Wait for server response
    ↓
Success → Refresh list
Failure → Keep server-derived state and show error
```

Do not optimistically change account state.

Prevent duplicate submission for the same action.

---

# 7. UI States

## Loading

The shell, page heading, and controls remain visible.

The list area shows a loading state without unnecessary layout shift.

For refreshes and filter changes, preserve the existing page structure while the new response is pending.

## Empty

### No Accounts

```text
No administrator accounts

There are no administrator accounts to display.

[Create Account]
```

### Filtered Empty

```text
No matching administrator accounts

No accounts match the current filters.

[Clear Filters]
```

### No Deleted Accounts

```text
No deleted administrator accounts

There are no soft-deleted accounts to restore.
```

## Error

```text
Unable to load administrator accounts

The account list could not be loaded.

[Try Again]
```

## Permission Error

```text
You do not have permission to view administrator accounts.
```

Do not expose internal permission identifiers.

## Authentication Expired

When the API returns `401`:

```text
Your session is no longer valid.
```

Then navigate to:

```text
/login
```

## Mutation Pending

Disable the initiating action while its request is pending.

## Mutation Success

Use an accessible status message such as:

```text
Account updated.
```

Do not move focus unnecessarily.

## Feature Error

Feature errors remain inside the Main Content Area.

The Admin Shell remains rendered.

---

# 8. Security

| ID                  | Requirement                                                                |
| ------------------- | -------------------------------------------------------------------------- |
| `FE_ACCOUNT_SEC_01` | `/admin/accounts` requires authentication.                                 |
| `FE_ACCOUNT_SEC_02` | Backend Authentication is authoritative.                                   |
| `FE_ACCOUNT_SEC_03` | Backend Authorization is authoritative.                                    |
| `FE_ACCOUNT_SEC_04` | Bearer credentials are sent through the `Authorization` header.            |
| `FE_ACCOUNT_SEC_05` | Tokens never appear in URLs.                                               |
| `FE_ACCOUNT_SEC_06` | Tokens never appear in rendered UI.                                        |
| `FE_ACCOUNT_SEC_07` | Tokens never appear in application logs.                                   |
| `FE_ACCOUNT_SEC_08` | Passwords and password hashes are never rendered or stored by the feature. |
| `FE_ACCOUNT_SEC_09` | Client-side state is not treated as proof of authorization.                |
| `FE_ACCOUNT_SEC_10` | UI visibility is not a security boundary.                                  |
| `FE_ACCOUNT_SEC_11` | Mutations are only considered successful after server confirmation.        |
| `FE_ACCOUNT_SEC_12` | Account data is not persisted in browser storage.                          |
| `FE_ACCOUNT_SEC_13` | Backend internal errors are not exposed to users.                          |

Request pipeline:

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
Account Handler
```

---

# 9. Accessibility

The Account List must follow the Admin Shell accessibility requirements and WCAG 2.2-oriented implementation.

| ID                   | Requirement                                              |
| -------------------- | -------------------------------------------------------- |
| `FE_ACCOUNT_A11Y_01` | Use semantic page structure.                             |
| `FE_ACCOUNT_A11Y_02` | Provide one primary `h1`.                                |
| `FE_ACCOUNT_A11Y_03` | Use a native `table` for tabular data.                   |
| `FE_ACCOUNT_A11Y_04` | Provide semantic table headers.                          |
| `FE_ACCOUNT_A11Y_05` | Provide an accessible table name or caption.             |
| `FE_ACCOUNT_A11Y_06` | Use real links for navigation.                           |
| `FE_ACCOUNT_A11Y_07` | Use real buttons for actions.                            |
| `FE_ACCOUNT_A11Y_08` | Provide visible focus indicators.                        |
| `FE_ACCOUNT_A11Y_09` | Ensure focused controls are not obscured.                |
| `FE_ACCOUNT_A11Y_10` | Provide accessible labels for all filters.               |
| `FE_ACCOUNT_A11Y_11` | Provide descriptive accessible names for row actions.    |
| `FE_ACCOUNT_A11Y_12` | Do not communicate status through color alone.           |
| `FE_ACCOUNT_A11Y_13` | Announce relevant loading, success, and error states.    |
| `FE_ACCOUNT_A11Y_14` | Keep pagination keyboard accessible.                     |
| `FE_ACCOUNT_A11Y_15` | Keep keyboard order predictable.                         |
| `FE_ACCOUNT_A11Y_16` | Ensure dialogs, when used, manage focus correctly.       |
| `FE_ACCOUNT_A11Y_17` | Keep the feature usable across supported viewport sizes. |

### Row Action Naming

Actions should identify their target account.

Examples:

```text
Edit account: Library Administrator
Deactivate account: Library Administrator
Activate account: Editor Account
Restore account: Former Administrator
```

### Status

Always provide text:

```text
Active
Inactive
Deleted
```

Color may supplement the text but must not carry the meaning alone.

---

# 10. Testing

## Unit

| ID                        | Test                                                   |
| ------------------------- | ------------------------------------------------------ |
| `FE_ACCOUNT_TEST_UNIT_01` | Account List renders.                                  |
| `FE_ACCOUNT_TEST_UNIT_02` | Table columns render correctly.                        |
| `FE_ACCOUNT_TEST_UNIT_03` | Active, inactive, and deleted states render correctly. |
| `FE_ACCOUNT_TEST_UNIT_04` | Null `display_name` renders safely.                    |
| `FE_ACCOUNT_TEST_UNIT_05` | Pagination calculations are correct.                   |
| `FE_ACCOUNT_TEST_UNIT_06` | Loading, empty, and error states render correctly.     |
| `FE_ACCOUNT_TEST_UNIT_07` | 401 and 403 handling are distinct.                     |
| `FE_ACCOUNT_TEST_UNIT_08` | Mutation buttons prevent duplicate submission.         |
| `FE_ACCOUNT_TEST_UNIT_09` | Mutation success triggers a refresh.                   |
| `FE_ACCOUNT_TEST_UNIT_10` | Accessible action names are generated correctly.       |

## Integration

| ID                       | Test                                                          |
| ------------------------ | ------------------------------------------------------------- |
| `FE_ACCOUNT_TEST_INT_01` | Authenticated user can open `/admin/accounts`.                |
| `FE_ACCOUNT_TEST_INT_02` | Unauthenticated user is redirected to `/login`.               |
| `FE_ACCOUNT_TEST_INT_03` | Accounts navigation is active.                                |
| `FE_ACCOUNT_TEST_INT_04` | `GET /admin/accounts` uses the expected query parameters.     |
| `FE_ACCOUNT_TEST_INT_05` | `include_deleted=false` is the default.                       |
| `FE_ACCOUNT_TEST_INT_06` | Status filtering works.                                       |
| `FE_ACCOUNT_TEST_INT_07` | Pagination works.                                             |
| `FE_ACCOUNT_TEST_INT_08` | Create and Edit navigation uses the correct routes.           |
| `FE_ACCOUNT_TEST_INT_09` | Activate, Deactivate, and Restore call the correct endpoints. |
| `FE_ACCOUNT_TEST_INT_10` | Mutation success refreshes authoritative data.                |
| `FE_ACCOUNT_TEST_INT_11` | 401 clears authentication and navigates to `/login`.          |
| `FE_ACCOUNT_TEST_INT_12` | 403 preserves authentication state.                           |
| `FE_ACCOUNT_TEST_INT_13` | 409 lifecycle conflicts are handled correctly.                |
| `FE_ACCOUNT_TEST_INT_14` | Problem Details `code` values are handled correctly.          |
| `FE_ACCOUNT_TEST_INT_15` | Tokens are never added to URLs or persistent storage.         |

## E2E

| ID                       | Test                                                   |
| ------------------------ | ------------------------------------------------------ |
| `FE_ACCOUNT_TEST_E2E_01` | Login → Admin Shell → Accounts works.                  |
| `FE_ACCOUNT_TEST_E2E_02` | Account rows render.                                   |
| `FE_ACCOUNT_TEST_E2E_03` | Filters and pagination work.                           |
| `FE_ACCOUNT_TEST_E2E_04` | Create navigation works.                               |
| `FE_ACCOUNT_TEST_E2E_05` | Edit navigation works.                                 |
| `FE_ACCOUNT_TEST_E2E_06` | Deactivate works.                                      |
| `FE_ACCOUNT_TEST_E2E_07` | Activate works.                                        |
| `FE_ACCOUNT_TEST_E2E_08` | Restore works when authorized.                         |
| `FE_ACCOUNT_TEST_E2E_09` | Last-active-administrator conflict is shown correctly. |
| `FE_ACCOUNT_TEST_E2E_10` | Authentication expiry redirects to `/login`.           |
| `FE_ACCOUNT_TEST_E2E_11` | Mobile layout remains usable.                          |
| `FE_ACCOUNT_TEST_E2E_12` | Keyboard interaction works.                            |

## Accessibility

| ID                        | Test                                                    |
| ------------------------- | ------------------------------------------------------- |
| `FE_ACCOUNT_TEST_A11Y_01` | Heading hierarchy is correct.                           |
| `FE_ACCOUNT_TEST_A11Y_02` | Table headers are programmatically associated.          |
| `FE_ACCOUNT_TEST_A11Y_03` | All controls are keyboard accessible.                   |
| `FE_ACCOUNT_TEST_A11Y_04` | Focus is visible.                                       |
| `FE_ACCOUNT_TEST_A11Y_05` | Status does not rely on color alone.                    |
| `FE_ACCOUNT_TEST_A11Y_06` | Async messages are accessible.                          |
| `FE_ACCOUNT_TEST_A11Y_07` | Pagination is accessible.                               |
| `FE_ACCOUNT_TEST_A11Y_08` | Dialog focus behavior is correct when dialogs are used. |

---

# 11. Implementation Criteria

## Route

| ID                    | Criteria                                     | Status         |
| --------------------- | -------------------------------------------- | -------------- |
| `FE_ACCOUNT_ROUTE_01` | `/admin/accounts` renders inside Admin Shell | 🟡 In Progress |
| `FE_ACCOUNT_ROUTE_02` | Unauthenticated access redirects to `/login` | 🟢 Implemented |
| `FE_ACCOUNT_ROUTE_03` | Accounts navigation is active                | 🟡 In Progress |

## Data

| ID                  | Criteria                                             | Status        |
| ------------------- | ---------------------------------------------------- | ------------- |
| `FE_ACCOUNT_API_01` | List uses `GET /admin/accounts`                      | ⚪ Not Started |
| `FE_ACCOUNT_API_02` | Bearer token uses `Authorization` header             | ⚪ Not Started |
| `FE_ACCOUNT_API_03` | Deleted accounts are excluded by default             | ⚪ Not Started |
| `FE_ACCOUNT_API_04` | Pagination uses `page` and `page_size`               | ⚪ Not Started |
| `FE_ACCOUNT_API_05` | `page_size` never exceeds `100`                      | ⚪ Not Started |
| `FE_ACCOUNT_API_06` | Status filter maps to API values                     | ⚪ Not Started |
| `FE_ACCOUNT_API_07` | `include_deleted` behavior matches API authorization | ⚪ Not Started |
| `FE_ACCOUNT_API_08` | Server response is authoritative                     | ⚪ Not Started |
| `FE_ACCOUNT_API_09` | Problem Details are handled                          | ⚪ Not Started |

## UI

| ID                 | Criteria                      | Status        |
| ------------------ | ----------------------------- | ------------- |
| `FE_ACCOUNT_UI_01` | Page header implemented       | ⚪ Not Started |
| `FE_ACCOUNT_UI_02` | Filters implemented           | ⚪ Not Started |
| `FE_ACCOUNT_UI_03` | Account table implemented     | ⚪ Not Started |
| `FE_ACCOUNT_UI_04` | Pagination implemented        | ⚪ Not Started |
| `FE_ACCOUNT_UI_05` | Loading state implemented     | ⚪ Not Started |
| `FE_ACCOUNT_UI_06` | Empty state implemented       | ⚪ Not Started |
| `FE_ACCOUNT_UI_07` | Error state implemented       | ⚪ Not Started |
| `FE_ACCOUNT_UI_08` | Permission state implemented  | ⚪ Not Started |
| `FE_ACCOUNT_UI_09` | Responsive layout implemented | ⚪ Not Started |

## Actions

| ID                     | Criteria                           | Status        |
| ---------------------- | ---------------------------------- | ------------- |
| `FE_ACCOUNT_ACTION_01` | Edit navigation implemented        | ⚪ Not Started |
| `FE_ACCOUNT_ACTION_02` | Activate implemented               | ⚪ Not Started |
| `FE_ACCOUNT_ACTION_03` | Deactivate implemented             | ⚪ Not Started |
| `FE_ACCOUNT_ACTION_04` | Restore implemented                | ⚪ Not Started |
| `FE_ACCOUNT_ACTION_05` | Duplicate submission prevented     | ⚪ Not Started |
| `FE_ACCOUNT_ACTION_06` | Successful mutation refreshes list | ⚪ Not Started |
| `FE_ACCOUNT_ACTION_07` | Hard Delete not exposed            | ⚪ Not Started |

## Security

| ID                  | Criteria                                  | Status         |
| ------------------- | ----------------------------------------- | -------------- |
| `FE_ACCOUNT_SEC_01` | Backend remains authentication authority  | 🟢 Implemented |
| `FE_ACCOUNT_SEC_02` | Backend remains authorization authority   | 🟢 Implemented |
| `FE_ACCOUNT_SEC_03` | Tokens never appear in URLs               | ⚪ Not Started  |
| `FE_ACCOUNT_SEC_04` | Tokens never appear in logs               | ⚪ Not Started  |
| `FE_ACCOUNT_SEC_05` | Passwords are never rendered              | ⚪ Not Started  |
| `FE_ACCOUNT_SEC_06` | Account data is not persisted client-side | ⚪ Not Started  |

## Accessibility

| ID                   | Criteria                          | Status        |
| -------------------- | --------------------------------- | ------------- |
| `FE_ACCOUNT_A11Y_01` | Semantic layout implemented       | ⚪ Not Started |
| `FE_ACCOUNT_A11Y_02` | Native table implemented          | ⚪ Not Started |
| `FE_ACCOUNT_A11Y_03` | Visible focus implemented         | ⚪ Not Started |
| `FE_ACCOUNT_A11Y_04` | Keyboard interaction verified     | ⚪ Not Started |
| `FE_ACCOUNT_A11Y_05` | Async status messaging verified   | ⚪ Not Started |
| `FE_ACCOUNT_A11Y_06` | Responsive accessibility verified | ⚪ Not Started |

---

# 12. Contract Notes

## `display_name`

The current repository documents contain two representations:

* The project requirements define administrator **Name** as required.
* The Account domain defines `display_name` as optional.
* The Account List response example includes `display_name`.
* The generic Account Response example currently omits `display_name`.

Frontend implementation should use:

```text
display_name
```

as the source for the displayed Name field.

The API contract should be aligned so that the list response and Account response define `display_name` consistently.

## Deleted Accounts

The Account API explicitly supports:

```text
include_deleted=true
```

and requires:

```text
account:view
account:view_deleted
```

for that request.

The frontend must not infer permission availability from the current user's visible UI state.

## Account Actions

The project requirements explicitly require administrator account:

* View
* Create
* Edit
* Deactivate

The Account domain additionally defines:

* Activate
* Soft Delete
* Restore
* Hard Delete
* Change Email
* Change Password

The Account List exposes only the actions defined by this document. Domain support alone does not imply that an operation must appear in the UI.

## Traceability

| Area           | References                                                                       |
| -------------- | -------------------------------------------------------------------------------- |
| Route          | `ADM-AUTH-001`, `ADM-AUTH-002`                                                   |
| Account List   | `ADM-AUTH-004`, `AC_UC_02`, `AC_API_02`                                          |
| Create         | `ADM-AUTH-003`, `AC_UC_01`, `AC_API_01`                                          |
| Edit           | `ADM-AUTH-005`, `AC_UC_04`, `AC_API_04`                                          |
| Deactivate     | `ADM-AUTH-006`, `ADM-AUTH-007`, `AC_UC_05`, `AC_API_05`                          |
| Activate       | `AC_UC_06`, `AC_API_06`                                                          |
| Restore        | `AC_UC_08`, `AC_API_08`                                                          |
| Authentication | Authentication domain protected-request contract                                 |
| Authorization  | `account:view`, `account:view_deleted`, `account:activate`, `account:deactivate` |
| Shell          | `admin_shell.md`                                                                 |

The requirement-to-design-to-implementation-to-test chain must remain traceable throughout development:

```text
Requirement
    ↓
Frontend Design
    ↓
Implementation
    ↓
Test
```
