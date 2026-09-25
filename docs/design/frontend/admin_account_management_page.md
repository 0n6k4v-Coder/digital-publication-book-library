# Frontend Admin Account Management Page

## Table of Contents

1. [Scope](#scope)
2. [Routes](#routes)
3. [Requirements](#requirements)
4. [Layout](#layout)
5. [API Contract](#api-contract)
6. [Account Actions](#account-actions)
7. [UI States](#ui-states)
8. [Security](#security)
9. [Accessibility](#accessibility)
10. [Testing](#testing)
11. [Implementation Criteria](#implementation-criteria)
12. [Traceability](#traceability)

---

# 1. Scope

This document defines the protected **Administrator Account Management** frontend feature.

The primary page is:

```text
/admin/accounts
```

The feature is rendered inside the existing [Admin Shell](./admin_shell.md).

## Responsibilities

| ID                     | Responsibility                                                                    |
| ---------------------- | --------------------------------------------------------------------------------- |
| `FE-ACCOUNT-SCOPE-001` | Display administrator accounts.                                                   |
| `FE-ACCOUNT-SCOPE-002` | Filter accounts by supported status values.                                       |
| `FE-ACCOUNT-SCOPE-003` | Optionally include soft-deleted accounts.                                         |
| `FE-ACCOUNT-SCOPE-004` | Paginate server-provided results.                                                 |
| `FE-ACCOUNT-SCOPE-005` | Navigate to Create Account.                                                       |
| `FE-ACCOUNT-SCOPE-006` | Navigate to Edit Account.                                                         |
| `FE-ACCOUNT-SCOPE-007` | Activate administrator accounts.                                                  |
| `FE-ACCOUNT-SCOPE-008` | Deactivate administrator accounts.                                                |
| `FE-ACCOUNT-SCOPE-009` | Restore soft-deleted administrator accounts.                                      |
| `FE-ACCOUNT-SCOPE-010` | Handle loading, empty, error, authentication, authorization, and conflict states. |
| `FE-ACCOUNT-SCOPE-011` | Preserve Admin Shell behavior and navigation state.                               |

## Out of Scope

| ID                   | Excluded Area                                                |
| -------------------- | ------------------------------------------------------------ |
| `FE-ACCOUNT-OOS-001` | Authentication implementation.                               |
| `FE-ACCOUNT-OOS-002` | Authorization implementation.                                |
| `FE-ACCOUNT-OOS-003` | Role management.                                             |
| `FE-ACCOUNT-OOS-004` | Password hashing and credential validation.                  |
| `FE-ACCOUNT-OOS-005` | Account persistence and database transactions.               |
| `FE-ACCOUNT-OOS-006` | Account lifecycle invariant enforcement.                     |
| `FE-ACCOUNT-OOS-007` | Create Account form implementation.                          |
| `FE-ACCOUNT-OOS-008` | Edit Account form implementation.                            |
| `FE-ACCOUNT-OOS-009` | Hard-delete UI.                                              |
| `FE-ACCOUNT-OOS-010` | Complex search and filtering not defined by the Account API. |

---

# 2. Routes

| ID                     | Route                      | Access          | Behavior                        | References                                     |
| ---------------------- | -------------------------- | --------------- | ------------------------------- | ---------------------------------------------- |
| `FE-ACCOUNT-ROUTE-001` | `/admin/accounts`          | Authenticated   | Render Account Management page. | `ADM-AUTH-001`, `ADM-AUTH-002`, `PAGE-ADM-008` |
| `FE-ACCOUNT-ROUTE-002` | `/admin/accounts`          | Unauthenticated | Redirect to `/login`.           | `FE_SHELL_ROUTE_05`, Authentication domain     |
| `FE-ACCOUNT-ROUTE-003` | `/admin/accounts/create`   | Authenticated   | Render Create Account page.     | `ADM-AUTH-003`, `PAGE-ADM-009`                 |
| `FE-ACCOUNT-ROUTE-004` | `/admin/accounts/:id/edit` | Authenticated   | Render Edit Account page.       | `ADM-AUTH-005`, `PAGE-ADM-010`                 |

The page must render inside the Admin Shell:

```text
/admin/accounts
    └── Admin Shell
        ├── Sidebar
        │   └── Accounts = active
        └── Main Content
            └── Account Management
```

## Query State

The list state may be represented in the URL:

```text
/admin/accounts
/admin/accounts?page=2
/admin/accounts?page=2&page_size=50
/admin/accounts?page=2&page_size=50&status=active
/admin/accounts?page=2&page_size=50&status=inactive&include_deleted=true
```

| ID                     | Rule                                                              |
| ---------------------- | ----------------------------------------------------------------- |
| `FE-ACCOUNT-ROUTE-005` | `page` defaults to `1`.                                           |
| `FE-ACCOUNT-ROUTE-006` | `page_size` defaults to `20`.                                     |
| `FE-ACCOUNT-ROUTE-007` | `status` may be `active` or `inactive`.                           |
| `FE-ACCOUNT-ROUTE-008` | `include_deleted` defaults to `false`.                            |
| `FE-ACCOUNT-ROUTE-009` | Changing filters resets `page` to `1`.                            |
| `FE-ACCOUNT-ROUTE-010` | Changing `page_size` resets `page` to `1`.                        |
| `FE-ACCOUNT-ROUTE-011` | Access tokens and other credentials must never appear in the URL. |

---

# 3. Requirements

| ID                   | Requirement                                                    | Repository Reference                                       |
| -------------------- | -------------------------------------------------------------- | ---------------------------------------------------------- |
| `FE-ACCOUNT-REQ-001` | Provide a protected `/admin/accounts` route.                   | `ADM-AUTH-001`, `PAGE-ADM-008`                             |
| `FE-ACCOUNT-REQ-002` | Display administrator accounts.                                | `ADM-AUTH-004`, `AC_UC_02`, `AC_API_02`                    |
| `FE-ACCOUNT-REQ-003` | Support Create Account navigation.                             | `ADM-AUTH-003`, `AC_UC_01`, `AC_API_01`                    |
| `FE-ACCOUNT-REQ-004` | Support Edit Account navigation.                               | `ADM-AUTH-005`, `AC_UC_04`, `AC_API_04`                    |
| `FE-ACCOUNT-REQ-005` | Support administrator deactivation.                            | `ADM-AUTH-006`, `ADM-AUTH-007`, `AC_UC_05`, `AC_API_05`    |
| `FE-ACCOUNT-REQ-006` | Support administrator activation.                              | `AC_UC_06`, `AC_API_06`                                    |
| `FE-ACCOUNT-REQ-007` | Support soft-deleted account restoration.                      | `AC_UC_08`, `AC_API_08`                                    |
| `FE-ACCOUNT-REQ-008` | Exclude soft-deleted accounts by default.                      | `AC_UC_02`, `AC_API_02`                                    |
| `FE-ACCOUNT-REQ-009` | Support `status=active`.                                       | `AC_API_02`                                                |
| `FE-ACCOUNT-REQ-010` | Support `status=inactive`.                                     | `AC_API_02`                                                |
| `FE-ACCOUNT-REQ-011` | Support `include_deleted=true` when authorized.                | `AC_API_02`, `account:view_deleted`                        |
| `FE-ACCOUNT-REQ-012` | Support server-side pagination.                                | `AC_API_02`                                                |
| `FE-ACCOUNT-REQ-013` | Preserve backend-defined ordering.                             | `AC_API_02`                                                |
| `FE-ACCOUNT-REQ-014` | Treat backend Authentication as authoritative.                 | Authentication domain, `FE_SHELL_SEC_03`                   |
| `FE-ACCOUNT-REQ-015` | Treat backend Authorization as authoritative.                  | Authorization domain, `FE_SHELL_SEC_09`, `FE_SHELL_SEC_10` |
| `FE-ACCOUNT-REQ-016` | Handle `401 Unauthorized` as invalid authentication.           | Authentication domain                                      |
| `FE-ACCOUNT-REQ-017` | Handle `403 Forbidden` as authorization failure.               | Authorization domain                                       |
| `FE-ACCOUNT-REQ-018` | Refresh authoritative account data after successful mutations. | Account domain                                             |
| `FE-ACCOUNT-REQ-019` | Prevent duplicate mutation submissions.                        | Frontend behavior                                          |
| `FE-ACCOUNT-REQ-020` | Preserve list state when returning from Edit Account.          | Frontend behavior                                          |
| `FE-ACCOUNT-REQ-021` | Do not introduce unsupported sorting.                          | Account API                                                |
| `FE-ACCOUNT-REQ-022` | Do not introduce unsupported search/filtering.                 | `OUT-012`, Account API                                     |
| `FE-ACCOUNT-REQ-023` | Do not expose credentials or tokens in the UI.                 | Account Security, Authentication Security                  |
| `FE-ACCOUNT-REQ-024` | Do not persist Account List data in client storage.            | Account API `no-store` contract                            |
| `FE-ACCOUNT-REQ-025` | Provide keyboard-accessible interaction.                       | Admin Shell accessibility                                  |
| `FE-ACCOUNT-REQ-026` | Provide visible focus states.                                  | Admin Shell accessibility                                  |
| `FE-ACCOUNT-REQ-027` | Provide accessible asynchronous status and error feedback.     | WCAG-oriented implementation                               |
| `FE-ACCOUNT-REQ-028` | Prevent unintended page-level horizontal scrolling.            | Admin Shell responsive requirements                        |

---

# 4. Layout

## 4.1 Page Header

```text
Administrator Accounts
Manage administrator accounts and access.

[Create Account]
```

| ID                  | Requirement                                               | References           |
| ------------------- | --------------------------------------------------------- | -------------------- |
| `FE-ACCOUNT-UI-001` | Use `Administrator Accounts` as the primary page heading. | `FE-ACCOUNT-REQ-002` |
| `FE-ACCOUNT-UI-002` | Provide a short supporting description.                   | Frontend design      |
| `FE-ACCOUNT-UI-003` | Provide Create Account navigation.                        | `FE-ACCOUNT-REQ-003` |

Create Account navigates to:

```text
/admin/accounts/create
```

---

## 4.2 List Controls

```text
Status           [All ▼]
Include deleted  [ ]
Page size        [20 ▼]
                 [Refresh]
```

| ID                  | Control         | Behavior                       |
| ------------------- | --------------- | ------------------------------ |
| `FE-ACCOUNT-UI-004` | Status          | `All`, `Active`, `Inactive`    |
| `FE-ACCOUNT-UI-005` | Include deleted | Sends `include_deleted=true`   |
| `FE-ACCOUNT-UI-006` | Page size       | Supports values up to `100`    |
| `FE-ACCOUNT-UI-007` | Refresh         | Re-fetches current query state |

The UI must not add:

* Search.
* Client-side sorting.
* Unsupported filters.

---

## 4.3 Account Table

Use a native HTML `table`.

| ID                     | Column  | Source                 | Display                                |
| ---------------------- | ------- | ---------------------- | -------------------------------------- |
| `FE-ACCOUNT-TABLE-001` | Name    | `display_name`         | Value or `—`                           |
| `FE-ACCOUNT-TABLE-002` | Email   | `email`                | Email address                          |
| `FE-ACCOUNT-TABLE-003` | Status  | `status`, `deleted_at` | Active / Inactive / Deleted            |
| `FE-ACCOUNT-TABLE-004` | Created | `created_at`           | Localized date/time                    |
| `FE-ACCOUNT-TABLE-005` | Updated | `updated_at`           | Localized date/time                    |
| `FE-ACCOUNT-TABLE-006` | Actions | Derived                | Edit / Activate / Deactivate / Restore |

The table should provide an accessible caption or equivalent accessible name:

```text
Administrator accounts
```

The table must not be implemented as an ARIA `grid` unless grid-specific interaction becomes an explicit product requirement.

### Status Mapping

| Condition                               | UI Status  |
| --------------------------------------- | ---------- |
| `status=active` and `deleted_at=null`   | `Active`   |
| `status=inactive` and `deleted_at=null` | `Inactive` |
| `deleted_at != null`                    | `Deleted`  |

`Deleted` is a derived UI state, not a third Account `status` value.

### Row Actions

| ID                     | Account State         | Actions          | References                     |
| ---------------------- | --------------------- | ---------------- | ------------------------------ |
| `FE-ACCOUNT-TABLE-007` | Active, non-deleted   | Edit, Deactivate | `ADM-AUTH-005`, `ADM-AUTH-006` |
| `FE-ACCOUNT-TABLE-008` | Inactive, non-deleted | Edit, Activate   | `AC_UC_06`                     |
| `FE-ACCOUNT-TABLE-009` | Soft-deleted          | Restore          | `AC_UC_08`                     |

Hard Delete must not be exposed as a routine row action.

---

## 4.4 Account Count

The API returns:

```json
{
  "items": [],
  "page": 1,
  "page_size": 20,
  "total": 0
}
```

| ID                  | Rule                                                   |
| ------------------- | ------------------------------------------------------ |
| `FE-ACCOUNT-UI-008` | Display `0 accounts` when `total=0`.                   |
| `FE-ACCOUNT-UI-009` | Otherwise display `{start}–{end} of {total} accounts`. |
| `FE-ACCOUNT-UI-010` | Use the server-provided `total`.                       |

Calculation:

```text
start = ((page - 1) * page_size) + 1
end   = min(page * page_size, total)
```

---

## 4.5 Pagination

| ID                  | Requirement                                      |
| ------------------- | ------------------------------------------------ |
| `FE-ACCOUNT-UI-011` | First page is `1`.                               |
| `FE-ACCOUNT-UI-012` | Previous is disabled on page `1`.                |
| `FE-ACCOUNT-UI-013` | Next is disabled on the last page.               |
| `FE-ACCOUNT-UI-014` | Filter changes reset to page `1`.                |
| `FE-ACCOUNT-UI-015` | Page-size changes reset to page `1`.             |
| `FE-ACCOUNT-UI-016` | Pagination is server-side.                       |
| `FE-ACCOUNT-UI-017` | Client-side pagination is not used.              |
| `FE-ACCOUNT-UI-018` | The frontend preserves server response ordering. |

The backend ordering is:

```text
id ASC
```

The frontend must not re-sort the collection.

---

## 4.6 Responsive Layout

| ID                  | Viewport       | Layout                                       |
| ------------------- | -------------- | -------------------------------------------- |
| `FE-ACCOUNT-UI-019` | `>= 1280px`    | Full table                                   |
| `FE-ACCOUNT-UI-020` | `768px–1279px` | Full table with local horizontal overflow    |
| `FE-ACCOUNT-UI-021` | `< 768px`      | Compact table with local horizontal overflow |

Horizontal scrolling must be contained within the table region.

The following must remain unchanged across viewport sizes:

| ID                  | Behavior                |
| ------------------- | ----------------------- |
| `FE-ACCOUNT-UI-022` | Authentication behavior |
| `FE-ACCOUNT-UI-023` | Authorization behavior  |
| `FE-ACCOUNT-UI-024` | API contract            |
| `FE-ACCOUNT-UI-025` | Account state semantics |

---

# 5. API Contract

## 5.1 List Endpoint

```http
GET /admin/accounts
Authorization: Bearer <access-token>
```

| ID                   | Contract                                                              |
| -------------------- | --------------------------------------------------------------------- |
| `FE-ACCOUNT-API-001` | Use `GET /admin/accounts`.                                            |
| `FE-ACCOUNT-API-002` | Send bearer credentials using the `Authorization` header.             |
| `FE-ACCOUNT-API-003` | Do not send bearer credentials in query parameters or request bodies. |
| `FE-ACCOUNT-API-004` | Default `page=1`.                                                     |
| `FE-ACCOUNT-API-005` | Default `page_size=20`.                                               |
| `FE-ACCOUNT-API-006` | Never request `page_size > 100`.                                      |
| `FE-ACCOUNT-API-007` | `status` may be `active` or `inactive`.                               |
| `FE-ACCOUNT-API-008` | Default `include_deleted=false`.                                      |

References:

* `AC_API_02`
* `AC_SEC_DEC_AUTH_01`
* `AC_SEC_DEC_AUTH_02`
* `AC_SEC_DEC_AUTH_04`

---

## 5.2 Authorization

### Normal List

```text
include_deleted = false
```

Required permission:

```text
account:view
```

### Deleted Accounts

```text
include_deleted = true
```

Required permissions:

```text
account:view
account:view_deleted
```

| ID                   | Requirement                                                              | References           |
| -------------------- | ------------------------------------------------------------------------ | -------------------- |
| `FE-ACCOUNT-API-009` | Backend evaluates `account:view`.                                        | Authorization domain |
| `FE-ACCOUNT-API-010` | Backend evaluates `account:view_deleted` when `include_deleted=true`.    | Authorization domain |
| `FE-ACCOUNT-API-011` | Client roles and permissions must never affect the authorization result. | Authorization domain |
| `FE-ACCOUNT-API-012` | Missing required permission results in `403 Forbidden`.                  | Authorization domain |

---

## 5.3 Response

Expected response:

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

| ID                   | Requirement                                                      |
| -------------------- | ---------------------------------------------------------------- |
| `FE-ACCOUNT-API-013` | Use the server response as the authoritative Account List state. |
| `FE-ACCOUNT-API-014` | Preserve server response ordering.                               |
| `FE-ACCOUNT-API-015` | Ignore unknown response fields.                                  |
| `FE-ACCOUNT-API-016` | Never expect `password` or `password_hash` in the response.      |
| `FE-ACCOUNT-API-017` | Do not fabricate missing Account fields.                         |

---

## 5.4 Ordering

The backend guarantees:

```text
id ASC
```

| ID                   | Requirement                                  |
| -------------------- | -------------------------------------------- |
| `FE-ACCOUNT-API-018` | Preserve `id ASC` ordering.                  |
| `FE-ACCOUNT-API-019` | Do not re-sort the response in the frontend. |
| `FE-ACCOUNT-API-020` | Do not expose unsupported sort controls.     |

---

## 5.5 Cache

The Account API requires:

```http
Cache-Control: no-store
```

| ID                   | Requirement                                                |
| -------------------- | ---------------------------------------------------------- |
| `FE-ACCOUNT-API-021` | Do not persist Account List responses in `localStorage`.   |
| `FE-ACCOUNT-API-022` | Do not persist Account List responses in `sessionStorage`. |
| `FE-ACCOUNT-API-023` | Do not persist Account List responses in IndexedDB.        |
| `FE-ACCOUNT-API-024` | Do not use cached account data as authorization evidence.  |

---

## 5.6 Problem Details

Error responses use:

```text
application/problem+json
```

| ID                   | Requirement                                                  |
| -------------------- | ------------------------------------------------------------ |
| `FE-ACCOUNT-API-025` | Parse standard Problem Details fields when provided.         |
| `FE-ACCOUNT-API-026` | Prefer stable `code` values for feature-specific behavior.   |
| `FE-ACCOUNT-API-027` | Do not match internal behavior only by free-form error text. |

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

---

## 5.7 HTTP Status Handling

| Status          | Frontend Behavior                                    | ID                   |
| --------------- | ---------------------------------------------------- | -------------------- |
| `200`           | Replace list with authoritative response.            | `FE-ACCOUNT-API-028` |
| `204`           | Treat operation as successful with no response body. | `FE-ACCOUNT-API-029` |
| `400`           | Display request validation error.                    | `FE-ACCOUNT-API-030` |
| `401`           | Clear auth state and navigate to `/login`.           | `FE-ACCOUNT-API-031` |
| `403`           | Preserve auth state and display authorization error. | `FE-ACCOUNT-API-032` |
| `404`           | Refresh stale state and notify the user.             | `FE-ACCOUNT-API-033` |
| `409`           | Refresh state and display domain conflict.           | `FE-ACCOUNT-API-034` |
| `415`           | Display request media-type error.                    | `FE-ACCOUNT-API-035` |
| `422`           | Display validation error.                            | `FE-ACCOUNT-API-036` |
| `5xx`           | Display server error and allow retry.                | `FE-ACCOUNT-API-037` |
| Network failure | Preserve current state and allow retry.              | `FE-ACCOUNT-API-038` |

---

# 6. Account Actions

## 6.1 Edit

| ID                      | Requirement                                            | References                  |
| ----------------------- | ------------------------------------------------------ | --------------------------- |
| `FE-ACCOUNT-ACTION-001` | Navigate to `/admin/accounts/:id/edit`.                | `ADM-AUTH-005`, `AC_API_04` |
| `FE-ACCOUNT-ACTION-002` | Offer Edit only for non-deleted accounts.              | `AC_UC_04`                  |
| `FE-ACCOUNT-ACTION-003` | Do not implement Edit fields inside the list page.     | Scope boundary              |
| `FE-ACCOUNT-ACTION-004` | Preserve the previous list query state when returning. | Frontend behavior           |

Example return state:

```text
/admin/accounts?page=2&page_size=20&status=active
```

---

## 6.2 Deactivate

Endpoint:

```http
POST /admin/accounts/{id}/deactivate
```

| ID                      | Requirement                                             | References                 |
| ----------------------- | ------------------------------------------------------- | -------------------------- |
| `FE-ACCOUNT-ACTION-005` | Offer Deactivate only for active, non-deleted accounts. | `AC_UC_05`                 |
| `FE-ACCOUNT-ACTION-006` | Require confirmation before deactivation.               | Frontend behavior          |
| `FE-ACCOUNT-ACTION-007` | Disable the initiating action while pending.            | Frontend behavior          |
| `FE-ACCOUNT-ACTION-008` | Refresh the list after success.                         | `AC_API_05`                |
| `FE-ACCOUNT-ACTION-009` | Handle `LAST_ACTIVE_ADMINISTRATOR`.                     | `AC_UC_05`, `ADM-AUTH-007` |

Suggested confirmation:

```text
Deactivate this administrator account?

The account will no longer be able to authenticate until it is activated again.
```

Conflict message:

```text
This account cannot be deactivated because it is the last active administrator account.
```

---

## 6.3 Activate

Endpoint:

```http
POST /admin/accounts/{id}/activate
```

| ID                      | Requirement                                                            | References        |
| ----------------------- | ---------------------------------------------------------------------- | ----------------- |
| `FE-ACCOUNT-ACTION-010` | Offer Activate only for inactive, non-deleted accounts.                | `AC_UC_06`        |
| `FE-ACCOUNT-ACTION-011` | Disable the initiating action while pending.                           | Frontend behavior |
| `FE-ACCOUNT-ACTION-012` | Refresh the list after success.                                        | `AC_API_06`       |
| `FE-ACCOUNT-ACTION-013` | Treat `ACCOUNT_ALREADY_ACTIVE` as a stale-state condition and refresh. | `AC_API_06`       |

---

## 6.4 Restore

Endpoint:

```http
POST /admin/accounts/{id}/restore
```

| ID                      | Requirement                                                         | References        |
| ----------------------- | ------------------------------------------------------------------- | ----------------- |
| `FE-ACCOUNT-ACTION-014` | Offer Restore only for soft-deleted accounts.                       | `AC_UC_08`        |
| `FE-ACCOUNT-ACTION-015` | Allow Restore only when deleted accounts are included.              | `AC_API_02`       |
| `FE-ACCOUNT-ACTION-016` | Disable the initiating action while pending.                        | Frontend behavior |
| `FE-ACCOUNT-ACTION-017` | Refresh the list after success.                                     | `AC_API_08`       |
| `FE-ACCOUNT-ACTION-018` | Render restored account as `Inactive` unless API returns otherwise. | `AC_UC_08`        |

Restoration produces:

```text
status = inactive
deleted_at = null
```

---

## 6.5 Soft Delete

The Account domain supports:

```http
DELETE /admin/accounts/{id}
```

| ID                      | Requirement                                                                                            |
| ----------------------- | ------------------------------------------------------------------------------------------------------ |
| `FE-ACCOUNT-ACTION-019` | Do not expose Soft Delete as a primary Account List action unless explicitly enabled by product scope. |
| `FE-ACCOUNT-ACTION-020` | If later enabled, require confirmation and refresh authoritative data after success.                   |

References:

* `AC_UC_07`
* `AC_API_07`

---

## 6.6 Hard Delete

The Account domain supports:

```http
DELETE /admin/accounts/{id}/purge
```

| ID                      | Requirement                                                                       |
| ----------------------- | --------------------------------------------------------------------------------- |
| `FE-ACCOUNT-ACTION-021` | Do not expose Hard Delete as a routine Account List action.                       |
| `FE-ACCOUNT-ACTION-022` | Do not expose hard-delete UI without a separate frontend product/design decision. |

Reference:

* `AC_UC_09`
* `AC_API_09`

---

## 6.7 Mutation Behavior

All row mutations follow:

```text
User action
    ↓
Disable initiating action
    ↓
Send request
    ↓
Wait for server response
    ↓
Success → Refresh authoritative list
Failure → Preserve server-derived state and display error
```

| ID                      | Requirement                                                  |
| ----------------------- | ------------------------------------------------------------ |
| `FE-ACCOUNT-ACTION-023` | Prevent duplicate submission for the same mutation.          |
| `FE-ACCOUNT-ACTION-024` | Do not optimistically change account lifecycle state.        |
| `FE-ACCOUNT-ACTION-025` | Refresh after successful lifecycle mutation.                 |
| `FE-ACCOUNT-ACTION-026` | Refresh after stale-state `404` or relevant `409` responses. |
| `FE-ACCOUNT-ACTION-027` | Do not fabricate success from client-side state.             |

---

# 7. UI States

## 7.1 Loading

| ID                     | State            | Behavior                                         |
| ---------------------- | ---------------- | ------------------------------------------------ |
| `FE-ACCOUNT-STATE-001` | Initial Loading  | Keep shell and page structure visible.           |
| `FE-ACCOUNT-STATE-002` | Refresh Loading  | Keep current structure while refreshing.         |
| `FE-ACCOUNT-STATE-003` | Filter Loading   | Keep controls visible while loading new results. |
| `FE-ACCOUNT-STATE-004` | Mutation Pending | Disable the initiating row action.               |

A loading state must not unnecessarily shift the page layout.

---

## 7.2 Empty

### No Accounts

| ID                     | Behavior                                               |
| ---------------------- | ------------------------------------------------------ |
| `FE-ACCOUNT-STATE-005` | Show an empty-state message and Create Account action. |

```text
No administrator accounts

There are no administrator accounts to display.

[Create Account]
```

### Filtered Empty

| ID                     | Behavior                                            |
| ---------------------- | --------------------------------------------------- |
| `FE-ACCOUNT-STATE-006` | Show filtered empty state and Clear Filters action. |

```text
No matching administrator accounts

No accounts match the current filters.

[Clear Filters]
```

### No Deleted Accounts

| ID                     | Behavior                                          |
| ---------------------- | ------------------------------------------------- |
| `FE-ACCOUNT-STATE-007` | Show deleted-account empty state when applicable. |

```text
No deleted administrator accounts

There are no soft-deleted accounts to restore.
```

---

## 7.3 Errors

| ID                     | Error              | User Experience                                |
| ---------------------- | ------------------ | ---------------------------------------------- |
| `FE-ACCOUNT-STATE-008` | List fetch error   | Show retryable Account List error.             |
| `FE-ACCOUNT-STATE-009` | `403 Forbidden`    | Show authorization error without logout.       |
| `FE-ACCOUNT-STATE-010` | `401 Unauthorized` | Clear authentication and navigate to `/login`. |
| `FE-ACCOUNT-STATE-011` | `404 Not Found`    | Notify user and refresh list.                  |
| `FE-ACCOUNT-STATE-012` | `409 Conflict`     | Show domain conflict and refresh.              |
| `FE-ACCOUNT-STATE-013` | `422 Validation`   | Show server validation error.                  |
| `FE-ACCOUNT-STATE-014` | `5xx`              | Show retryable server error.                   |
| `FE-ACCOUNT-STATE-015` | Network failure    | Preserve current state and allow retry.        |

Generic fetch error:

```text
Unable to load administrator accounts

The account list could not be loaded.

[Try Again]
```

Authorization error:

```text
You do not have permission to view administrator accounts.
```

Authentication error:

```text
Your session is no longer valid.
```

---

## 7.4 Mutation Success

| ID                     | Behavior                                                           |
| ---------------------- | ------------------------------------------------------------------ |
| `FE-ACCOUNT-STATE-016` | Announce a successful mutation using an accessible status message. |
| `FE-ACCOUNT-STATE-017` | Refresh authoritative Account List data.                           |
| `FE-ACCOUNT-STATE-018` | Do not move focus unnecessarily.                                   |

Suggested message:

```text
Account updated.
```

---

## 7.5 Feature Error Boundary

| ID                     | Requirement                                                           |
| ---------------------- | --------------------------------------------------------------------- |
| `FE-ACCOUNT-STATE-019` | Feature errors remain inside Main Content.                            |
| `FE-ACCOUNT-STATE-020` | Admin Shell remains rendered during feature loading and error states. |

References:

* `FE_SHELL_STATE_09`
* `FE_SHELL_STATE_10`
* `FE_SHELL_STATE_11`

---

# 8. Security

| ID                   | Requirement                                                            | References                              |
| -------------------- | ---------------------------------------------------------------------- | --------------------------------------- |
| `FE-ACCOUNT-SEC-001` | `/admin/accounts` requires authentication.                             | `FE_SHELL_SEC_01`                       |
| `FE-ACCOUNT-SEC-002` | Backend Authentication is authoritative.                               | `FE_SHELL_SEC_03`                       |
| `FE-ACCOUNT-SEC-003` | Backend Authorization is authoritative.                                | `FE_SHELL_SEC_09`, `FE_SHELL_SEC_10`    |
| `FE-ACCOUNT-SEC-004` | Send bearer credentials only through `Authorization`.                  | `AC_SEC_DEC_AUTH_02`                    |
| `FE-ACCOUNT-SEC-005` | Never place access tokens in URLs.                                     | `AC_SEC_DEC_AUTH_04`, `FE_SHELL_SEC_05` |
| `FE-ACCOUNT-SEC-006` | Never render access tokens.                                            | `FE_SHELL_SEC_06`                       |
| `FE-ACCOUNT-SEC-007` | Never log access tokens.                                               | `FE_SHELL_SEC_07`                       |
| `FE-ACCOUNT-SEC-008` | Never render passwords or password hashes.                             | `AC_SEC_REQ_NON_FC_08`                  |
| `FE-ACCOUNT-SEC-009` | Never use client-provided roles or permissions as authorization proof. | Authorization domain                    |
| `FE-ACCOUNT-SEC-010` | UI visibility is not an authorization boundary.                        | Authorization domain                    |
| `FE-ACCOUNT-SEC-011` | Do not treat mutations as successful before backend confirmation.      | Account domain                          |
| `FE-ACCOUNT-SEC-012` | Do not persist Account List data in browser storage.                   | Account API                             |
| `FE-ACCOUNT-SEC-013` | Do not expose backend stack traces or internal infrastructure details. | Security requirement                    |
| `FE-ACCOUNT-SEC-014` | Do not use cached account data as authorization evidence.              | Security requirement                    |

## Request Pipeline

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

Authentication failure:

```text
401 Unauthorized
```

Authorization failure:

```text
403 Forbidden
```

The frontend must preserve this distinction.

---

# 9. Accessibility

The feature follows the Admin Shell accessibility requirements and should target WCAG 2.2 AA-oriented implementation.

| ID                    | Requirement                                             |
| --------------------- | ------------------------------------------------------- |
| `FE-ACCOUNT-A11Y-001` | Use semantic page structure.                            |
| `FE-ACCOUNT-A11Y-002` | Provide one primary `h1`.                               |
| `FE-ACCOUNT-A11Y-003` | Use a native HTML `table`.                              |
| `FE-ACCOUNT-A11Y-004` | Use semantic table headers.                             |
| `FE-ACCOUNT-A11Y-005` | Provide an accessible table caption or equivalent name. |
| `FE-ACCOUNT-A11Y-006` | Use actual links for navigation.                        |
| `FE-ACCOUNT-A11Y-007` | Use actual buttons for actions.                         |
| `FE-ACCOUNT-A11Y-008` | Provide visible keyboard focus.                         |
| `FE-ACCOUNT-A11Y-009` | Ensure focused elements are not completely obscured.    |
| `FE-ACCOUNT-A11Y-010` | Provide explicit labels for filters.                    |
| `FE-ACCOUNT-A11Y-011` | Provide descriptive names for row actions.              |
| `FE-ACCOUNT-A11Y-012` | Do not use color as the only status indicator.          |
| `FE-ACCOUNT-A11Y-013` | Announce relevant asynchronous status changes.          |
| `FE-ACCOUNT-A11Y-014` | Keep pagination keyboard accessible.                    |
| `FE-ACCOUNT-A11Y-015` | Keep keyboard navigation order predictable.             |
| `FE-ACCOUNT-A11Y-016` | Manage focus correctly for confirmation dialogs.        |
| `FE-ACCOUNT-A11Y-017` | Keep the page usable across supported viewport sizes.   |

## Row Action Names

| Action     | Accessible Name Example                     |
| ---------- | ------------------------------------------- |
| Edit       | `Edit account: Library Administrator`       |
| Activate   | `Activate account: Editor Account`          |
| Deactivate | `Deactivate account: Library Administrator` |
| Restore    | `Restore account: Former Administrator`     |

## Status

Status must always have text:

```text
Active
Inactive
Deleted
```

Color may supplement the label but must not carry the meaning alone.

---

# 10. Testing

## 10.1 Unit

| ID                         | Test                                            | Verifies                        |
| -------------------------- | ----------------------------------------------- | ------------------------------- |
| `FE-ACCOUNT-TEST-UNIT-001` | Account Management page renders                 | `FE-ACCOUNT-REQ-001`            |
| `FE-ACCOUNT-TEST-UNIT-002` | Table columns render correctly                  | `FE-ACCOUNT-TABLE-001` to `006` |
| `FE-ACCOUNT-TEST-UNIT-003` | Active/inactive/deleted states render correctly | `FE-ACCOUNT-TABLE-003`          |
| `FE-ACCOUNT-TEST-UNIT-004` | Null `display_name` renders safely              | `FE-ACCOUNT-TABLE-001`          |
| `FE-ACCOUNT-TEST-UNIT-005` | Pagination calculations are correct             | `FE-ACCOUNT-UI-011` to `018`    |
| `FE-ACCOUNT-TEST-UNIT-006` | Loading states render correctly                 | `FE-ACCOUNT-STATE-001` to `004` |
| `FE-ACCOUNT-TEST-UNIT-007` | Empty states render correctly                   | `FE-ACCOUNT-STATE-005` to `007` |
| `FE-ACCOUNT-TEST-UNIT-008` | 401 and 403 handling are distinct               | `FE-ACCOUNT-API-031`, `032`     |
| `FE-ACCOUNT-TEST-UNIT-009` | Duplicate mutation submission is prevented      | `FE-ACCOUNT-ACTION-023`         |
| `FE-ACCOUNT-TEST-UNIT-010` | Successful mutations trigger refresh            | `FE-ACCOUNT-ACTION-025`         |
| `FE-ACCOUNT-TEST-UNIT-011` | Action accessible names are generated correctly | `FE-ACCOUNT-A11Y-011`           |

---

## 10.2 Integration

| ID                        | Test                                             | Verifies                          |
| ------------------------- | ------------------------------------------------ | --------------------------------- |
| `FE-ACCOUNT-TEST-INT-001` | Authenticated user opens `/admin/accounts`       | `FE-ACCOUNT-ROUTE-001`            |
| `FE-ACCOUNT-TEST-INT-002` | Unauthenticated user redirects to `/login`       | `FE-ACCOUNT-ROUTE-002`            |
| `FE-ACCOUNT-TEST-INT-003` | Accounts navigation is active                    | `FE-ACCOUNT-REQ-001`, Admin Shell |
| `FE-ACCOUNT-TEST-INT-004` | List uses `GET /admin/accounts`                  | `FE-ACCOUNT-API-001`              |
| `FE-ACCOUNT-TEST-INT-005` | Default request excludes deleted accounts        | `FE-ACCOUNT-API-008`              |
| `FE-ACCOUNT-TEST-INT-006` | Status filter sends correct API value            | `FE-ACCOUNT-API-007`              |
| `FE-ACCOUNT-TEST-INT-007` | Pagination sends correct `page` and `page_size`  | `FE-ACCOUNT-API-004` to `006`     |
| `FE-ACCOUNT-TEST-INT-008` | Include Deleted sends `include_deleted=true`     | `FE-ACCOUNT-API-008`              |
| `FE-ACCOUNT-TEST-INT-009` | Create navigation uses correct route             | `FE-ACCOUNT-ACTION-001`           |
| `FE-ACCOUNT-TEST-INT-010` | Edit navigation uses correct route               | `FE-ACCOUNT-ACTION-001`           |
| `FE-ACCOUNT-TEST-INT-011` | Deactivate calls correct endpoint                | `FE-ACCOUNT-ACTION-005`           |
| `FE-ACCOUNT-TEST-INT-012` | Activate calls correct endpoint                  | `FE-ACCOUNT-ACTION-010`           |
| `FE-ACCOUNT-TEST-INT-013` | Restore calls correct endpoint                   | `FE-ACCOUNT-ACTION-014`           |
| `FE-ACCOUNT-TEST-INT-014` | Successful mutations refresh the list            | `FE-ACCOUNT-ACTION-025`           |
| `FE-ACCOUNT-TEST-INT-015` | 401 clears auth and navigates to login           | `FE-ACCOUNT-API-031`              |
| `FE-ACCOUNT-TEST-INT-016` | 403 preserves authentication                     | `FE-ACCOUNT-API-032`              |
| `FE-ACCOUNT-TEST-INT-017` | 409 conflict refreshes authoritative state       | `FE-ACCOUNT-API-034`              |
| `FE-ACCOUNT-TEST-INT-018` | Problem Details codes are handled                | `FE-ACCOUNT-API-025` to `027`     |
| `FE-ACCOUNT-TEST-INT-019` | Tokens are not present in URLs                   | `FE-ACCOUNT-SEC-005`              |
| `FE-ACCOUNT-TEST-INT-020` | Account data is not persisted in browser storage | `FE-ACCOUNT-SEC-012`              |

---

## 10.3 E2E

| ID                        | Test                                        | Verifies                     |
| ------------------------- | ------------------------------------------- | ---------------------------- |
| `FE-ACCOUNT-TEST-E2E-001` | Login → Admin Shell → Accounts works        | `FE-ACCOUNT-ROUTE-001`       |
| `FE-ACCOUNT-TEST-E2E-002` | Account rows render                         | `FE-ACCOUNT-REQ-002`         |
| `FE-ACCOUNT-TEST-E2E-003` | Status filter works                         | `FE-ACCOUNT-REQ-009`, `010`  |
| `FE-ACCOUNT-TEST-E2E-004` | Pagination works                            | `FE-ACCOUNT-REQ-012`         |
| `FE-ACCOUNT-TEST-E2E-005` | Create Account navigation works             | `FE-ACCOUNT-REQ-003`         |
| `FE-ACCOUNT-TEST-E2E-006` | Edit Account navigation works               | `FE-ACCOUNT-REQ-004`         |
| `FE-ACCOUNT-TEST-E2E-007` | Deactivate works                            | `FE-ACCOUNT-REQ-005`         |
| `FE-ACCOUNT-TEST-E2E-008` | Activate works                              | `FE-ACCOUNT-REQ-006`         |
| `FE-ACCOUNT-TEST-E2E-009` | Restore works when authorized               | `FE-ACCOUNT-REQ-007`         |
| `FE-ACCOUNT-TEST-E2E-010` | Last-active-administrator conflict is shown | `FE-ACCOUNT-ACTION-009`      |
| `FE-ACCOUNT-TEST-E2E-011` | Authentication expiry redirects to `/login` | `FE-ACCOUNT-REQ-016`         |
| `FE-ACCOUNT-TEST-E2E-012` | Mobile layout remains usable                | `FE-ACCOUNT-UI-019` to `025` |

---

## 10.4 Accessibility

| ID                         | Test                                 | Verifies                            |
| -------------------------- | ------------------------------------ | ----------------------------------- |
| `FE-ACCOUNT-TEST-A11Y-001` | Heading hierarchy is correct         | `FE-ACCOUNT-A11Y-001`, `002`        |
| `FE-ACCOUNT-TEST-A11Y-002` | Table headers are semantic           | `FE-ACCOUNT-A11Y-003`, `004`        |
| `FE-ACCOUNT-TEST-A11Y-003` | Table has accessible naming          | `FE-ACCOUNT-A11Y-005`               |
| `FE-ACCOUNT-TEST-A11Y-004` | All controls are keyboard accessible | `FE-ACCOUNT-A11Y-006`, `007`, `014` |
| `FE-ACCOUNT-TEST-A11Y-005` | Focus is visible                     | `FE-ACCOUNT-A11Y-008`               |
| `FE-ACCOUNT-TEST-A11Y-006` | Focused controls are not obscured    | `FE-ACCOUNT-A11Y-009`               |
| `FE-ACCOUNT-TEST-A11Y-007` | Status does not rely on color        | `FE-ACCOUNT-A11Y-012`               |
| `FE-ACCOUNT-TEST-A11Y-008` | Async messages are accessible        | `FE-ACCOUNT-A11Y-013`               |
| `FE-ACCOUNT-TEST-A11Y-009` | Dialog focus behavior is correct     | `FE-ACCOUNT-A11Y-016`               |

---

## 10.5 Responsive

| ID                         | Test                                         | Verifies                     |
| -------------------------- | -------------------------------------------- | ---------------------------- |
| `FE-ACCOUNT-TEST-RESP-001` | Desktop works at `>=1280px`                  | `FE-ACCOUNT-UI-019`          |
| `FE-ACCOUNT-TEST-RESP-002` | Tablet works at `768px–1279px`               | `FE-ACCOUNT-UI-020`          |
| `FE-ACCOUNT-TEST-RESP-003` | Mobile works below `768px`                   | `FE-ACCOUNT-UI-021`          |
| `FE-ACCOUNT-TEST-RESP-004` | Page-level horizontal scrolling is prevented | `FE-ACCOUNT-REQ-028`         |
| `FE-ACCOUNT-TEST-RESP-005` | Table overflow remains local                 | `FE-ACCOUNT-UI-019` to `021` |
| `FE-ACCOUNT-TEST-RESP-006` | Authentication behavior is unchanged         | `FE-ACCOUNT-UI-022`          |
| `FE-ACCOUNT-TEST-RESP-007` | Authorization behavior is unchanged          | `FE-ACCOUNT-UI-023`          |

---

# 11. Implementation Criteria

## 11.1 Route

| ID                     | Criteria                                      | Status         | References          |
| ---------------------- | --------------------------------------------- | -------------- | ------------------- |
| `FE-ACCOUNT-ROUTE-001` | `/admin/accounts` renders inside Admin Shell. | 🟡 In Progress | `PAGE-ADM-008`      |
| `FE-ACCOUNT-ROUTE-002` | Unauthenticated access redirects to `/login`. | 🟢 Implemented | `FE_SHELL_ROUTE_05` |
| `FE-ACCOUNT-ROUTE-003` | Accounts navigation is active.                | 🟡 In Progress | `FE_SHELL_UI_10`    |
| `FE-ACCOUNT-ROUTE-004` | Create Account route exists.                  | 🟡 In Progress | `PAGE-ADM-009`      |
| `FE-ACCOUNT-ROUTE-005` | Edit Account route exists.                    | 🟡 In Progress | `PAGE-ADM-010`      |

---

## 11.2 Requirements

| ID                   | Criteria                                      | Status         |
| -------------------- | --------------------------------------------- | -------------- |
| `FE-ACCOUNT-REQ-001` | Protected route implemented.                  | 🟡 In Progress |
| `FE-ACCOUNT-REQ-002` | Account list rendered.                        | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-003` | Create navigation implemented.                | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-004` | Edit navigation implemented.                  | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-005` | Deactivation implemented.                     | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-006` | Activation implemented.                       | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-007` | Restoration implemented.                      | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-008` | Deleted accounts excluded by default.         | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-009` | Active filter implemented.                    | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-010` | Inactive filter implemented.                  | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-011` | Deleted-account workflow implemented.         | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-012` | Server-side pagination implemented.           | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-013` | Server ordering preserved.                    | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-014` | Backend authentication remains authoritative. | 🟢 Implemented |
| `FE-ACCOUNT-REQ-015` | Backend authorization remains authoritative.  | 🟢 Implemented |
| `FE-ACCOUNT-REQ-016` | `401` handling implemented.                   | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-017` | `403` handling implemented.                   | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-018` | Mutation refresh implemented.                 | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-019` | Duplicate mutation prevention implemented.    | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-020` | List context preserved across Edit.           | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-021` | Unsupported sorting excluded.                 | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-022` | Unsupported search/filtering excluded.        | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-023` | Credential/token exposure prevented.          | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-024` | Account data persistence prevented.           | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-025` | Keyboard interaction implemented.             | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-026` | Visible focus implemented.                    | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-027` | Async accessibility implemented.              | ⚪ Not Started  |
| `FE-ACCOUNT-REQ-028` | Page-level horizontal scrolling prevented.    | ⚪ Not Started  |

---

## 11.3 API

| ID                   | Criteria                                         | Status         |
| -------------------- | ------------------------------------------------ | -------------- |
| `FE-ACCOUNT-API-001` | List uses `GET /admin/accounts`.                 | ⚪ Not Started  |
| `FE-ACCOUNT-API-002` | Bearer credentials use `Authorization`.          | ⚪ Not Started  |
| `FE-ACCOUNT-API-003` | Tokens are not sent in URLs or bodies.           | ⚪ Not Started  |
| `FE-ACCOUNT-API-004` | Default page is `1`.                             | ⚪ Not Started  |
| `FE-ACCOUNT-API-005` | Default page size is `20`.                       | ⚪ Not Started  |
| `FE-ACCOUNT-API-006` | Page size never exceeds `100`.                   | ⚪ Not Started  |
| `FE-ACCOUNT-API-007` | Status filter maps to API values.                | ⚪ Not Started  |
| `FE-ACCOUNT-API-008` | Deleted accounts are excluded by default.        | ⚪ Not Started  |
| `FE-ACCOUNT-API-009` | `account:view` is enforced by backend.           | 🟢 Implemented |
| `FE-ACCOUNT-API-010` | `account:view_deleted` is enforced by backend.   | 🟢 Implemented |
| `FE-ACCOUNT-API-011` | Client authorization claims are ignored.         | 🟢 Implemented |
| `FE-ACCOUNT-API-012` | Missing permissions result in `403`.             | 🟢 Implemented |
| `FE-ACCOUNT-API-013` | Server response is authoritative.                | ⚪ Not Started  |
| `FE-ACCOUNT-API-018` | Backend ordering is preserved.                   | ⚪ Not Started  |
| `FE-ACCOUNT-API-021` | Account responses are not persisted.             | ⚪ Not Started  |
| `FE-ACCOUNT-API-025` | Problem Details are parsed.                      | ⚪ Not Started  |
| `FE-ACCOUNT-API-031` | `401` clears authentication state and redirects. | ⚪ Not Started  |
| `FE-ACCOUNT-API-032` | `403` preserves authentication state.            | ⚪ Not Started  |

---

## 11.4 Actions

| ID                      | Criteria                                    | Status        |
| ----------------------- | ------------------------------------------- | ------------- |
| `FE-ACCOUNT-ACTION-001` | Edit navigation implemented.                | ⚪ Not Started |
| `FE-ACCOUNT-ACTION-005` | Deactivate implemented.                     | ⚪ Not Started |
| `FE-ACCOUNT-ACTION-009` | Last-active-administrator conflict handled. | ⚪ Not Started |
| `FE-ACCOUNT-ACTION-010` | Activate implemented.                       | ⚪ Not Started |
| `FE-ACCOUNT-ACTION-013` | Stale activation state handled.             | ⚪ Not Started |
| `FE-ACCOUNT-ACTION-014` | Restore implemented.                        | ⚪ Not Started |
| `FE-ACCOUNT-ACTION-018` | Restored account renders as inactive.       | ⚪ Not Started |
| `FE-ACCOUNT-ACTION-019` | Soft Delete is not exposed by default.      | ⚪ Not Started |
| `FE-ACCOUNT-ACTION-021` | Hard Delete is not exposed.                 | ⚪ Not Started |
| `FE-ACCOUNT-ACTION-023` | Duplicate submissions prevented.            | ⚪ Not Started |
| `FE-ACCOUNT-ACTION-024` | Optimistic lifecycle updates are not used.  | ⚪ Not Started |
| `FE-ACCOUNT-ACTION-025` | Successful mutation refreshes list.         | ⚪ Not Started |

---

## 11.5 UI States

| ID                     | Criteria                                   | Status         |
| ---------------------- | ------------------------------------------ | -------------- |
| `FE-ACCOUNT-STATE-001` | Initial loading state implemented.         | ⚪ Not Started  |
| `FE-ACCOUNT-STATE-002` | Refresh loading state implemented.         | ⚪ Not Started  |
| `FE-ACCOUNT-STATE-003` | Filter loading state implemented.          | ⚪ Not Started  |
| `FE-ACCOUNT-STATE-004` | Mutation pending state implemented.        | ⚪ Not Started  |
| `FE-ACCOUNT-STATE-005` | Empty state implemented.                   | ⚪ Not Started  |
| `FE-ACCOUNT-STATE-006` | Filtered empty state implemented.          | ⚪ Not Started  |
| `FE-ACCOUNT-STATE-007` | Deleted-account empty state implemented.   | ⚪ Not Started  |
| `FE-ACCOUNT-STATE-008` | General error state implemented.           | ⚪ Not Started  |
| `FE-ACCOUNT-STATE-009` | Authorization error implemented.           | ⚪ Not Started  |
| `FE-ACCOUNT-STATE-010` | Authentication expiry handled.             | 🟢 Implemented |
| `FE-ACCOUNT-STATE-011` | Not-found mutation state implemented.      | ⚪ Not Started  |
| `FE-ACCOUNT-STATE-012` | Conflict state implemented.                | ⚪ Not Started  |
| `FE-ACCOUNT-STATE-013` | Validation state implemented.              | ⚪ Not Started  |
| `FE-ACCOUNT-STATE-014` | Server-error state implemented.            | ⚪ Not Started  |
| `FE-ACCOUNT-STATE-015` | Network-error state implemented.           | ⚪ Not Started  |
| `FE-ACCOUNT-STATE-016` | Mutation success announced.                | ⚪ Not Started  |
| `FE-ACCOUNT-STATE-019` | Feature errors remain inside Main Content. | 🟢 Implemented |

---

## 11.6 Security

| ID                   | Criteria                                           | Status         |
| -------------------- | -------------------------------------------------- | -------------- |
| `FE-ACCOUNT-SEC-001` | Route requires authentication.                     | 🟢 Implemented |
| `FE-ACCOUNT-SEC-002` | Backend Authentication is authoritative.           | 🟢 Implemented |
| `FE-ACCOUNT-SEC-003` | Backend Authorization is authoritative.            | 🟢 Implemented |
| `FE-ACCOUNT-SEC-004` | Bearer token uses `Authorization`.                 | ⚪ Not Started  |
| `FE-ACCOUNT-SEC-005` | Tokens never appear in URLs.                       | ⚪ Not Started  |
| `FE-ACCOUNT-SEC-006` | Tokens never render.                               | ⚪ Not Started  |
| `FE-ACCOUNT-SEC-007` | Tokens never log.                                  | ⚪ Not Started  |
| `FE-ACCOUNT-SEC-008` | Passwords and hashes never render.                 | ⚪ Not Started  |
| `FE-ACCOUNT-SEC-009` | Client roles/permissions are not trusted.          | 🟢 Implemented |
| `FE-ACCOUNT-SEC-010` | UI visibility is not an authorization boundary.    | 🟢 Implemented |
| `FE-ACCOUNT-SEC-011` | Mutation requires server confirmation.             | ⚪ Not Started  |
| `FE-ACCOUNT-SEC-012` | Account data is not persisted client-side.         | ⚪ Not Started  |
| `FE-ACCOUNT-SEC-013` | Internal errors are not exposed.                   | ⚪ Not Started  |
| `FE-ACCOUNT-SEC-014` | Cached data is not used as authorization evidence. | ⚪ Not Started  |

---

## 11.7 Accessibility

| ID                    | Criteria                               | Status        |
| --------------------- | -------------------------------------- | ------------- |
| `FE-ACCOUNT-A11Y-001` | Semantic page structure implemented.   | ⚪ Not Started |
| `FE-ACCOUNT-A11Y-002` | Primary `h1` implemented.              | ⚪ Not Started |
| `FE-ACCOUNT-A11Y-003` | Native table implemented.              | ⚪ Not Started |
| `FE-ACCOUNT-A11Y-004` | Semantic table headers implemented.    | ⚪ Not Started |
| `FE-ACCOUNT-A11Y-005` | Accessible table naming implemented.   | ⚪ Not Started |
| `FE-ACCOUNT-A11Y-006` | Navigation uses actual links.          | ⚪ Not Started |
| `FE-ACCOUNT-A11Y-007` | Actions use actual buttons.            | ⚪ Not Started |
| `FE-ACCOUNT-A11Y-008` | Visible focus implemented.             | ⚪ Not Started |
| `FE-ACCOUNT-A11Y-009` | Focus is not obscured.                 | ⚪ Not Started |
| `FE-ACCOUNT-A11Y-010` | Filter labels implemented.             | ⚪ Not Started |
| `FE-ACCOUNT-A11Y-011` | Row action names are descriptive.      | ⚪ Not Started |
| `FE-ACCOUNT-A11Y-012` | Status does not rely on color.         | ⚪ Not Started |
| `FE-ACCOUNT-A11Y-013` | Async status is accessible.            | ⚪ Not Started |
| `FE-ACCOUNT-A11Y-014` | Pagination is keyboard accessible.     | ⚪ Not Started |
| `FE-ACCOUNT-A11Y-015` | Keyboard order is predictable.         | ⚪ Not Started |
| `FE-ACCOUNT-A11Y-016` | Dialog focus is managed correctly.     | ⚪ Not Started |
| `FE-ACCOUNT-A11Y-017` | Responsive accessibility is preserved. | ⚪ Not Started |

---

# 12. Traceability

## Requirements → Design → Implementation → Test

| Requirement            | Frontend Design                           | Backend Contract            | Test                               |
| ---------------------- | ----------------------------------------- | --------------------------- | ---------------------------------- |
| `ADM-AUTH-001`         | `FE-ACCOUNT-ROUTE-001`                    | Authentication domain       | `FE-ACCOUNT-TEST-INT-001`          |
| `ADM-AUTH-004`         | `FE-ACCOUNT-TABLE-001` to `006`           | `AC_UC_02`, `AC_API_02`     | `FE-ACCOUNT-TEST-INT-004`          |
| `ADM-AUTH-003`         | `FE-ACCOUNT-ACTION-001`                   | `AC_UC_01`, `AC_API_01`     | `FE-ACCOUNT-TEST-E2E-005`          |
| `ADM-AUTH-005`         | `FE-ACCOUNT-ACTION-001`                   | `AC_UC_04`, `AC_API_04`     | `FE-ACCOUNT-TEST-E2E-006`          |
| `ADM-AUTH-006`         | `FE-ACCOUNT-ACTION-005`                   | `AC_UC_05`, `AC_API_05`     | `FE-ACCOUNT-TEST-E2E-007`          |
| `ADM-AUTH-007`         | `FE-ACCOUNT-ACTION-009`                   | `LAST_ACTIVE_ADMINISTRATOR` | `FE-ACCOUNT-TEST-E2E-010`          |
| `AC_UC_06`             | `FE-ACCOUNT-ACTION-010`                   | `AC_API_06`                 | `FE-ACCOUNT-TEST-E2E-008`          |
| `AC_UC_08`             | `FE-ACCOUNT-ACTION-014`                   | `AC_API_08`                 | `FE-ACCOUNT-TEST-E2E-009`          |
| `account:view`         | `FE-ACCOUNT-API-009`                      | Authorization domain        | `FE-ACCOUNT-TEST-INT-016`          |
| `account:view_deleted` | `FE-ACCOUNT-API-010`                      | Authorization domain        | Deleted-account authorization test |
| Authentication `401`   | `FE-ACCOUNT-API-031`                      | Authentication domain       | `FE-ACCOUNT-TEST-INT-015`          |
| Authorization `403`    | `FE-ACCOUNT-API-032`                      | Authorization domain        | `FE-ACCOUNT-TEST-INT-016`          |
| Admin Shell            | `FE-ACCOUNT-ROUTE-001`, `FE-ACCOUNT-UI-*` | `admin_shell.md`            | `FE-ACCOUNT-TEST-E2E-001`          |

## Domain References

| Domain         | Relevant References                                                       |
| -------------- | ------------------------------------------------------------------------- |
| Requirements   | `ADM-AUTH-001` to `ADM-AUTH-007`, `CNT-ADMIN-*`, `SCP-009`                |
| Sitemap        | `PAGE-ADM-008`, `PAGE-ADM-009`, `PAGE-ADM-010`                            |
| Account        | `AC_UC_01` to `AC_UC_11`, `AC_API_01` to `AC_API_11`                      |
| Authentication | Protected request contract, `AuthenticatedPrincipal`, `401` handling      |
| Authorization  | `account:view`, `account:view_deleted`, deny-by-default, `403`            |
| Admin Shell    | `FE_SHELL_ROUTE_05`, `FE_SHELL_UI_*`, `FE_SHELL_SEC_*`, `FE_SHELL_A11Y_*` |
