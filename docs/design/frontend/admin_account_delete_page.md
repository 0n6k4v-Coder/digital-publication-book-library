# Frontend Admin Account Delete Page

## Table of Contents

1. [Scope](#1-scope)
2. [Routes](#2-routes)
3. [Requirements](#3-requirements)
4. [Target Account Resolution](#4-target-account-resolution)
5. [Page Structure](#5-page-structure)
6. [Soft Delete Flow](#6-soft-delete-flow)
7. [Hard Delete Flow](#7-hard-delete-flow)
8. [Confirmation Dialogs](#8-confirmation-dialogs)
9. [API Contract](#9-api-contract)
10. [Validation and Conflict Handling](#10-validation-and-conflict-handling)
11. [UI States](#11-ui-states)
12. [Return Navigation](#12-return-navigation)
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

This document defines the protected **Administrator Account Delete** frontend page.

The canonical route is:

```text
/admin/accounts/:id/delete
```

The page is rendered inside the existing [Admin Shell](./admin_shell.md).

The page owns:

* Explicit Account deletion confirmation.
* Administrator Account soft deletion.
* Administrator Account hard deletion.
* Delete and purge mutation states.
* Authentication and authorization error presentation.
* Lifecycle conflict handling.
* Return navigation to Account Management.
* Accessible confirmation interaction.

The page does not own:

* Account creation.
* Account field editing.
* Account read-only Detail behavior.
* Account activation.
* Account deactivation outside the delete operation.
* Account restoration.
* Password management.
* Email management.
* Role assignment.
* Role revocation.
* Authentication implementation.
* Authorization policy implementation.
* Database transactions.
* Referential-action cleanup.
* Session or token revocation implementation.

The backend Account and Authorization domains remain authoritative for deletion rules, lifecycle invariants, authorization, persistence, and cleanup.

## Responsibilities

| ID                            | Responsibility                                                                         |
| ----------------------------- | -------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DELETE-SCOPE-001` | Load the target Account when available through normal Account lookup.                  |
| `FE-ACCOUNT-DELETE-SCOPE-002` | Identify the target Account and requested destructive operation.                       |
| `FE-ACCOUNT-DELETE-SCOPE-003` | Require explicit user confirmation before any deletion mutation.                       |
| `FE-ACCOUNT-DELETE-SCOPE-004` | Execute Account soft deletion through `AC_API_07`.                                     |
| `FE-ACCOUNT-DELETE-SCOPE-005` | Execute Account hard deletion through `AC_API_09`.                                     |
| `FE-ACCOUNT-DELETE-SCOPE-006` | Handle authentication, authorization, conflict, not-found, server, and network errors. |
| `FE-ACCOUNT-DELETE-SCOPE-007` | Preserve originating Account List return navigation.                                   |
| `FE-ACCOUNT-DELETE-SCOPE-008` | Prevent duplicate destructive submissions.                                             |
| `FE-ACCOUNT-DELETE-SCOPE-009` | Provide accessible confirmation, status, and error interaction.                        |
| `FE-ACCOUNT-DELETE-SCOPE-010` | Preserve Admin Shell authentication and navigation behavior.                           |

## Out of Scope

| ID                          | Excluded Area                              |
| --------------------------- | ------------------------------------------ |
| `FE-ACCOUNT-DELETE-OOS-001` | Authentication implementation.             |
| `FE-ACCOUNT-DELETE-OOS-002` | Authorization implementation.              |
| `FE-ACCOUNT-DELETE-OOS-003` | Account persistence.                       |
| `FE-ACCOUNT-DELETE-OOS-004` | Database transaction management.           |
| `FE-ACCOUNT-DELETE-OOS-005` | Referential-action cleanup.                |
| `FE-ACCOUNT-DELETE-OOS-006` | Authentication/session invalidation logic. |
| `FE-ACCOUNT-DELETE-OOS-007` | Authorization role cleanup.                |
| `FE-ACCOUNT-DELETE-OOS-008` | Account restoration.                       |
| `FE-ACCOUNT-DELETE-OOS-009` | Account editing.                           |
| `FE-ACCOUNT-DELETE-OOS-010` | Role management.                           |

---

# 2. Routes

| ID                            | Route                        | Access                       | Behavior                                                                         | References                            |
| ----------------------------- | ---------------------------- | ---------------------------- | -------------------------------------------------------------------------------- | ------------------------------------- |
| `FE-ACCOUNT-DELETE-ROUTE-001` | `/admin/accounts/:id/delete` | Authenticated                | Render Account Delete page.                                                      | Account Management, Account Detail    |
| `FE-ACCOUNT-DELETE-ROUTE-002` | `/admin/accounts/:id/delete` | Bootstrap                    | Keep route pending until shared authentication resolves.                         | Authentication domain, Admin Shell    |
| `FE-ACCOUNT-DELETE-ROUTE-003` | `/admin/accounts/:id/delete` | Unauthenticated              | Redirect through shared authentication behavior to `/login`.                     | `ADM-AUTH-001`, Admin Shell           |
| `FE-ACCOUNT-DELETE-ROUTE-004` | `/admin/accounts/:id/delete` | Authenticated + unauthorized | Render the applicable authorization error while preserving authentication state. | Authorization domain                  |
| `FE-ACCOUNT-DELETE-ROUTE-005` | `/admin/accounts/:id/delete` | Invalid Account ID           | Render invalid Account identifier error.                                         | `AC_API_07`, `AC_API_09`              |
| `FE-ACCOUNT-DELETE-ROUTE-006` | `/admin/accounts/:id/delete` | Unavailable Account          | Render unavailable Account state and prevent soft-delete submission.             | `AC_API_03`, `AC_API_07`, `AC_API_09` |

The route parameter is the Account UUID:

```text
/admin/accounts/:id/delete
```

Authentication credentials, passwords, access tokens, refresh credentials, and other secrets must never appear in the route.

The Delete route is not a mutation endpoint. Loading the page must never perform deletion.

The repository sitemap currently does not define a Page ID for this route. Sitemap alignment is tracked in [Implementation Criteria](#18-implementation-criteria).

---

# 3. Requirements

| ID                          | Requirement                                                                    | Repository Reference               |
| --------------------------- | ------------------------------------------------------------------------------ | ---------------------------------- |
| `FE-ACCOUNT-DELETE-REQ-001` | Provide a protected Account Delete route.                                      | `ADM-AUTH-001`                     |
| `FE-ACCOUNT-DELETE-REQ-002` | Support explicit Account soft deletion.                                        | `AC_UC_07`, `AC_API_07`            |
| `FE-ACCOUNT-DELETE-REQ-003` | Support explicit Account hard deletion.                                        | `AC_UC_09`, `AC_API_09`            |
| `FE-ACCOUNT-DELETE-REQ-004` | Require explicit confirmation before soft deletion.                            | Account Delete design              |
| `FE-ACCOUNT-DELETE-REQ-005` | Require explicit confirmation before hard deletion.                            | Account Delete design              |
| `FE-ACCOUNT-DELETE-REQ-006` | Use backend domains as the authoritative deletion and authorization authority. | Account API, Authorization domain  |
| `FE-ACCOUNT-DELETE-REQ-007` | Handle `401 Unauthorized` through shared authentication behavior.              | Authentication domain              |
| `FE-ACCOUNT-DELETE-REQ-008` | Handle `403 Forbidden` without unnecessary logout.                             | Authorization domain               |
| `FE-ACCOUNT-DELETE-REQ-009` | Handle `404 ACCOUNT_NOT_FOUND` without claiming a successful deletion.         | `AC_API_07`, `AC_API_09`           |
| `FE-ACCOUNT-DELETE-REQ-010` | Handle `409 ACCOUNT_ALREADY_DELETED` for soft deletion.                        | `AC_API_07`                        |
| `FE-ACCOUNT-DELETE-REQ-011` | Handle `409 LAST_ACTIVE_ADMINISTRATOR` for destructive operations.             | `AC_API_07`, `AC_API_09`           |
| `FE-ACCOUNT-DELETE-REQ-012` | Prevent duplicate destructive submissions.                                     | Frontend behavior                  |
| `FE-ACCOUNT-DELETE-REQ-013` | Never perform deletion through a GET request or route navigation alone.        | HTTP semantics                     |
| `FE-ACCOUNT-DELETE-REQ-014` | Preserve originating Account List navigation state.                            | Account Management frontend design |
| `FE-ACCOUNT-DELETE-REQ-015` | Do not expose passwords, password hashes, tokens, or authentication secrets.   | Account Security                   |
| `FE-ACCOUNT-DELETE-REQ-016` | Provide keyboard-accessible confirmation and cancellation.                     | WCAG 2.2, WAI-ARIA APG             |
| `FE-ACCOUNT-DELETE-REQ-017` | Provide accessible status and error messaging.                                 | WCAG 2.2                           |
| `FE-ACCOUNT-DELETE-REQ-018` | Preserve Admin Shell behavior across supported viewport sizes.                 | Admin Shell frontend design        |
| `FE-ACCOUNT-DELETE-REQ-019` | Do not infer authorization from client-held permissions.                       | Authorization domain               |
| `FE-ACCOUNT-DELETE-REQ-020` | Do not persist Account representations in browser storage.                     | Account API `no-store` contract    |

---

# 4. Target Account Resolution

## 4.1 Normal Account Lookup

The page should first resolve the target Account through:

```http
GET /admin/accounts/{id}
```

A successful response identifies a non-soft-deleted Account because normal Account lookup excludes soft-deleted Accounts.

The response is authoritative for:

```text
id
email
display_name
status
created_at
updated_at
deleted_at
```

The frontend must not fabricate Account values.

## 4.2 Soft-Deleted Target

A soft-deleted Account is intentionally excluded from:

```http
GET /admin/accounts/{id}
```

Therefore, `404 ACCOUNT_NOT_FOUND` from normal lookup must not be interpreted by the frontend as proof that the Account was soft-deleted.

The frontend must not infer lifecycle state from `404`.

Hard deletion remains independently authoritative because `AC_API_09` permits hard deletion of active, inactive, or soft-deleted Accounts.

For a target that cannot be loaded through normal lookup, the page may present a minimal identifier-based confirmation for hard deletion:

```text
Account ID
019...
```

The frontend must not claim the Account's email, display name, or lifecycle state when those values were not obtained from an authoritative response.

## 4.3 Target State Matrix

| Target State               | Normal Lookup | Soft Delete   | Hard Delete           |
| -------------------------- | ------------- | ------------- | --------------------- |
| Active Account             | `200`         | Available     | Available             |
| Inactive Account           | `200`         | Available     | Available             |
| Soft-deleted Account       | `404`         | Not available | Available             |
| Physically deleted Account | `404`         | Not available | Backend returns `404` |
| Invalid Account ID         | `400`         | Not available | Not available         |

The backend remains authoritative for the final result of every mutation.

---

# 5. Page Structure

## 5.1 Page Header

Recommended structure:

```text
Back to Administrator Accounts

Delete Administrator Account
Library Administrator
admin@example.com
Active
```

For a target unavailable through normal lookup:

```text
Delete Administrator Account

Account ID
019...

The account is not available through normal account lookup.
```

| ID                         | Requirement                                                               |
| -------------------------- | ------------------------------------------------------------------------- |
| `FE-ACCOUNT-DELETE-UI-001` | Use `Delete Administrator Account` as the primary page heading.           |
| `FE-ACCOUNT-DELETE-UI-002` | Display the authoritative Account display name when available.            |
| `FE-ACCOUNT-DELETE-UI-003` | Display the authoritative Account email when available.                   |
| `FE-ACCOUNT-DELETE-UI-004` | Display lifecycle status when returned authoritatively.                   |
| `FE-ACCOUNT-DELETE-UI-005` | Display the Account ID as read-only context.                              |
| `FE-ACCOUNT-DELETE-UI-006` | Provide native Back navigation to the validated Account List destination. |
| `FE-ACCOUNT-DELETE-UI-007` | Keep destructive actions visually and semantically distinct.              |
| `FE-ACCOUNT-DELETE-UI-008` | Do not execute a mutation merely by loading or navigating to the page.    |

## 5.2 Deletion Actions

For a loaded non-deleted Account:

```text
Delete Account
    Soft delete

Permanently Delete Account
    Hard delete
```

For a target that is unavailable through normal lookup:

```text
Permanently Delete Account
```

may remain available because the backend hard-delete contract permits soft-deleted targets.

| ID                             | Requirement                                                                              |
| ------------------------------ | ---------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DELETE-ACTION-001` | Provide a Delete Account action for soft deletion when the target is normally available. |
| `FE-ACCOUNT-DELETE-ACTION-002` | Provide a Permanently Delete Account action when the hard-delete operation is available. |
| `FE-ACCOUNT-DELETE-ACTION-003` | Never execute either mutation without explicit confirmation.                             |
| `FE-ACCOUNT-DELETE-ACTION-004` | Use operation-specific labels instead of ambiguous labels such as `Yes`.                 |
| `FE-ACCOUNT-DELETE-ACTION-005` | Disable the initiating destructive action while the corresponding mutation is pending.   |
| `FE-ACCOUNT-DELETE-ACTION-006` | Prevent simultaneous destructive mutations.                                              |

## 5.3 Deletion Effects

Soft deletion:

```text
Account
    ↓
status = inactive
deleted_at = populated
deleted_by = populated
```

Hard deletion:

```text
Account
    ↓
Account row removed
    ↓
Defined dependent records removed atomically
```

The frontend must not implement these persistence effects locally.

---

# 6. Soft Delete Flow

## 6.1 Soft Delete Operation

Soft deletion uses:

```http
DELETE /admin/accounts/{id}
```

Permission:

```text
account:delete
```

The operation must only be executed after explicit confirmation.

## 6.2 Confirmation

The confirmation must communicate:

```text
The account will be soft-deleted.
Its status will become inactive.
It can be restored later through the Account lifecycle flow.
```

Recommended confirmation:

```text
Delete administrator account?

Library Administrator
admin@example.com

This will deactivate the account and mark it as deleted.
The account can be restored later.

[Cancel] [Delete account]
```

## 6.3 Success

The backend returns:

```text
204 No Content
```

The frontend must:

1. Treat the mutation as successful only after the `204` response.
2. Not fabricate a new Account representation.
3. Navigate to the validated Account List destination.
4. Allow Account Management to re-fetch authoritative data.
5. Preserve originating Account List query state.

## 6.4 Soft Delete Conflicts

| Status | Code                        | UI Behavior                                                                    |
| ------ | --------------------------- | ------------------------------------------------------------------------------ |
| `404`  | `ACCOUNT_NOT_FOUND`         | Account is unavailable; do not claim deletion.                                 |
| `409`  | `ACCOUNT_ALREADY_DELETED`   | Explain that the Account was already deleted and return to Account Management. |
| `409`  | `LAST_ACTIVE_ADMINISTRATOR` | Explain that the last active administrator cannot be deleted.                  |

The frontend must not attempt to bypass the `LAST_ACTIVE_ADMINISTRATOR` invariant.

---

# 7. Hard Delete Flow

## 7.1 Hard Delete Operation

Hard deletion uses:

```http
DELETE /admin/accounts/{id}/purge
```

Permission:

```text
account:purge
```

The operation may target:

```text
active
inactive
soft-deleted
```

Accounts.

## 7.2 Confirmation

Hard deletion requires a stronger confirmation message because the operation is irreversible.

Recommended confirmation:

```text
Permanently delete administrator account?

Library Administrator
admin@example.com

This permanently deletes the account and its associated credentials,
authentication state, and defined account-owned records.
This action cannot be undone.

[Cancel] [Permanently delete account]
```

For an unavailable target:

```text
Permanently delete administrator account?

Account ID
019...

The account is not available through normal account lookup.
If it still exists, this action will permanently delete it.

[Cancel] [Permanently delete account]
```

The frontend must not claim specific dependent-record cleanup beyond the backend contract.

## 7.3 Success

The backend returns:

```text
204 No Content
```

The frontend must:

1. Treat the mutation as successful only after the `204` response.
2. Not fabricate an Account response.
3. Navigate to the validated Account List destination.
4. Allow Account Management to re-fetch authoritative data.
5. Preserve originating Account List query state.

## 7.4 Hard Delete Conflicts

| Status | Code                        | UI Behavior                                                                 |
| ------ | --------------------------- | --------------------------------------------------------------------------- |
| `404`  | `ACCOUNT_NOT_FOUND`         | Explain that the Account no longer exists and return to Account Management. |
| `409`  | `LAST_ACTIVE_ADMINISTRATOR` | Explain that the last active administrator cannot be permanently deleted.   |

The frontend must not retry a failed hard delete automatically.

---

# 8. Confirmation Dialogs

## 8.1 Pattern

Destructive confirmations must use an accessible modal confirmation pattern.

The implementation should prefer the native HTML `<dialog>` element with modal behavior where appropriate.

The confirmation dialog must expose:

```text
role = alertdialog
aria-modal = true
aria-labelledby = confirmation heading
aria-describedby = confirmation message
```

The dialog must prevent interaction with the page behind it while active.

## 8.2 Initial Focus

When the confirmation opens:

```text
Focus → Cancel
```

The least destructive action should receive initial focus.

## 8.3 Keyboard Interaction

| ID                             | Requirement                                                                       |
| ------------------------------ | --------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DELETE-DIALOG-001` | `Tab` keeps keyboard focus within the modal confirmation interaction.             |
| `FE-ACCOUNT-DELETE-DIALOG-002` | `Shift+Tab` keeps keyboard focus within the modal confirmation interaction.       |
| `FE-ACCOUNT-DELETE-DIALOG-003` | `Escape` cancels the confirmation without mutation.                               |
| `FE-ACCOUNT-DELETE-DIALOG-004` | Destructive confirmation requires explicit activation of the destructive control. |

## 8.4 Focus Restoration

When the confirmation is canceled:

```text
Focus → action that opened the confirmation
```

When deletion succeeds and the page navigates away:

```text
Focus → logical destination supplied by Account Management
```

When deletion fails:

```text
Focus → accessible error or usable confirmation/page state
```

The implementation must not leave keyboard focus on an unavailable or removed element.

## 8.5 Pending Dialog State

While deletion is pending:

```text
Deleting…
Permanently deleting…
```

The destructive action must be disabled from duplicate activation.

The dialog must remain in a coherent pending state until the mutation result is known.

---

# 9. API Contract

## 9.1 Authentication

All Account Delete API requests use:

```http
Authorization: Bearer <access-token>
```

Bearer credentials must not be sent in:

```text
URLs
query parameters
request bodies
```

## 9.2 Soft Delete

```http
DELETE /admin/accounts/{id}
Authorization: Bearer <access-token>
Accept: application/json, application/problem+json
```

Permission:

```text
account:delete
```

Success:

```text
204 No Content
```

Errors:

| Status  | Code                        | UI Behavior                                                   |
| ------- | --------------------------- | ------------------------------------------------------------- |
| `400`   | `INVALID_ACCOUNT_ID`        | Show invalid Account identifier error.                        |
| `401`   | —                           | Delegate to shared authentication behavior.                   |
| `403`   | —                           | Show authorization error without logout.                      |
| `404`   | `ACCOUNT_NOT_FOUND`         | Render unavailable Account state.                             |
| `409`   | `ACCOUNT_ALREADY_DELETED`   | Explain that the Account is already deleted.                  |
| `409`   | `LAST_ACTIVE_ADMINISTRATOR` | Explain that the last active administrator cannot be deleted. |
| `500`   | —                           | Show retryable server error.                                  |
| Network | —                           | Show retryable network error.                                 |

## 9.3 Hard Delete

```http
DELETE /admin/accounts/{id}/purge
Authorization: Bearer <access-token>
Accept: application/json, application/problem+json
```

Permission:

```text
account:purge
```

Success:

```text
204 No Content
```

Errors:

| Status  | Code                        | UI Behavior                                                               |
| ------- | --------------------------- | ------------------------------------------------------------------------- |
| `400`   | `INVALID_ACCOUNT_ID`        | Show invalid Account identifier error.                                    |
| `401`   | —                           | Delegate to shared authentication behavior.                               |
| `403`   | —                           | Show authorization error without logout.                                  |
| `404`   | `ACCOUNT_NOT_FOUND`         | Explain that the Account no longer exists.                                |
| `409`   | `LAST_ACTIVE_ADMINISTRATOR` | Explain that the last active administrator cannot be permanently deleted. |
| `500`   | —                           | Show retryable server error.                                              |
| Network | —                           | Show retryable network error.                                             |

## 9.4 Authorization Boundary

The frontend must not enforce:

```text
account:delete
account:purge
```

as security controls.

The backend Authorization domain must authorize every mutation server-side.

Client-held roles or permissions are never sufficient proof of authorization.

A `403 Forbidden` response must preserve the authenticated session unless shared authentication state separately becomes invalid.

## 9.5 Problem Details

Account API errors use:

```text
application/problem+json
```

with RFC 9457 Problem Details semantics.

The frontend should use the structured problem `code` when available.

The frontend must not expose:

```text
stack traces
database errors
internal identifiers
authentication secrets
security diagnostics
```

## 9.6 Cache

Account API responses use:

```http
Cache-Control: no-store
```

The frontend must not persist Account response data in:

```text
localStorage
sessionStorage
IndexedDB
```

---

# 10. Validation and Conflict Handling

## 10.1 Invalid Account ID

For an invalid route ID:

```text
The account identifier is invalid.
```

No destructive control may submit.

## 10.2 Authentication Failure

For `401 Unauthorized`:

```text
Your session is no longer valid.
```

The page delegates to shared authentication recovery.

The Delete page must not implement its own login or token-refresh system.

## 10.3 Authorization Failure

For `403 Forbidden`:

```text
You do not have permission to delete this administrator account.
```

For hard deletion:

```text
You do not have permission to permanently delete this administrator account.
```

The frontend must preserve the authenticated session.

## 10.4 Last Active Administrator

For:

```text
LAST_ACTIVE_ADMINISTRATOR
```

Soft delete:

```text
The last active administrator cannot be deleted.
```

Hard delete:

```text
The last active administrator cannot be permanently deleted.
```

The frontend must not claim that this invariant can be bypassed.

## 10.5 Already Deleted

For:

```text
ACCOUNT_ALREADY_DELETED
```

display:

```text
This administrator account has already been deleted.
```

The frontend must not issue a second soft-delete mutation automatically.

## 10.6 Account Not Found

For:

```text
ACCOUNT_NOT_FOUND
```

display:

```text
This administrator account is no longer available.
```

The page must stop the affected mutation flow and provide navigation back to Account Management.

## 10.7 General Failure

For server or network failure:

```text
The account deletion could not be completed.

[Try Again]
```

The frontend must not claim deletion success until the backend reports success.

---

# 11. UI States

## 11.1 Authentication Bootstrap

```text
Checking authentication…
```

The page must not flash a login screen before shared authentication state resolves.

## 11.2 Loading

```text
Delete Administrator Account

Loading administrator account…
```

No destructive action is enabled before required authoritative state is available.

## 11.3 Ready

```text
Delete Administrator Account

Library Administrator
admin@example.com
Active

[Delete account]
[Permanently delete account]

[Back to Administrator Accounts]
```

## 11.4 Soft Delete Confirmation Open

```text
Delete administrator account?

Library Administrator
admin@example.com

This will deactivate the account and mark it as deleted.
The account can be restored later.

[Cancel] [Delete account]
```

## 11.5 Hard Delete Confirmation Open

```text
Permanently delete administrator account?

Library Administrator
admin@example.com

This action permanently deletes the account and cannot be undone.

[Cancel] [Permanently delete account]
```

## 11.6 Mutation Pending

Soft delete:

```text
Deleting…
```

Hard delete:

```text
Permanently deleting…
```

Duplicate destructive activation must be prevented.

## 11.7 Mutation Success

The page must treat `204 No Content` as success.

The frontend must then navigate to the validated Account List destination and allow Account Management to re-fetch authoritative state.

## 11.8 Authorization Error

```text
You do not have permission to delete this administrator account.
```

The Admin Shell remains active.

## 11.9 Account Unavailable

```text
Administrator Account

This administrator account is no longer available.

[Back to Administrator Accounts]
```

No soft-delete action is available.

A hard-delete action may remain available for a purge-capable user because the backend permits purging soft-deleted targets.

## 11.10 Recoverable Error

```text
The account deletion could not be completed.

[Try Again]
```

The page remains usable and preserves non-sensitive context.

---

# 12. Return Navigation

Account Delete may receive:

```text
return_to
```

as navigation context.

Example:

```text
/admin/accounts/:id/delete?return_to=%2Fadmin%2Faccounts%3Fpage%3D2%26page_size%3D20%26status%3Dactive
```

Supported destination:

```text
/admin/accounts
```

Supported query parameters and values are defined by Account Management:

| Parameter         | Allowed Values               |
| ----------------- | ---------------------------- |
| `page`            | Positive integer             |
| `page_size`       | Positive integer up to `100` |
| `status`          | `active` or `inactive`       |
| `include_deleted` | `true` or omitted            |

The frontend must:

1. Require the same browser origin.
2. Require pathname `/admin/accounts`.
3. Permit only the supported Account List query parameters.
4. Validate supported parameter values.
5. Reject malformed, external, unrelated, or unsupported values.
6. Fall back to `/admin/accounts` when validation fails.
7. Preserve only Account List navigation state.
8. Never treat `return_to` as authorization state.

After successful soft or hard deletion, the page must navigate to the validated return destination.

The return URL must never contain:

```text
access tokens
refresh credentials
passwords
authorization headers
other secrets
```

---

# 13. Responsive Layout

The page inherits Admin Shell responsive behavior and manages only the Main Content layout.

| ID                           | Viewport       | Layout                                                                             |
| ---------------------------- | -------------- | ---------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DELETE-RESP-001` | `>= 1280px`    | Account summary and destructive actions may use a two-column layout.               |
| `FE-ACCOUNT-DELETE-RESP-002` | `768px–1279px` | Use a constrained single-column or two-column layout according to available width. |
| `FE-ACCOUNT-DELETE-RESP-003` | `< 768px`      | Stack Account context and destructive actions vertically.                          |
| `FE-ACCOUNT-DELETE-RESP-004` | All viewports  | Confirmation controls remain independently usable.                                 |
| `FE-ACCOUNT-DELETE-RESP-005` | All viewports  | No unintended page-level horizontal scrolling.                                     |
| `FE-ACCOUNT-DELETE-RESP-006` | All viewports  | Focused controls remain visible.                                                   |
| `FE-ACCOUNT-DELETE-RESP-007` | All viewports  | Admin Shell authentication and navigation behavior remain unchanged.               |

Suggested desktop structure:

```text
┌─────────────────────────────────────────────────────────────┐
│ Back to Administrator Accounts                              │
│                                                             │
│ Delete Administrator Account                                │
│ Library Administrator    admin@example.com    Active        │
│                                                             │
│ ┌───────────────────────────────┐ ┌────────────────────────┐ │
│ │ Account Information           │ │ Destructive Actions    │ │
│ │ Display name                  │ │                        │ │
│ │ Email                         │ │ [Delete account]       │ │
│ │ Account ID                    │ │ [Permanently delete]   │ │
│ │ Status                        │ │                        │ │
│ └───────────────────────────────┘ └────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

Mobile:

```text
Delete Administrator Account

Back to Administrator Accounts

Account Information
Display name
Library Administrator

Email
admin@example.com

Status
Active

[Delete account]
[Permanently delete account]
```

The exact visual styling must use the existing native CSS architecture.

---

# 14. Security

| ID                          | Security Rule            | Requirement                                                                           |
| --------------------------- | ------------------------ | ------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DELETE-SEC-001` | Authentication           | Delete page requires authenticated Admin Shell state.                                 |
| `FE-ACCOUNT-DELETE-SEC-002` | Authentication authority | Backend Authentication remains authoritative.                                         |
| `FE-ACCOUNT-DELETE-SEC-003` | Authorization            | Backend Authorization remains authoritative.                                          |
| `FE-ACCOUNT-DELETE-SEC-004` | Soft delete permission   | Soft deletion requires server-side `account:delete`.                                  |
| `FE-ACCOUNT-DELETE-SEC-005` | Hard delete permission   | Hard deletion requires server-side `account:purge`.                                   |
| `FE-ACCOUNT-DELETE-SEC-006` | Token transport          | Bearer credentials use the `Authorization` header only.                               |
| `FE-ACCOUNT-DELETE-SEC-007` | URL secrecy              | Passwords, tokens, and authentication credentials never appear in URLs.               |
| `FE-ACCOUNT-DELETE-SEC-008` | Browser persistence      | Account representations and credentials are not stored in browser persistence.        |
| `FE-ACCOUNT-DELETE-SEC-009` | Logging                  | Passwords, tokens, Authorization headers, and sensitive deletion data are not logged. |
| `FE-ACCOUNT-DELETE-SEC-010` | Mutation method          | Deletion uses the documented DELETE API endpoints.                                    |
| `FE-ACCOUNT-DELETE-SEC-011` | Navigation safety        | Loading the Delete route never performs a deletion mutation.                          |
| `FE-ACCOUNT-DELETE-SEC-012` | Confirmation             | Destructive mutations require explicit user confirmation.                             |
| `FE-ACCOUNT-DELETE-SEC-013` | Duplicate prevention     | A pending mutation cannot be submitted again.                                         |
| `FE-ACCOUNT-DELETE-SEC-014` | Error exposure           | Internal server diagnostics are never shown to users.                                 |
| `FE-ACCOUNT-DELETE-SEC-015` | Client authorization     | Client-held roles or permissions are never trusted as authorization proof.            |
| `FE-ACCOUNT-DELETE-SEC-016` | Return navigation        | `return_to` accepts only validated same-origin Account List destinations.             |
| `FE-ACCOUNT-DELETE-SEC-017` | Lifecycle authority      | Last-active-administrator protection remains server-enforced.                         |
| `FE-ACCOUNT-DELETE-SEC-018` | Backend cleanup          | Session, token, credential, and role cleanup remains a backend responsibility.        |

The frontend must not implement:

```text
database deletion
database transactions
referential-action cleanup
session revocation
access-token invalidation
refresh-token invalidation
role-assignment cleanup
administrator invariants
```

---

# 15. Accessibility

## 15.1 Semantic Structure

| ID                           | Requirement                                                                      |
| ---------------------------- | -------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DELETE-A11Y-001` | Use semantic `main`, `header`, `section`, headings, links, and buttons.          |
| `FE-ACCOUNT-DELETE-A11Y-002` | Provide exactly one primary `h1`.                                                |
| `FE-ACCOUNT-DELETE-A11Y-003` | Use meaningful headings for Account context and destructive actions.             |
| `FE-ACCOUNT-DELETE-A11Y-004` | Use native interactive elements instead of clickable non-interactive containers. |

## 15.2 Confirmation Dialog

| ID                           | Requirement                                                              |
| ---------------------------- | ------------------------------------------------------------------------ |
| `FE-ACCOUNT-DELETE-A11Y-005` | Confirmation has an accessible name.                                     |
| `FE-ACCOUNT-DELETE-A11Y-006` | Confirmation message is programmatically associated with the dialog.     |
| `FE-ACCOUNT-DELETE-A11Y-007` | Modal confirmation prevents interaction with background content.         |
| `FE-ACCOUNT-DELETE-A11Y-008` | Initial focus is placed on the least destructive action.                 |
| `FE-ACCOUNT-DELETE-A11Y-009` | `Escape` cancels the confirmation without mutation.                      |
| `FE-ACCOUNT-DELETE-A11Y-010` | Focus is restored to the invoking control when confirmation is canceled. |
| `FE-ACCOUNT-DELETE-A11Y-011` | Destructive controls have accessible names that identify the action.     |

## 15.3 Errors and Status

| ID                           | Requirement                                                                      |
| ---------------------------- | -------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DELETE-A11Y-012` | Errors identify the affected operation in text.                                  |
| `FE-ACCOUNT-DELETE-A11Y-013` | Recoverable status messages are programmatically determinable.                   |
| `FE-ACCOUNT-DELETE-A11Y-014` | Pending, success, and recoverable-error states use appropriate status semantics. |
| `FE-ACCOUNT-DELETE-A11Y-015` | Focus is not unnecessarily moved for ordinary status updates.                    |

## 15.4 Focus and Target Size

| ID                           | Requirement                                                                                          |
| ---------------------------- | ---------------------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DELETE-A11Y-016` | Keyboard focus is visibly indicated.                                                                 |
| `FE-ACCOUNT-DELETE-A11Y-017` | Focused controls are not completely obscured by author-created content.                              |
| `FE-ACCOUNT-DELETE-A11Y-018` | Destructive actions satisfy the applicable WCAG 2.2 Target Size requirement or documented exception. |
| `FE-ACCOUNT-DELETE-A11Y-019` | Cancel and destructive actions remain independently targetable.                                      |

## 15.5 Account Identification

The confirmation must provide enough authoritative information to identify the target safely.

When the target is available:

```text
display_name
email
Account ID
```

When the target is unavailable through normal lookup:

```text
Account ID
```

The frontend must not invent missing identifying data.

---

# 16. Component Structure

The page should remain feature-local.

Suggested structure:

```text
src/pages/accounts/
├── AccountListPage
├── AccountCreatePage
├── AccountDetailPage
├── AccountEditPage
└── AccountDeletePage
```

Suggested page-local components:

```text
AccountDeletePage
├── AccountDeleteHeader
├── AccountDeleteSummary
├── AccountDeleteActions
├── AccountDeleteConfirmationDialog
├── AccountDeleteStatus
├── AccountDeleteLoadingState
├── AccountDeleteErrorState
└── AccountDeleteUnavailableState
```

| ID                           | Component                         | Responsibility                                                              |
| ---------------------------- | --------------------------------- | --------------------------------------------------------------------------- |
| `FE-ACCOUNT-DELETE-COMP-001` | `AccountDeletePage`               | Coordinate route state, Account resolution, deletion flows, and navigation. |
| `FE-ACCOUNT-DELETE-COMP-002` | `AccountDeleteHeader`             | Render page heading and navigation context.                                 |
| `FE-ACCOUNT-DELETE-COMP-003` | `AccountDeleteSummary`            | Render authoritative Account summary.                                       |
| `FE-ACCOUNT-DELETE-COMP-004` | `AccountDeleteActions`            | Render soft-delete and hard-delete actions.                                 |
| `FE-ACCOUNT-DELETE-COMP-005` | `AccountDeleteConfirmationDialog` | Render accessible destructive confirmation.                                 |
| `FE-ACCOUNT-DELETE-COMP-006` | `AccountDeleteStatus`             | Render pending, success, and recoverable status messages.                   |
| `FE-ACCOUNT-DELETE-COMP-007` | `AccountDeleteLoadingState`       | Render initial target resolution state.                                     |
| `FE-ACCOUNT-DELETE-COMP-008` | `AccountDeleteErrorState`         | Render route-level and mutation errors.                                     |
| `FE-ACCOUNT-DELETE-COMP-009` | `AccountDeleteUnavailableState`   | Render a target unavailable through normal lookup.                          |

The page must not introduce:

```text
AccountEditForm
AccountLifecycleManager
RoleManagementSection
PasswordHashField
```

API calls remain in:

```text
src/services/
```

Types remain in:

```text
src/types/
```

Routing helpers remain in the existing Account navigation layer.

No page-specific UI framework should be introduced solely for Account Delete.

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

| ID                                | Test                                                                           |
| --------------------------------- | ------------------------------------------------------------------------------ |
| `FE-ACCOUNT-DELETE-TEST-UNIT-001` | Valid Account response initializes the Delete page correctly.                  |
| `FE-ACCOUNT-DELETE-TEST-UNIT-002` | Invalid Account ID produces an invalid identifier state.                       |
| `FE-ACCOUNT-DELETE-TEST-UNIT-003` | Normal `404 ACCOUNT_NOT_FOUND` does not claim that the target is soft-deleted. |
| `FE-ACCOUNT-DELETE-TEST-UNIT-004` | Soft-delete confirmation opens without submitting a mutation.                  |
| `FE-ACCOUNT-DELETE-TEST-UNIT-005` | Cancel closes the soft-delete confirmation without mutation.                   |
| `FE-ACCOUNT-DELETE-TEST-UNIT-006` | Hard-delete confirmation opens without submitting a mutation.                  |
| `FE-ACCOUNT-DELETE-TEST-UNIT-007` | Cancel closes the hard-delete confirmation without mutation.                   |
| `FE-ACCOUNT-DELETE-TEST-UNIT-008` | Soft delete uses only the Account soft-delete endpoint.                        |
| `FE-ACCOUNT-DELETE-TEST-UNIT-009` | Hard delete uses only the Account purge endpoint.                              |
| `FE-ACCOUNT-DELETE-TEST-UNIT-010` | Pending state prevents duplicate soft-delete submission.                       |
| `FE-ACCOUNT-DELETE-TEST-UNIT-011` | Pending state prevents duplicate hard-delete submission.                       |
| `FE-ACCOUNT-DELETE-TEST-UNIT-012` | `204` is treated as successful soft deletion.                                  |
| `FE-ACCOUNT-DELETE-TEST-UNIT-013` | `204` is treated as successful hard deletion.                                  |
| `FE-ACCOUNT-DELETE-TEST-UNIT-014` | `ACCOUNT_ALREADY_DELETED` displays the correct conflict state.                 |
| `FE-ACCOUNT-DELETE-TEST-UNIT-015` | `LAST_ACTIVE_ADMINISTRATOR` displays the correct conflict state.               |
| `FE-ACCOUNT-DELETE-TEST-UNIT-016` | `403` displays authorization feedback without logout.                          |
| `FE-ACCOUNT-DELETE-TEST-UNIT-017` | `401` delegates to shared authentication handling.                             |
| `FE-ACCOUNT-DELETE-TEST-UNIT-018` | `return_to` preserves supported Account List query parameters and values only. |
| `FE-ACCOUNT-DELETE-TEST-UNIT-019` | Invalid `return_to` falls back to `/admin/accounts`.                           |
| `FE-ACCOUNT-DELETE-TEST-UNIT-020` | External `return_to` destinations are rejected.                                |
| `FE-ACCOUNT-DELETE-TEST-UNIT-021` | Confirmation does not expose passwords or tokens.                              |
| `FE-ACCOUNT-DELETE-TEST-UNIT-022` | Confirmation accessible name and description are associated correctly.         |

## 17.2 Integration

| ID                               | Test                                                                    |
| -------------------------------- | ----------------------------------------------------------------------- |
| `FE-ACCOUNT-DELETE-TEST-INT-001` | Authenticated navigation to Account Delete succeeds.                    |
| `FE-ACCOUNT-DELETE-TEST-INT-002` | Account target loads through `GET /admin/accounts/{id}` when available. |
| `FE-ACCOUNT-DELETE-TEST-INT-003` | Soft delete uses `DELETE /admin/accounts/{id}`.                         |
| `FE-ACCOUNT-DELETE-TEST-INT-004` | Hard delete uses `DELETE /admin/accounts/{id}/purge`.                   |
| `FE-ACCOUNT-DELETE-TEST-INT-005` | Soft-delete `204` is handled correctly.                                 |
| `FE-ACCOUNT-DELETE-TEST-INT-006` | Hard-delete `204` is handled correctly.                                 |
| `FE-ACCOUNT-DELETE-TEST-INT-007` | Soft-delete `ACCOUNT_ALREADY_DELETED` is rendered correctly.            |
| `FE-ACCOUNT-DELETE-TEST-INT-008` | Soft-delete `LAST_ACTIVE_ADMINISTRATOR` is rendered correctly.          |
| `FE-ACCOUNT-DELETE-TEST-INT-009` | Hard-delete `LAST_ACTIVE_ADMINISTRATOR` is rendered correctly.          |
| `FE-ACCOUNT-DELETE-TEST-INT-010` | `404 ACCOUNT_NOT_FOUND` is rendered correctly.                          |
| `FE-ACCOUNT-DELETE-TEST-INT-011` | `401` delegates to shared authentication recovery.                      |
| `FE-ACCOUNT-DELETE-TEST-INT-012` | `403` preserves authentication state.                                   |
| `FE-ACCOUNT-DELETE-TEST-INT-013` | Each destructive action uses only its dedicated endpoint.               |
| `FE-ACCOUNT-DELETE-TEST-INT-014` | Duplicate destructive requests are prevented.                           |
| `FE-ACCOUNT-DELETE-TEST-INT-015` | Valid `return_to` navigation preserves Account List state.              |
| `FE-ACCOUNT-DELETE-TEST-INT-016` | Invalid `return_to` navigation falls back safely.                       |
| `FE-ACCOUNT-DELETE-TEST-INT-017` | Account data is not written to browser persistence.                     |
| `FE-ACCOUNT-DELETE-TEST-INT-018` | Passwords and tokens are not placed in URLs or browser state.           |

## 17.3 E2E

| ID                               | Test                                                                |
| -------------------------------- | ------------------------------------------------------------------- |
| `FE-ACCOUNT-DELETE-TEST-E2E-001` | Login → Admin Shell → Accounts → Delete works.                      |
| `FE-ACCOUNT-DELETE-TEST-E2E-002` | Delete route loads the expected administrator Account.              |
| `FE-ACCOUNT-DELETE-TEST-E2E-003` | Soft-delete confirmation opens and does not mutate on cancel.       |
| `FE-ACCOUNT-DELETE-TEST-E2E-004` | Soft deletion completes successfully for an eligible Account.       |
| `FE-ACCOUNT-DELETE-TEST-E2E-005` | Last active administrator cannot be soft-deleted.                   |
| `FE-ACCOUNT-DELETE-TEST-E2E-006` | Already-deleted conflict is displayed correctly.                    |
| `FE-ACCOUNT-DELETE-TEST-E2E-007` | Hard-delete confirmation opens and does not mutate on cancel.       |
| `FE-ACCOUNT-DELETE-TEST-E2E-008` | Hard deletion completes successfully for an eligible Account.       |
| `FE-ACCOUNT-DELETE-TEST-E2E-009` | Hard deletion can target a soft-deleted Account.                    |
| `FE-ACCOUNT-DELETE-TEST-E2E-010` | Last active administrator cannot be hard-deleted.                   |
| `FE-ACCOUNT-DELETE-TEST-E2E-011` | Hard deletion of a missing Account displays unavailable state.      |
| `FE-ACCOUNT-DELETE-TEST-E2E-012` | `401` authentication expiry follows shared recovery behavior.       |
| `FE-ACCOUNT-DELETE-TEST-E2E-013` | `403` preserves authentication and displays authorization feedback. |
| `FE-ACCOUNT-DELETE-TEST-E2E-014` | Successful deletion returns to the originating Account List state.  |
| `FE-ACCOUNT-DELETE-TEST-E2E-015` | Invalid external `return_to` is rejected.                           |
| `FE-ACCOUNT-DELETE-TEST-E2E-016` | Unsupported `return_to` query parameters or values are rejected.    |
| `FE-ACCOUNT-DELETE-TEST-E2E-017` | Duplicate destructive activation is prevented.                      |
| `FE-ACCOUNT-DELETE-TEST-E2E-018` | Escape cancels the confirmation dialog.                             |
| `FE-ACCOUNT-DELETE-TEST-E2E-019` | Confirmation focus starts on the least destructive action.          |
| `FE-ACCOUNT-DELETE-TEST-E2E-020` | Focus returns correctly after cancellation.                         |
| `FE-ACCOUNT-DELETE-TEST-E2E-021` | Mobile layout remains usable below `768px`.                         |
| `FE-ACCOUNT-DELETE-TEST-E2E-022` | Confirmation dialog remains usable on mobile.                       |

## 17.4 Accessibility

| ID                                | Test                                                                                                |
| --------------------------------- | --------------------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DELETE-TEST-A11Y-001` | One primary `h1` exists.                                                                            |
| `FE-ACCOUNT-DELETE-TEST-A11Y-002` | Destructive controls have accessible names.                                                         |
| `FE-ACCOUNT-DELETE-TEST-A11Y-003` | Confirmation dialog has an accessible name.                                                         |
| `FE-ACCOUNT-DELETE-TEST-A11Y-004` | Confirmation message is programmatically associated with the dialog.                                |
| `FE-ACCOUNT-DELETE-TEST-A11Y-005` | Focus is visible.                                                                                   |
| `FE-ACCOUNT-DELETE-TEST-A11Y-006` | Focus is not completely obscured.                                                                   |
| `FE-ACCOUNT-DELETE-TEST-A11Y-007` | Keyboard navigation works for all destructive actions.                                              |
| `FE-ACCOUNT-DELETE-TEST-A11Y-008` | Escape cancels the active confirmation.                                                             |
| `FE-ACCOUNT-DELETE-TEST-A11Y-009` | Pending status is announced appropriately.                                                          |
| `FE-ACCOUNT-DELETE-TEST-A11Y-010` | Error messages are presented as text.                                                               |
| `FE-ACCOUNT-DELETE-TEST-A11Y-011` | Interactive targets satisfy applicable WCAG 2.2 target-size requirements or a documented exception. |
| `FE-ACCOUNT-DELETE-TEST-A11Y-012` | Semantic role and label queries are used where practical.                                           |
| `FE-ACCOUNT-DELETE-TEST-A11Y-013` | Automated accessibility scanning is supplemented by manual keyboard verification.                   |

## 17.5 Responsive

| ID                                | Test                                                        |
| --------------------------------- | ----------------------------------------------------------- |
| `FE-ACCOUNT-DELETE-TEST-RESP-001` | Desktop layout works at `>= 1280px`.                        |
| `FE-ACCOUNT-DELETE-TEST-RESP-002` | Tablet layout works at `768px–1279px`.                      |
| `FE-ACCOUNT-DELETE-TEST-RESP-003` | Mobile layout works below `768px`.                          |
| `FE-ACCOUNT-DELETE-TEST-RESP-004` | No unintended page-level horizontal scrolling occurs.       |
| `FE-ACCOUNT-DELETE-TEST-RESP-005` | Destructive actions remain usable on mobile.                |
| `FE-ACCOUNT-DELETE-TEST-RESP-006` | Confirmation controls remain usable on mobile.              |
| `FE-ACCOUNT-DELETE-TEST-RESP-007` | Focused controls remain visible across supported viewports. |

---

# 18. Implementation Criteria

## 18.1 Route and Navigation

| ID                           | Criteria                                                                                          | Status         | Reason                                                                                  |
| ---------------------------- | ------------------------------------------------------------------------------------------------- | -------------- | --------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DELETE-IMPL-001` | `/admin/accounts/:id/delete` is defined and routed.                                               | 🟢 Implemented | `App.tsx` routes the Delete page and `AccountDeletePage.tsx` implements the route.     |
| `FE-ACCOUNT-DELETE-IMPL-002` | Delete route renders inside the Admin Shell.                                                      | 🟢 Implemented | The route renders through the existing `AdminShell`.                                    |
| `FE-ACCOUNT-DELETE-IMPL-003` | Unauthenticated access follows shared authentication behavior.                                    | 🟢 Implemented | Authentication and protected-route behavior are delegated to the shared auth flow.      |
| `FE-ACCOUNT-DELETE-IMPL-004` | `return_to` is same-origin `/admin/accounts` and uses only supported query parameters and values. | 🟢 Implemented | `resolveAccountReturnTo()` validates origin, pathname, credentials, hash, and queries. |
| `FE-ACCOUNT-DELETE-IMPL-005` | Delete route is represented in the repository sitemap.                                            | 🟡 Defined     | The frontend route exists, but the repository sitemap does not yet define this route. |

## 18.2 Soft Delete

| ID                           | Criteria                                                      | Status         | Reason |
| ---------------------------- | ------------------------------------------------------------- | -------------- | ------ |
| `FE-ACCOUNT-DELETE-IMPL-006` | Soft deletion uses `DELETE /admin/accounts/{id}`.             | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-007` | Soft deletion requires `account:delete` server authorization. | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-008` | Soft deletion requires explicit confirmation.                 | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-009` | Soft-delete `204` is treated as success.                      | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-010` | `ACCOUNT_ALREADY_DELETED` is handled explicitly.              | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-011` | `LAST_ACTIVE_ADMINISTRATOR` is handled explicitly.            | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-012` | Duplicate soft-delete submissions are prevented.              | 🟢 Implemented |        |

## 18.3 Hard Delete

| ID                           | Criteria                                                     | Status         | Reason |
| ---------------------------- | ------------------------------------------------------------ | -------------- | ------ |
| `FE-ACCOUNT-DELETE-IMPL-013` | Hard deletion uses `DELETE /admin/accounts/{id}/purge`.      | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-014` | Hard deletion requires `account:purge` server authorization. | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-015` | Hard deletion requires explicit confirmation.                | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-016` | Hard-delete `204` is treated as success.                     | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-017` | Soft-deleted targets can be permanently deleted.             | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-018` | `LAST_ACTIVE_ADMINISTRATOR` is handled explicitly.           | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-019` | Duplicate hard-delete submissions are prevented.             | 🟢 Implemented |        |

## 18.4 Confirmation and Accessibility

| ID                           | Criteria                                                                   | Status         | Reason |
| ---------------------------- | -------------------------------------------------------------------------- | -------------- | ------ |
| `FE-ACCOUNT-DELETE-IMPL-020` | Confirmation uses an accessible modal pattern.                             | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-021` | Least-destructive initial focus is implemented.                            | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-022` | Escape cancels confirmation without mutation.                              | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-023` | Focus restoration is implemented.                                          | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-024` | Pending, success, and error states use accessible status semantics.        | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-025` | Destructive controls satisfy applicable WCAG 2.2 target-size requirements. | 🟢 Implemented |        |

## 18.5 Security

| ID                           | Criteria                                              | Status         | Reason |
| ---------------------------- | ----------------------------------------------------- | -------------- | ------ |
| `FE-ACCOUNT-DELETE-IMPL-026` | Backend authentication remains authoritative.         | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-027` | Backend authorization remains authoritative.          | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-028` | Tokens do not appear in URLs.                         | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-029` | Account data is not stored in browser persistence.    | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-030` | Deletion is never triggered by GET navigation.        | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-031` | Destructive operations require explicit confirmation. | 🟢 Implemented |        |

## 18.6 Testing

| ID                           | Criteria                                                           | Status         | Reason |
| ---------------------------- | ------------------------------------------------------------------ | -------------- | ------ |
| `FE-ACCOUNT-DELETE-IMPL-032` | Unit coverage exists for deletion state and error handling.        | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-033` | Integration coverage exists for both deletion APIs.                | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-034` | E2E coverage exists for soft and hard deletion.                    | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-035` | Accessibility coverage exists for confirmation and focus behavior. | 🟢 Implemented |        |
| `FE-ACCOUNT-DELETE-IMPL-036` | Responsive coverage exists for supported viewport ranges.          | 🟢 Implemented |        |

---

# 19. Traceability

## Requirements → Design → Backend → Test

| Requirement                 | Frontend Design                    | Backend Contract                    | Test                                    |
| --------------------------- | ---------------------------------- | ----------------------------------- | --------------------------------------- |
| `ADM-AUTH-001`              | `FE-ACCOUNT-DELETE-ROUTE-001`      | Authentication domain               | `FE-ACCOUNT-DELETE-TEST-E2E-001`        |
| `AC_REQ_FC_13`              | `FE-ACCOUNT-DELETE-REQ-002`        | `AC_UC_07`, `AC_API_07`             | `FE-ACCOUNT-DELETE-TEST-INT-003`        |
| `AC_REQ_FC_18`              | `FE-ACCOUNT-DELETE-REQ-011`        | `AC_API_07`, `AC_API_09`            | `FE-ACCOUNT-DELETE-TEST-E2E-005`, `010` |
| `AC_REQ_FC_19`              | `FE-ACCOUNT-DELETE-REQ-003`        | `AC_UC_09`, `AC_API_09`             | `FE-ACCOUNT-DELETE-TEST-E2E-008`        |
| `AC_API_07`                 | `FE-ACCOUNT-DELETE-SCOPE-004`      | `DELETE /admin/accounts/{id}`       | `FE-ACCOUNT-DELETE-TEST-INT-003`        |
| `AC_API_09`                 | `FE-ACCOUNT-DELETE-SCOPE-005`      | `DELETE /admin/accounts/{id}/purge` | `FE-ACCOUNT-DELETE-TEST-INT-004`        |
| `account:delete`            | `FE-ACCOUNT-DELETE-SEC-004`        | Authorization domain                | `FE-ACCOUNT-DELETE-TEST-INT-003`        |
| `account:purge`             | `FE-ACCOUNT-DELETE-SEC-005`        | Authorization domain                | `FE-ACCOUNT-DELETE-TEST-INT-004`        |
| `ACCOUNT_ALREADY_DELETED`   | `FE-ACCOUNT-DELETE-REQ-010`        | `AC_API_07`                         | `FE-ACCOUNT-DELETE-TEST-INT-007`        |
| `LAST_ACTIVE_ADMINISTRATOR` | `FE-ACCOUNT-DELETE-REQ-011`        | `AC_API_07`, `AC_API_09`            | `FE-ACCOUNT-DELETE-TEST-E2E-005`, `010` |
| Authentication `401`        | `FE-ACCOUNT-DELETE-REQ-007`        | Authentication domain               | `FE-ACCOUNT-DELETE-TEST-E2E-012`        |
| Authorization `403`         | `FE-ACCOUNT-DELETE-REQ-008`        | Authorization domain                | `FE-ACCOUNT-DELETE-TEST-E2E-013`        |
| Account List return state   | `FE-ACCOUNT-DELETE-REQ-014`        | Account Management                  | `FE-ACCOUNT-DELETE-TEST-E2E-014`        |
| WCAG confirmation behavior  | `FE-ACCOUNT-DELETE-A11Y-005`–`011` | WAI-ARIA APG                        | `FE-ACCOUNT-DELETE-TEST-A11Y-003`–`009` |

## Domain References

| Domain             | Relevant References                                                          |
| ------------------ | ---------------------------------------------------------------------------- |
| Requirements       | `ADM-AUTH-001`                                                               |
| Sitemap            | `/admin/accounts/:id/delete`                                                 |
| Account            | `AC_UC_07`, `AC_UC_09`, `AC_API_07`, `AC_API_09`                             |
| Authentication     | Protected route behavior, bearer authentication, `401` handling, HTTPS/TLS   |
| Authorization      | `account:delete`, `account:purge`, deny-by-default, `403`                    |
| Admin Shell        | Authentication state, layout, navigation, responsive behavior                |
| Account Management | Account List, lifecycle actions, query-state preservation, Delete navigation |
| Account Detail     | Delete navigation into Account Delete                                        |
| Account Edit       | Delete actions remain outside Account Edit                                   |

## Cross-Document Dependencies

```text
../../sitemap.md
    → /admin/accounts/:id/delete
    → sitemap alignment required

./admin_account_management_page.md
    → owns Account List
    → owns lifecycle actions
    → owns Delete navigation
    → provides originating Account List query state

./admin_account_detail_page.md
    → provides Delete Account navigation
    → remains read-only
    → does not execute deletion mutations

./admin_account_edit_page.md
    → does not provide deletion controls
    → leaves deletion to Account Delete

../account.md
    → owns soft-delete contract
    → owns hard-delete contract
    → owns lifecycle invariants
    → owns Account Response and error contracts

../authentication.md
    → owns authentication state
    → owns bearer token lifecycle
    → owns session behavior

../authorization.md
    → owns permission evaluation
    → owns `account:delete`
    → owns `account:purge`
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
Account Delete
    ├── Soft Delete
    │      ↓
    │   AC_API_07
    │
    └── Hard Delete
           ↓
        AC_API_09
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

The Account Delete page follows the repository Admin Application stack:

```text
React 19.3
Vite 8
TypeScript 5
Native CSS
Vitest
Playwright
Testing Library
```

## Official and Industry Guidance

| ID                          | Reference                              | Application                                                                                                                      |
| --------------------------- | -------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| `FE-ACCOUNT-DELETE-STD-001` | React DOM Components                   | Use native browser links, buttons, and dialog-capable HTML elements through React.                                               |
| `FE-ACCOUNT-DELETE-STD-002` | React `useRef` / `useEffect` / `useId` | Use refs for dialog DOM interaction and unique IDs for accessible relationships where required.                                  |
| `FE-ACCOUNT-DELETE-STD-003` | Vite TypeScript workflow               | Keep type checking in the explicit `tsc --noEmit` workflow.                                                                      |
| `FE-ACCOUNT-DELETE-STD-004` | TypeScript Handbook                    | Keep route state, Account models, API results, and deletion state explicitly typed.                                              |
| `FE-ACCOUNT-DELETE-STD-005` | WCAG 2.2                               | Apply focus visibility, error identification, status messaging, and target-size requirements.                                    |
| `FE-ACCOUNT-DELETE-STD-006` | WAI-ARIA APG Alert Dialog Pattern      | Use accessible names, descriptions, modal behavior, keyboard cancellation, and safe initial focus for destructive confirmations. |
| `FE-ACCOUNT-DELETE-STD-007` | HTML `<dialog>` / `HTMLDialogElement`  | Prefer native modal dialog behavior where appropriate.                                                                           |
| `FE-ACCOUNT-DELETE-STD-008` | RFC 9110 HTTP Semantics                | Use DELETE semantics and handle documented `204`, `401`, `403`, `404`, and `409` responses.                                      |
| `FE-ACCOUNT-DELETE-STD-009` | RFC 9457 Problem Details               | Consume structured problem details and machine-readable error codes.                                                             |
| `FE-ACCOUNT-DELETE-STD-010` | OWASP Authorization guidance           | Maintain least privilege, deny-by-default, and server-side authorization enforcement.                                            |
| `FE-ACCOUNT-DELETE-STD-011` | Testing Library query guidance         | Prefer semantic queries such as roles and accessible names.                                                                      |
| `FE-ACCOUNT-DELETE-STD-012` | Playwright accessibility guidance      | Combine automated accessibility checks with manual accessibility verification.                                                   |

## Normative Rules

The Account Delete page must:

1. Use the existing React, Vite, TypeScript, and native CSS architecture.
2. Keep soft deletion and hard deletion as separate backend operations.
3. Require explicit confirmation before every destructive mutation.
4. Use operation-specific confirmation language.
5. Treat backend authentication, authorization, and Account lifecycle invariants as authoritative.
6. Never perform deletion through GET navigation.
7. Handle `204 No Content` without fabricating an Account response.
8. Handle `LAST_ACTIVE_ADMINISTRATOR` as a backend-enforced conflict.
9. Preserve safe Account List return navigation.
10. Never put credentials or secrets in URLs.
11. Never persist Account representations in browser storage.
12. Use accessible modal confirmation behavior.
13. Place initial confirmation focus on the least destructive action.
14. Restore focus appropriately when confirmation is canceled.
15. Provide accessible text for errors and operation status.
16. Keep destructive controls usable across supported viewport sizes.
17. Prefer semantic Testing Library queries.
18. Supplement automated accessibility testing with manual keyboard verification.

## Implementation Status Boundary

The current frontend branch contains Account Management, Account Detail, and Account Edit implementations, but it does not currently contain the dedicated Account Delete page implementation.

This document therefore defines the Account Delete page as the frontend source of truth for:

```text
/admin/accounts/:id/delete
```

Implementation status remains tracked separately in Section 18.

The repository sitemap must be updated to represent the Delete route before the route is considered repository-complete.
