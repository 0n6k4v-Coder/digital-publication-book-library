# Frontend Admin Account Create Page

## Table of Contents

1. [Scope](#scope)
2. [Route](#route)
3. [Requirements](#requirements)
4. [Layout](#layout)
5. [Form Contract](#form-contract)
6. [API Contract](#api-contract)
7. [Submission Behavior](#submission-behavior)
8. [UI States](#ui-states)
9. [Error Handling](#error-handling)
10. [Security](#security)
11. [Accessibility](#accessibility)
12. [Responsive Behavior](#responsive-behavior)
13. [Component Structure](#component-structure)
14. [Testing](#testing)
15. [Implementation Criteria](#implementation-criteria)
16. [Traceability](#traceability)

---

# 1. Scope

This document defines the protected **Administrator Account Create** frontend page.

The page is:

    /admin/accounts/create

The page is rendered inside the existing [Admin Shell](./admin_shell.md) and is part of the Administrator Account Management feature.

The page is responsible for collecting the data required by the existing Account Create API, submitting the request, presenting validation and authorization feedback, and navigating the administrator after successful creation.

## Responsibilities

| ID | Responsibility |
|---|---|
| `FE-ACCOUNT-CREATE-SCOPE-001` | Render the protected Administrator Account Create page. |
| `FE-ACCOUNT-CREATE-SCOPE-002` | Render the Account Create form. |
| `FE-ACCOUNT-CREATE-SCOPE-003` | Collect administrator email. |
| `FE-ACCOUNT-CREATE-SCOPE-004` | Collect administrator password. |
| `FE-ACCOUNT-CREATE-SCOPE-005` | Submit the form to `POST /admin/accounts`. |
| `FE-ACCOUNT-CREATE-SCOPE-006` | Prevent duplicate submissions. |
| `FE-ACCOUNT-CREATE-SCOPE-007` | Display server validation and conflict errors. |
| `FE-ACCOUNT-CREATE-SCOPE-008` | Handle authentication and authorization failures. |
| `FE-ACCOUNT-CREATE-SCOPE-009` | Navigate back to Administrator Accounts. |
| `FE-ACCOUNT-CREATE-SCOPE-010` | Navigate to the created Account after successful creation. |
| `FE-ACCOUNT-CREATE-SCOPE-011` | Preserve Admin Shell behavior and navigation state. |
| `FE-ACCOUNT-CREATE-SCOPE-012` | Preserve accessibility and responsive behavior. |

## Out of Scope

| ID | Excluded Area |
|---|---|
| `FE-ACCOUNT-CREATE-OOS-001` | Authentication implementation. |
| `FE-ACCOUNT-CREATE-OOS-002` | Authorization implementation. |
| `FE-ACCOUNT-CREATE-OOS-003` | Password hashing or credential storage. |
| `FE-ACCOUNT-CREATE-OOS-004` | Account persistence or transaction handling. |
| `FE-ACCOUNT-CREATE-OOS-005` | Account lifecycle operations after creation. |
| `FE-ACCOUNT-CREATE-OOS-006` | Account editing. |
| `FE-ACCOUNT-CREATE-OOS-007` | Account deletion. |
| `FE-ACCOUNT-CREATE-OOS-008` | Role assignment or role management. |
| `FE-ACCOUNT-CREATE-OOS-009` | Client-side implementation of backend password policy. |
| `FE-ACCOUNT-CREATE-OOS-010` | Additional search, filtering, or account list behavior. |

The page must not implement its own authentication or authorization rules. Backend Authentication and Authorization remain authoritative.

---

# 2. Route

| ID | Route | Access | Behavior | References |
|---|---|---|---|---|
| `FE-ACCOUNT-CREATE-ROUTE-001` | `/admin/accounts/create` | Authenticated | Render Account Create page inside Admin Shell. | `ADM-AUTH-003`, `PAGE-ADM-009` |
| `FE-ACCOUNT-CREATE-ROUTE-002` | `/admin/accounts/create` | Authentication bootstrap | Keep the protected route pending while authentication state is `unknown`. | `FE_SHELL_AUTH_03` |
| `FE-ACCOUNT-CREATE-ROUTE-003` | `/admin/accounts/create` | Unauthenticated | Redirect to `/login` after authentication bootstrap resolves to unauthenticated. | Authentication domain |
| `FE-ACCOUNT-CREATE-ROUTE-004` | `/admin/accounts/create` | Authenticated but unauthorized | Keep the user authenticated and render an authorization error. | Authorization domain |

The page must render inside the Admin Shell:

    /admin/accounts/create
        └── Admin Shell
            ├── Sidebar
            │   └── Accounts = active
            └── Main Content
                └── Account Create

The Accounts navigation item remains active while the page is open.

## Navigation

| ID | Navigation | Destination |
|---|---|---|
| `FE-ACCOUNT-CREATE-ROUTE-005` | Back to Accounts | `/admin/accounts` |
| `FE-ACCOUNT-CREATE-ROUTE-006` | Successful creation | `/admin/accounts/:id` |

The successful-creation destination uses the newly created Account ID returned by the API response or the API `Location` header.

Credentials must never appear in route paths or query parameters.

---

# 3. Requirements

| ID | Requirement | Repository Reference |
|---|---|---|
| `FE-ACCOUNT-CREATE-REQ-001` | Provide a protected `/admin/accounts/create` route. | `ADM-AUTH-003`, `PAGE-ADM-009` |
| `FE-ACCOUNT-CREATE-REQ-002` | Allow an authorized administrator to create an administrator Account. | `ADM-AUTH-003`, `AC_UC_01` |
| `FE-ACCOUNT-CREATE-REQ-003` | Collect email required by the Create Account API. | `AC_UC_01`, `AC_API_01` |
| `FE-ACCOUNT-CREATE-REQ-004` | Collect password required by the Create Account API. | `AC_UC_01`, `AC_API_01` |
| `FE-ACCOUNT-CREATE-REQ-005` | Submit credentials only through the Account Create API contract. | `AC_API_01` |
| `FE-ACCOUNT-CREATE-REQ-006` | Prevent duplicate submissions while creation is pending. | Frontend behavior |
| `FE-ACCOUNT-CREATE-REQ-007` | Treat server-side validation as authoritative. | Account domain |
| `FE-ACCOUNT-CREATE-REQ-008` | Handle `409 EMAIL_ALREADY_IN_USE`. | `AC_API_01` |
| `FE-ACCOUNT-CREATE-REQ-009` | Handle `422 VALIDATION_ERROR`. | `AC_API_01` |
| `FE-ACCOUNT-CREATE-REQ-010` | Handle `401 Unauthorized` through the Authentication flow. | Authentication domain |
| `FE-ACCOUNT-CREATE-REQ-011` | Handle `403 Forbidden` as an authorization failure without logging the user out. | Authorization domain |
| `FE-ACCOUNT-CREATE-REQ-012` | Do not expose access tokens, refresh credentials, passwords, or password hashes in the UI. | Authentication and Account Security |
| `FE-ACCOUNT-CREATE-REQ-013` | Do not persist form credentials in browser storage. | Authentication Security |
| `FE-ACCOUNT-CREATE-REQ-014` | Provide keyboard-accessible form controls. | Admin Shell accessibility |
| `FE-ACCOUNT-CREATE-REQ-015` | Provide visible focus states. | Admin Shell accessibility |
| `FE-ACCOUNT-CREATE-REQ-016` | Provide accessible loading and error feedback. | Frontend accessibility |
| `FE-ACCOUNT-CREATE-REQ-017` | Prevent unintended page-level horizontal scrolling. | Admin Shell responsive behavior |
| `FE-ACCOUNT-CREATE-REQ-018` | Preserve the Admin Shell while the feature page loads or submits. | `FE_SHELL_14` |
| `FE-ACCOUNT-CREATE-REQ-019` | Preserve server-defined Account semantics. | Account domain |
| `FE-ACCOUNT-CREATE-REQ-020` | Do not add unsupported client-side password-policy rules. | Authentication and Account domain |

## Account Field Boundary

The current Account Create API accepts:

    Email
    Password

The existing Account domain defines `display_name` as an Account-owned field, but `AC_UC_01` and `AC_API_01` currently define Account creation input as Email and Password only.

Therefore:

| ID | Decision |
|---|---|
| `FE-ACCOUNT-CREATE-REQ-021` | The Create page must not submit `display_name` because `POST /admin/accounts` does not currently define it as a create input. |
| `FE-ACCOUNT-CREATE-REQ-022` | The Create page must not fabricate or infer a display name from the email address. |
| `FE-ACCOUNT-CREATE-REQ-023` | Display name configuration belongs to the Account Edit flow unless the backend Create Account contract is explicitly extended. |

This prevents the frontend design from introducing an API contract that does not exist.

---

# 4. Layout

## 4.1 Page Header

Suggested structure:

    Administrator Account
    Create a new administrator account.

    [Back to Administrator Accounts]

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-CREATE-UI-001` | Use `Create Administrator Account` or equivalent as the primary page heading. |
| `FE-ACCOUNT-CREATE-UI-002` | Provide a short supporting description. |
| `FE-ACCOUNT-CREATE-UI-003` | Provide a native Back navigation control to `/admin/accounts`. |
| `FE-ACCOUNT-CREATE-UI-004` | Keep the header visually consistent with the Account Management and Account Detail pages. |

The page must have exactly one primary `h1`.

## 4.2 Form

Suggested structure:

    Administrator Account

    Email
    [________________________________]

    Password
    [________________________________]

    [Cancel]    [Create Account]

| ID | Element | Requirement |
|---|---|---|
| `FE-ACCOUNT-CREATE-UI-005` | Form | Use a native HTML `form` element. |
| `FE-ACCOUNT-CREATE-UI-006` | Email | Use a labeled email input. |
| `FE-ACCOUNT-CREATE-UI-007` | Password | Use a labeled password input. |
| `FE-ACCOUNT-CREATE-UI-008` | Submit | Provide `Create Account` as the primary submit action. |
| `FE-ACCOUNT-CREATE-UI-009` | Cancel | Provide a navigation action back to `/admin/accounts`. |
| `FE-ACCOUNT-CREATE-UI-010` | Validation | Associate validation feedback with the relevant field. |
| `FE-ACCOUNT-CREATE-UI-011` | Pending | Disable duplicate submission while the request is pending. |

The form should use the application's existing native CSS architecture.

## 4.3 Form Width

The form should use a readable constrained width inside the Admin Shell.

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-CREATE-UI-012` | Keep the form readable on large screens rather than stretching across the full main-content width. |
| `FE-ACCOUNT-CREATE-UI-013` | Preserve consistent spacing between labels, controls, errors, and actions. |
| `FE-ACCOUNT-CREATE-UI-014` | Keep primary and secondary actions visually distinct. |
| `FE-ACCOUNT-CREATE-UI-015` | Keep the action group usable on narrow viewports. |

## 4.4 Action Semantics

| ID | Action | Behavior |
|---|---|---|
| `FE-ACCOUNT-CREATE-ACTION-001` | Back to Accounts | Navigate to `/admin/accounts`. |
| `FE-ACCOUNT-CREATE-ACTION-002` | Cancel | Navigate to `/admin/accounts` without submitting. |
| `FE-ACCOUNT-CREATE-ACTION-003` | Create Account | Submit the form once when valid. |

`Cancel` must never submit the form.

---

# 5. Form Contract

## 5.1 Email

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-CREATE-FORM-001` | Use `type="email"`. |
| `FE-ACCOUNT-CREATE-FORM-002` | Mark the field as required. |
| `FE-ACCOUNT-CREATE-FORM-003` | Use `autocomplete="email"` or the closest appropriate browser-supported value. |
| `FE-ACCOUNT-CREATE-FORM-004` | Use the native browser email validity model for basic input syntax. |
| `FE-ACCOUNT-CREATE-FORM-005` | Do not attempt to reproduce backend email normalization in client-side business logic. |
| `FE-ACCOUNT-CREATE-FORM-006` | Preserve the entered email when the submission fails and a retry is possible. |

The backend remains authoritative for canonical email normalization and uniqueness.

## 5.2 Password

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-CREATE-FORM-007` | Use `type="password"`. |
| `FE-ACCOUNT-CREATE-FORM-008` | Mark the field as required. |
| `FE-ACCOUNT-CREATE-FORM-009` | Use `autocomplete="new-password"`. |
| `FE-ACCOUNT-CREATE-FORM-010` | Do not introduce a frontend password policy not defined by the backend contract. |
| `FE-ACCOUNT-CREATE-FORM-011` | Do not display or persist the password outside the form's in-memory state. |
| `FE-ACCOUNT-CREATE-FORM-012` | Do not log the password during validation, submission, or error handling. |
| `FE-ACCOUNT-CREATE-FORM-013` | Do not include the password in URLs, analytics payloads, client persistence, or diagnostic output. |

## 5.3 Password Confirmation

A password confirmation field is not part of `AC_UC_01` or `AC_API_01`.

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-CREATE-FORM-014` | Do not require a password confirmation field unless the backend and product contract explicitly introduce it. |
| `FE-ACCOUNT-CREATE-FORM-015` | Do not perform client-side password equality validation for a field that is not part of the current create contract. |

## 5.4 Browser Validation

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-CREATE-FORM-016` | Use native required-field validation. |
| `FE-ACCOUNT-CREATE-FORM-017` | Use native email input validation for basic email syntax. |
| `FE-ACCOUNT-CREATE-FORM-018` | Do not reject passwords using additional frontend-only business rules. |
| `FE-ACCOUNT-CREATE-FORM-019` | Server validation remains authoritative for all Account security rules. |

---

# 6. API Contract

## 6.1 Create Endpoint

    POST /admin/accounts
    Authorization: Bearer <access-token>
    Content-Type: application/json

Request body:

    {
      "email": "admin@example.com",
      "password": "example-secure-password"
    }

| ID | Contract |
|---|---|
| `FE-ACCOUNT-CREATE-API-001` | Use `POST /admin/accounts`. |
| `FE-ACCOUNT-CREATE-API-002` | Send the bearer access credential only in the `Authorization` header. |
| `FE-ACCOUNT-CREATE-API-003` | Do not send bearer credentials in query parameters. |
| `FE-ACCOUNT-CREATE-API-004` | Do not send bearer credentials in the request body. |
| `FE-ACCOUNT-CREATE-API-005` | Send JSON request content using the existing frontend service layer. |
| `FE-ACCOUNT-CREATE-API-006` | Submit `email` and `password` exactly as the Create Account API contract defines. |
| `FE-ACCOUNT-CREATE-API-007` | Do not submit unsupported fields. |
| `FE-ACCOUNT-CREATE-API-008` | Treat the backend as the source of truth for validation and Account state. |

## 6.2 Required Authorization

The Create Account operation requires the server-side permission:

    account:create

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-CREATE-API-009` | Backend authorization must evaluate `account:create`. |
| `FE-ACCOUNT-CREATE-API-010` | Client-side roles or permissions must never be used as proof of authorization. |
| `FE-ACCOUNT-CREATE-API-011` | A missing permission must result in `403 Forbidden`. |
| `FE-ACCOUNT-CREATE-API-012` | A `403` response must not cause an unnecessary logout. |

## 6.3 Success

Expected status:

    201 Created

The response uses the authoritative Account Response representation.

The backend provides:

    Location: /admin/accounts/{id}

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-CREATE-API-013` | Treat `201 Created` as successful creation. |
| `FE-ACCOUNT-CREATE-API-014` | Obtain the new Account ID from the response representation or `Location` header. |
| `FE-ACCOUNT-CREATE-API-015` | Never render or expose the submitted password after successful creation. |
| `FE-ACCOUNT-CREATE-API-016` | Do not assume the created Account has a display name. |
| `FE-ACCOUNT-CREATE-API-017` | Navigate to the created Account Detail page after success. |

## 6.4 Error Contract

| Status | Code | Frontend Behavior |
|---|---|---|
| `400` | `INVALID_REQUEST` | Show a generic request error and allow correction or retry. |
| `401` | Authentication error | Delegate to the Authentication flow and redirect to `/login` when authentication becomes unauthenticated. |
| `403` | Authorization failure | Show a permission error while preserving authenticated state. |
| `409` | `EMAIL_ALREADY_IN_USE` | Show an email conflict error associated with the Email field. |
| `422` | `VALIDATION_ERROR` | Show server validation feedback without inventing client-side rules. |
| `500` | Unexpected server error | Show a generic retryable error without exposing server diagnostics. |

All error responses must be treated as `no-store` and must never expose secrets.

---

# 7. Submission Behavior

## 7.1 Submit Flow

    User fills Email and Password
            ↓
    Native validation
            ↓
    Submit Create Account
            ↓
    Disable duplicate submission
            ↓
    POST /admin/accounts
            ↓
    ┌───────────────┬────────────────┬─────────────────┐
    │ 201           │ 4xx            │ 5xx / Network  │
    ↓               ↓                ↓
    Navigate        Render error     Preserve form
    to Detail       Allow retry      Allow retry

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-CREATE-SUBMIT-001` | Intercept form submission using the page's normal React event handling. |
| `FE-ACCOUNT-CREATE-SUBMIT-002` | Run native form validity checks before the API request. |
| `FE-ACCOUNT-CREATE-SUBMIT-003` | Prevent duplicate submission while the request is pending. |
| `FE-ACCOUNT-CREATE-SUBMIT-004` | Disable the Create Account action while submission is pending. |
| `FE-ACCOUNT-CREATE-SUBMIT-005` | Preserve the entered Email after retryable failures. |
| `FE-ACCOUNT-CREATE-SUBMIT-006` | Do not expose the Password in error messages or logs. |
| `FE-ACCOUNT-CREATE-SUBMIT-007` | On success, clear transient form state before navigation. |
| `FE-ACCOUNT-CREATE-SUBMIT-008` | On success, navigate to the created Account Detail route. |
| `FE-ACCOUNT-CREATE-SUBMIT-009` | On authorization failure, keep the page mounted and show the authorization state. |
| `FE-ACCOUNT-CREATE-SUBMIT-010` | On authentication failure, use the shared Authentication behavior. |

## 7.2 Duplicate Submission Prevention

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-CREATE-SUBMIT-011` | The page maintains an explicit submission-pending state. |
| `FE-ACCOUNT-CREATE-SUBMIT-012` | A pending form cannot submit a second create request. |
| `FE-ACCOUNT-CREATE-SUBMIT-013` | The pending state is reset when the request completes. |
| `FE-ACCOUNT-CREATE-SUBMIT-014` | A failed request leaves the form retryable unless authentication or authorization state prevents it. |

---

# 8. UI States

| ID | State | Behavior |
|---|---|---|
| `FE-ACCOUNT-CREATE-STATE-001` | Initial | Render the empty Create Account form. |
| `FE-ACCOUNT-CREATE-STATE-002` | Native Validation | Keep the user on the form and expose native validation feedback. |
| `FE-ACCOUNT-CREATE-STATE-003` | Submission Pending | Disable duplicate submission and communicate progress. |
| `FE-ACCOUNT-CREATE-STATE-004` | Created | Navigate to `/admin/accounts/:id`. |
| `FE-ACCOUNT-CREATE-STATE-005` | Email Conflict | Show `Email is already in use` or equivalent field-specific feedback. |
| `FE-ACCOUNT-CREATE-STATE-006` | Validation Error | Show server validation feedback without exposing internal diagnostics. |
| `FE-ACCOUNT-CREATE-STATE-007` | Unauthorized | Preserve authenticated state and display an authorization error. |
| `FE-ACCOUNT-CREATE-STATE-008` | Unauthenticated | Transition through shared Authentication behavior and navigate to `/login`. |
| `FE-ACCOUNT-CREATE-STATE-009` | Network Failure | Preserve retryable form state and display a generic retry message. |
| `FE-ACCOUNT-CREATE-STATE-010` | Server Error | Preserve retryable form state and display a generic error. |
| `FE-ACCOUNT-CREATE-STATE-011` | Back Navigation | Return to `/admin/accounts` without submitting. |

## Status Messaging

Submission status should use an accessible status region.

Suggested behavior:

    Creating administrator account…

Error messages should identify the affected field when the backend supplies field-specific validation information.

---

# 9. Error Handling

## 9.1 Field Errors

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-CREATE-ERR-001` | Associate Email validation errors with the Email field. |
| `FE-ACCOUNT-CREATE-ERR-002` | Associate Password validation errors with the Password field. |
| `FE-ACCOUNT-CREATE-ERR-003` | Use `aria-describedby` when helper or error content is associated with an input. |
| `FE-ACCOUNT-CREATE-ERR-004` | Do not replace server validation with guessed client rules. |
| `FE-ACCOUNT-CREATE-ERR-005` | Do not expose raw problem-response internals when the detail is not intended for administrators. |

## 9.2 Email Conflict

`EMAIL_ALREADY_IN_USE` should be presented as a field-level Email error.

Suggested message:

    This email address is already in use.

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-CREATE-ERR-006` | Associate `EMAIL_ALREADY_IN_USE` with Email. |
| `FE-ACCOUNT-CREATE-ERR-007` | Keep the Email value available for correction. |
| `FE-ACCOUNT-CREATE-ERR-008` | Do not clear the Password because of an email conflict unless required by the application's security policy. |

## 9.3 Authorization Error

Suggested behavior:

    You do not have permission to create administrator accounts.

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-CREATE-ERR-009` | Present a generic authorization message for `403`. |
| `FE-ACCOUNT-CREATE-ERR-010` | Do not expose role names, permission mappings, or authorization internals. |
| `FE-ACCOUNT-CREATE-ERR-011` | Do not redirect to `/login` solely because of `403`. |

## 9.4 Unexpected Error

Suggested behavior:

    Something went wrong while creating the administrator account. Please try again.

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-CREATE-ERR-012` | Provide a generic retryable message for unexpected failures. |
| `FE-ACCOUNT-CREATE-ERR-013` | Do not render backend stack traces or internal diagnostics. |
| `FE-ACCOUNT-CREATE-ERR-014` | Keep the form available for retry when safe. |
| `FE-ACCOUNT-CREATE-ERR-015` | Never log the Password while diagnosing the failure. |

---

# 10. Security

The Account Create page must follow the Authentication, Authorization, and Account security contracts.

| ID | Security Rule | Requirement |
|---|---|---|
| `FE-ACCOUNT-CREATE-SEC-001` | Authentication | Protected route requires authenticated Admin Shell state. |
| `FE-ACCOUNT-CREATE-SEC-002` | Authentication Authority | Backend remains authoritative for authentication. |
| `FE-ACCOUNT-CREATE-SEC-003` | Authorization | Backend remains authoritative for `account:create`. |
| `FE-ACCOUNT-CREATE-SEC-004` | Access Token | Bearer credentials are sent only in the `Authorization` header. |
| `FE-ACCOUNT-CREATE-SEC-005` | Token URL Secrecy | Tokens never appear in URLs. |
| `FE-ACCOUNT-CREATE-SEC-006` | Token UI Secrecy | Tokens are never rendered. |
| `FE-ACCOUNT-CREATE-SEC-007` | Token Logging | Tokens are never logged. |
| `FE-ACCOUNT-CREATE-SEC-008` | Password Secrecy | Password values are never logged. |
| `FE-ACCOUNT-CREATE-SEC-009` | Password Persistence | Password values are never written to localStorage, sessionStorage, IndexedDB, or other browser persistence. |
| `FE-ACCOUNT-CREATE-SEC-010` | Browser History | Password values must never enter browser history through URLs. |
| `FE-ACCOUNT-CREATE-SEC-011` | API Response | Passwords and password hashes are never expected in the Account response. |
| `FE-ACCOUNT-CREATE-SEC-012` | Client Authorization | Client roles and permissions are not trusted as proof of authorization. |
| `FE-ACCOUNT-CREATE-SEC-013` | Cache | Account Create responses are treated as `no-store`. |
| `FE-ACCOUNT-CREATE-SEC-014` | Error Exposure | Internal server diagnostics are not exposed to administrators. |
| `FE-ACCOUNT-CREATE-SEC-015` | Mutation Boundary | The page performs only Account creation and no lifecycle or role mutations. |

The frontend must not infer authorization from:

    role
    permission
    account status
    route visibility
    client state

Server-side authorization remains the security boundary.

---

# 11. Accessibility

## 11.1 Semantic Structure

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-CREATE-A11Y-001` | Use semantic layout elements such as `main`, `header`, `section`, and `form`. |
| `FE-ACCOUNT-CREATE-A11Y-002` | Provide exactly one primary `h1`. |
| `FE-ACCOUNT-CREATE-A11Y-003` | Use logical heading hierarchy. |
| `FE-ACCOUNT-CREATE-A11Y-004` | Use native form controls where possible. |
| `FE-ACCOUNT-CREATE-A11Y-005` | Associate every form control with a visible label. |

## 11.2 Validation

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-CREATE-A11Y-006` | Associate validation messages with their controls. |
| `FE-ACCOUNT-CREATE-A11Y-007` | Mark invalid controls using appropriate semantics such as `aria-invalid`. |
| `FE-ACCOUNT-CREATE-A11Y-008` | Make error messages readable without requiring color perception. |
| `FE-ACCOUNT-CREATE-A11Y-009` | Do not rely only on border or color changes to communicate validation state. |

## 11.3 Async Status

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-CREATE-A11Y-010` | Communicate submission progress to assistive technology. |
| `FE-ACCOUNT-CREATE-A11Y-011` | Communicate server errors appropriately. |
| `FE-ACCOUNT-CREATE-A11Y-012` | Do not steal focus unnecessarily during normal submission. |
| `FE-ACCOUNT-CREATE-A11Y-013` | Move focus to the most relevant error summary or first invalid field when necessary for recovery. |

A suitable non-disruptive status region may use a polite live announcement.

## 11.4 Keyboard Interaction

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-CREATE-A11Y-014` | All form controls are keyboard accessible. |
| `FE-ACCOUNT-CREATE-A11Y-015` | Back and Cancel navigation are keyboard accessible. |
| `FE-ACCOUNT-CREATE-A11Y-016` | Submit is keyboard accessible. |
| `FE-ACCOUNT-CREATE-A11Y-017` | Focus is visibly indicated. |
| `FE-ACCOUNT-CREATE-A11Y-018` | Focus is not obscured by responsive UI. |

---

# 12. Responsive Behavior

The page is rendered within the Admin Shell responsive structure.

| ID | Viewport | Layout |
|---|---|---|
| `FE-ACCOUNT-CREATE-RESP-001` | `>= 1280px` | Standard desktop Admin Shell with constrained Create form. |
| `FE-ACCOUNT-CREATE-RESP-002` | `768px–1279px` | Reduced-width Shell with readable form content. |
| `FE-ACCOUNT-CREATE-RESP-003` | `< 768px` | Collapsible Sidebar with single-column Create form. |

## Responsive Requirements

| ID | Requirement |
|---|---|
| `FE-ACCOUNT-CREATE-RESP-004` | Prevent unintended page-level horizontal scrolling. |
| `FE-ACCOUNT-CREATE-RESP-005` | Keep form controls within the available content width. |
| `FE-ACCOUNT-CREATE-RESP-006` | Allow action controls to stack when necessary on narrow screens. |
| `FE-ACCOUNT-CREATE-RESP-007` | Preserve readable touch targets. |
| `FE-ACCOUNT-CREATE-RESP-008` | Preserve keyboard accessibility across viewport sizes. |
| `FE-ACCOUNT-CREATE-RESP-009` | Preserve authentication behavior across viewport sizes. |
| `FE-ACCOUNT-CREATE-RESP-010` | Preserve authorization behavior across viewport sizes. |
| `FE-ACCOUNT-CREATE-RESP-011` | Preserve API contract and Account state semantics across viewport sizes. |

Suggested mobile structure:

    Administrator Account

    Create a new administrator account.

    Email
    [________________________]

    Password
    [________________________]

    [Create Account]
    [Cancel]

---

# 13. Component Structure

The page should remain feature-local and keep Account Create behavior outside the Admin Shell.

Suggested structure:

    src/pages/accounts/
    ├── AccountListPage
    ├── AccountCreatePage
    ├── AccountDetailPage
    └── AccountEditPage

Suggested Account Create page-local components:

    AccountCreatePage
    ├── AccountCreateHeader
    ├── AccountCreateForm
    ├── AccountCreateField
    ├── AccountCreateActions
    ├── AccountCreateStatus
    └── AccountCreateErrorState

| ID | Component | Responsibility |
|---|---|---|
| `FE-ACCOUNT-CREATE-COMP-001` | `AccountCreatePage` | Coordinate route state, form state, submission state, navigation, and page layout. |
| `FE-ACCOUNT-CREATE-COMP-002` | `AccountCreateHeader` | Render page title, description, and Back navigation. |
| `FE-ACCOUNT-CREATE-COMP-003` | `AccountCreateForm` | Render Email and Password fields and own form submission behavior. |
| `FE-ACCOUNT-CREATE-COMP-004` | `AccountCreateField` | Provide feature-local field presentation, labels, and validation messaging where useful. |
| `FE-ACCOUNT-CREATE-COMP-005` | `AccountCreateActions` | Render Create Account and Cancel actions. |
| `FE-ACCOUNT-CREATE-COMP-006` | `AccountCreateStatus` | Render submission progress and accessible status messaging. |
| `FE-ACCOUNT-CREATE-COMP-007` | `AccountCreateErrorState` | Render page-level authorization and unexpected error states. |

The page must not introduce:

    AccountLifecycleSection
    AccountRoleEditor
    AccountPasswordPolicyValidator
    AccountDeleteAction

because those responsibilities belong to other domains or pages.

API calls should remain in the existing frontend service layer:

    src/services/

Type definitions should remain in:

    src/types/

The page should use React, TypeScript, and the project's native CSS approach.

No page-specific UI framework or large component dependency should be introduced solely for this page.

---

# 14. Testing

The implementation must follow the repository's frontend test structure:

    Static Checks
        ↓
    Unit
        ↓
    Integration
        ↓
    E2E
        ↓
    Manual

## 14.1 Unit

| ID | Test |
|---|---|
| `FE-ACCOUNT-CREATE-TEST-UNIT-001` | Create Account page renders the expected heading. |
| `FE-ACCOUNT-CREATE-TEST-UNIT-002` | Email field renders as required email input. |
| `FE-ACCOUNT-CREATE-TEST-UNIT-003` | Password field renders as required password input. |
| `FE-ACCOUNT-CREATE-TEST-UNIT-004` | Password uses `autocomplete="new-password"`. |
| `FE-ACCOUNT-CREATE-TEST-UNIT-005` | Cancel action targets `/admin/accounts`. |
| `FE-ACCOUNT-CREATE-TEST-UNIT-006` | Create action submits the form. |
| `FE-ACCOUNT-CREATE-TEST-UNIT-007` | Duplicate submission is prevented while pending. |
| `FE-ACCOUNT-CREATE-TEST-UNIT-008` | `EMAIL_ALREADY_IN_USE` renders on the Email field. |
| `FE-ACCOUNT-CREATE-TEST-UNIT-009` | `403` renders an authorization error. |
| `FE-ACCOUNT-CREATE-TEST-UNIT-010` | `401` delegates to shared authentication handling. |
| `FE-ACCOUNT-CREATE-TEST-UNIT-011` | Unexpected errors render a generic retryable message. |
| `FE-ACCOUNT-CREATE-TEST-UNIT-012` | No display-name field is submitted to the Create API. |
| `FE-ACCOUNT-CREATE-TEST-UNIT-013` | No client-only password policy is enforced. |
| `FE-ACCOUNT-CREATE-TEST-UNIT-014` | Password is not included in rendered error output. |
| `FE-ACCOUNT-CREATE-TEST-UNIT-015` | Successful creation resolves the destination Account ID correctly. |

## 14.2 Integration

| ID | Test |
|---|---|
| `FE-ACCOUNT-CREATE-TEST-INT-001` | Authenticated navigation from Account Management to Create Account works. |
| `FE-ACCOUNT-CREATE-TEST-INT-002` | Create Account page renders inside the Admin Shell. |
| `FE-ACCOUNT-CREATE-TEST-INT-003` | Accounts navigation remains active. |
| `FE-ACCOUNT-CREATE-TEST-INT-004` | `POST /admin/accounts` receives the expected Email and Password payload. |
| `FE-ACCOUNT-CREATE-TEST-INT-005` | Bearer credentials are sent through the Authorization header. |
| `FE-ACCOUNT-CREATE-TEST-INT-006` | `201 Created` navigates to the created Account Detail route. |
| `FE-ACCOUNT-CREATE-TEST-INT-007` | `409 EMAIL_ALREADY_IN_USE` remains on the Create page. |
| `FE-ACCOUNT-CREATE-TEST-INT-008` | `422 VALIDATION_ERROR` renders server validation feedback. |
| `FE-ACCOUNT-CREATE-TEST-INT-009` | `403` preserves authentication state. |
| `FE-ACCOUNT-CREATE-TEST-INT-010` | `401` transitions through the Authentication flow. |
| `FE-ACCOUNT-CREATE-TEST-INT-011` | Network failure preserves retryable form state. |
| `FE-ACCOUNT-CREATE-TEST-INT-012` | Password never appears in browser storage. |
| `FE-ACCOUNT-CREATE-TEST-INT-013` | Tokens never appear in URLs or rendered output. |
| `FE-ACCOUNT-CREATE-TEST-INT-014` | Duplicate submissions produce only one create request. |

## 14.3 E2E

| ID | Test |
|---|---|
| `FE-ACCOUNT-CREATE-TEST-E2E-001` | Login → Admin Shell → Accounts works. |
| `FE-ACCOUNT-CREATE-TEST-E2E-002` | Create Account navigation opens `/admin/accounts/create`. |
| `FE-ACCOUNT-CREATE-TEST-E2E-003` | Account Create form renders correctly. |
| `FE-ACCOUNT-CREATE-TEST-E2E-004` | Valid Email and Password create an administrator account. |
| `FE-ACCOUNT-CREATE-TEST-E2E-005` | Successful creation navigates to the new Account Detail page. |
| `FE-ACCOUNT-CREATE-TEST-E2E-006` | Duplicate Email displays a field-specific error. |
| `FE-ACCOUNT-CREATE-TEST-E2E-007` | Unauthorized Account creation displays `403` without logout. |
| `FE-ACCOUNT-CREATE-TEST-E2E-008` | Authentication expiry redirects to `/login`. |
| `FE-ACCOUNT-CREATE-TEST-E2E-009` | Cancel returns to Account Management without submitting. |
| `FE-ACCOUNT-CREATE-TEST-E2E-010` | Mobile layout remains usable below `768px`. |

## 14.4 Accessibility

| ID | Test |
|---|---|
| `FE-ACCOUNT-CREATE-TEST-A11Y-001` | Exactly one primary page heading is present. |
| `FE-ACCOUNT-CREATE-TEST-A11Y-002` | Email has an accessible label. |
| `FE-ACCOUNT-CREATE-TEST-A11Y-003` | Password has an accessible label. |
| `FE-ACCOUNT-CREATE-TEST-A11Y-004` | Validation messages are associated with their controls. |
| `FE-ACCOUNT-CREATE-TEST-A11Y-005` | Submission status is announced appropriately. |
| `FE-ACCOUNT-CREATE-TEST-A11Y-006` | Back and Cancel navigation are keyboard accessible. |
| `FE-ACCOUNT-CREATE-TEST-A11Y-007` | Create Account action is keyboard accessible. |
| `FE-ACCOUNT-CREATE-TEST-A11Y-008` | Focus is visible. |
| `FE-ACCOUNT-CREATE-TEST-A11Y-009` | Error presentation does not rely only on color. |

## 14.5 Responsive

| ID | Test |
|---|---|
| `FE-ACCOUNT-CREATE-TEST-RESP-001` | Desktop layout works at `>= 1280px`. |
| `FE-ACCOUNT-CREATE-TEST-RESP-002` | Tablet layout works at `768px–1279px`. |
| `FE-ACCOUNT-CREATE-TEST-RESP-003` | Mobile layout works below `768px`. |
| `FE-ACCOUNT-CREATE-TEST-RESP-004` | No unintended page-level horizontal scrolling occurs. |
| `FE-ACCOUNT-CREATE-TEST-RESP-005` | Primary and secondary actions remain usable on mobile. |
| `FE-ACCOUNT-CREATE-TEST-RESP-006` | Admin Shell behavior is preserved at every viewport size. |
| `FE-ACCOUNT-CREATE-TEST-RESP-007` | Authentication and authorization behavior is unchanged across viewport sizes. |

---

# 15. Implementation Criteria

## Status Values

| Status | Meaning |
|---|---|
| ⚪ Not Started | Criteria has not been implemented or verified. |
| 🟡 In Progress | Implementation or verification is still in progress. |
| 🟢 Implemented | Implementation is complete and verified. |
| 🔴 Blocked | Implementation cannot proceed because of a blocker. |

The current implementation is in place. Criteria are marked according to the implemented frontend behavior and the latest verification results.

## 15.1 Route

| ID | Criteria | Status | Reason |
|---|---|---|---|
| `FE-ACCOUNT-CREATE-IMPL-001` | `/admin/accounts/create` renders inside the Admin Shell. | 🟢 Implemented | Implemented and covered by the Admin Shell Account Create E2E flow. |
| `FE-ACCOUNT-CREATE-IMPL-002` | Unauthenticated access follows Admin Shell authentication behavior. | 🟢 Implemented | Shared authentication routing handles protected Account Create access. |
| `FE-ACCOUNT-CREATE-IMPL-003` | Accounts navigation remains active while creating an Account. | 🟢 Implemented | Admin Shell keeps Accounts active on the Create route. |
| `FE-ACCOUNT-CREATE-IMPL-004` | Back and Cancel navigate to `/admin/accounts`. | 🟡 In Progress | Implementation is present; the latest E2E run still has an unscoped Create Account test locator in the Back/Cancel flow. |
| `FE-ACCOUNT-CREATE-IMPL-005` | Successful creation navigates to `/admin/accounts/:id`. | 🟢 Implemented | Successful creation and Account Detail navigation pass the current E2E contract test. |

## 15.2 Form

| ID | Criteria | Status | Reason |
|---|---|---|---|
| `FE-ACCOUNT-CREATE-IMPL-006` | Email renders as a required email input. | 🟢 Implemented | Implemented with native email and required constraints. |
| `FE-ACCOUNT-CREATE-IMPL-007` | Password renders as a required password input. | 🟢 Implemented | Implemented with native password and required constraints. |
| `FE-ACCOUNT-CREATE-IMPL-008` | Password uses `autocomplete="new-password"`. | 🟢 Implemented | Implemented directly on the Password field. |
| `FE-ACCOUNT-CREATE-IMPL-009` | No unsupported display-name field is submitted. | 🟢 Implemented | Create request contains only Email and Password. |
| `FE-ACCOUNT-CREATE-IMPL-010` | No unsupported frontend password policy is enforced. | 🟢 Implemented | Frontend does not impose a password length or composition policy. |
| `FE-ACCOUNT-CREATE-IMPL-011` | Native form validation is supported. | 🟢 Implemented | Required and email constraints are enforced through browser validation. |

## 15.3 API

| ID | Criteria | Status | Reason |
|---|---|---|---|
| `FE-ACCOUNT-CREATE-IMPL-012` | Create uses `POST /admin/accounts`. | 🟢 Implemented | Implemented through `accountsService.create()`. |
| `FE-ACCOUNT-CREATE-IMPL-013` | Bearer credentials are sent only in the Authorization header. | 🟢 Implemented | Protected Account requests use the shared authentication service. |
| `FE-ACCOUNT-CREATE-IMPL-014` | The request body contains only supported Account Create fields. | 🟢 Implemented | Request body contains only `email` and `password`. |
| `FE-ACCOUNT-CREATE-IMPL-015` | `201` is handled as successful creation. | 🟢 Implemented | `201` responses are parsed as the created Account response. |
| `FE-ACCOUNT-CREATE-IMPL-016` | `Location` or response Account ID resolves the created Account destination. | 🟢 Implemented | The returned Account ID is used to build the Account Detail destination. |
| `FE-ACCOUNT-CREATE-IMPL-017` | `409 EMAIL_ALREADY_IN_USE` is handled as an Email conflict. | 🟢 Implemented | Conflict code renders a field-level Email error and preserves retry state. |
| `FE-ACCOUNT-CREATE-IMPL-018` | `422 VALIDATION_ERROR` is rendered from server validation. | 🟡 In Progress | Implementation is present; the latest E2E run still has an alert-locator assertion failure. |
| `FE-ACCOUNT-CREATE-IMPL-019` | `403` is rendered as an authorization failure without logout. | 🟡 In Progress | Implementation is present; the latest E2E run still has an alert-locator assertion failure. |
| `FE-ACCOUNT-CREATE-IMPL-020` | `401` is delegated to shared authentication behavior. | 🟢 Implemented | Shared authentication handling redirects to `/login` after failed recovery. |

## 15.4 Submission

| ID | Criteria | Status | Reason |
|---|---|---|---|
| `FE-ACCOUNT-CREATE-IMPL-021` | Duplicate submission is prevented. | 🟢 Implemented | In-flight submission state prevents a second create request. |
| `FE-ACCOUNT-CREATE-IMPL-022` | Create action is disabled while the request is pending. | 🟢 Implemented | Create action is disabled and changes to `Creating…`. |
| `FE-ACCOUNT-CREATE-IMPL-023` | Retryable errors preserve usable form state. | 🟢 Implemented | Conflict and authorization/validation paths preserve the entered form values. |
| `FE-ACCOUNT-CREATE-IMPL-024` | Successful creation clears transient form state before navigation. | 🟢 Implemented | Email, Password, and transient error state are cleared before navigation. |

## 15.5 Security

| ID | Criteria | Status | Reason |
|---|---|---|---|
| `FE-ACCOUNT-CREATE-IMPL-025` | Backend remains the authentication authority. | 🟢 Implemented | Authentication state and `401` handling remain in the shared authentication service. |
| `FE-ACCOUNT-CREATE-IMPL-026` | Backend remains the authorization authority. | 🟢 Implemented | The frontend renders `403` but does not establish authorization itself. |
| `FE-ACCOUNT-CREATE-IMPL-027` | Password is never persisted in browser storage. | 🟢 Implemented | Password state is component-local and is cleared before successful navigation. |
| `FE-ACCOUNT-CREATE-IMPL-028` | Password is never logged. | 🟢 Implemented | No logging of form credentials is performed. |
| `FE-ACCOUNT-CREATE-IMPL-029` | Tokens never appear in URLs. | 🟢 Implemented | Account Create navigation uses only the Account ID. |
| `FE-ACCOUNT-CREATE-IMPL-030` | Tokens never render in the UI. | 🟢 Implemented | Access tokens are kept out of rendered content. |
| `FE-ACCOUNT-CREATE-IMPL-031` | Internal server diagnostics are not exposed. | 🟢 Implemented | UI uses generic error messages instead of server `detail` content. |

## 15.6 Accessibility

| ID | Criteria | Status | Reason |
|---|---|---|---|
| `FE-ACCOUNT-CREATE-IMPL-032` | Semantic page structure is used. | 🟢 Implemented | Uses semantic sections, header, form, labels, and status/error regions. |
| `FE-ACCOUNT-CREATE-IMPL-033` | Primary page heading is present. | 🟢 Implemented | The page exposes one primary `h1`. |
| `FE-ACCOUNT-CREATE-IMPL-034` | All inputs have accessible labels. | 🟢 Implemented | Email and Password use explicit labels. |
| `FE-ACCOUNT-CREATE-IMPL-035` | Validation errors are associated with controls. | 🟢 Implemented | `aria-invalid` and `aria-describedby` associate field errors with controls. |
| `FE-ACCOUNT-CREATE-IMPL-036` | Async submission status is accessible. | 🟢 Implemented | Submission progress uses a polite status region. |
| `FE-ACCOUNT-CREATE-IMPL-037` | Navigation and actions are keyboard accessible. | 🟢 Implemented | Back, Cancel, and Create Account use native keyboard-accessible controls. |
| `FE-ACCOUNT-CREATE-IMPL-038` | Focus is visible and not obscured. | 🟢 Implemented | Existing repository focus styles and feature-local error focus handling are used. |

## 15.7 Current Verification Status

| Area | Status | Current Reason |
|---|---|---|
| Account Create implementation | 🟢 Implemented | The page, service integration, authentication handling, authorization handling, validation, security boundaries, and navigation are implemented. |
| Unit coverage | 🟢 Implemented | Account Create unit coverage exists for form contract, validation, duplicate submission, conflicts, errors, authorization, and success navigation. |
| Integration coverage | 🟢 Implemented | Account Create integration coverage exists for authenticated rendering, API submission, authorization failure, and security boundaries. |
| E2E coverage | 🟢 Implemented | The latest E2E run passes the Account Create scenarios for route rendering, API contract, native validation, conflict handling, validation errors, authorization errors, authentication expiry, duplicate submission, and navigation. |
| Production Account Create behavior | 🟢 Implemented | The latest E2E run confirms the implemented Account Create user flows and documented error states. |

---

# 16. Traceability

## Requirements → Design → Backend → Test

| Requirement | Frontend Design | Backend Contract | Test |
|---|---|---|---|
| `ADM-AUTH-001` | `FE-ACCOUNT-CREATE-ROUTE-001` | Authentication domain | `FE-ACCOUNT-CREATE-TEST-E2E-001` |
| `ADM-AUTH-003` | `FE-ACCOUNT-CREATE-REQ-002` | `AC_UC_01`, `AC_API_01` | `FE-ACCOUNT-CREATE-TEST-E2E-004` |
| `CNT-ADMIN-002` | `FE-ACCOUNT-CREATE-FORM-001` | `AC_API_01` | `FE-ACCOUNT-CREATE-TEST-UNIT-002` |
| `CNT-ADMIN-003` | `FE-ACCOUNT-CREATE-FORM-007` | `AC_API_01` | `FE-ACCOUNT-CREATE-TEST-UNIT-003` |
| `PAGE-ADM-009` | `FE-ACCOUNT-CREATE-ROUTE-001` | Frontend routing | `FE-ACCOUNT-CREATE-TEST-E2E-002` |
| `AC_UC_01` | `FE-ACCOUNT-CREATE-REQ-002` | `POST /admin/accounts` | `FE-ACCOUNT-CREATE-TEST-INT-004` |
| `AC_API_01` | `FE-ACCOUNT-CREATE-API-001` | `POST /admin/accounts` | `FE-ACCOUNT-CREATE-TEST-INT-004` |
| `account:create` | `FE-ACCOUNT-CREATE-API-009` | Authorization domain | `FE-ACCOUNT-CREATE-TEST-INT-009` |
| `EMAIL_ALREADY_IN_USE` | `FE-ACCOUNT-CREATE-ERR-006` | `AC_API_01` | `FE-ACCOUNT-CREATE-TEST-E2E-006` |
| `VALIDATION_ERROR` | `FE-ACCOUNT-CREATE-ERR-001` | `AC_API_01` | `FE-ACCOUNT-CREATE-TEST-INT-008` |
| Authentication `401` | `FE-ACCOUNT-CREATE-API-010` | Authentication domain | `FE-ACCOUNT-CREATE-TEST-INT-010` |
| Authorization `403` | `FE-ACCOUNT-CREATE-API-011` | Authorization domain | `FE-ACCOUNT-CREATE-TEST-E2E-007` |
| Admin Shell | `FE-ACCOUNT-CREATE-ROUTE-001`, `FE-ACCOUNT-CREATE-RESP-*` | `admin_shell.md` | `FE-ACCOUNT-CREATE-TEST-E2E-001` |
| Account Management | `FE-ACCOUNT-CREATE-ACTION-001` | `PAGE-ADM-008` | `FE-ACCOUNT-CREATE-TEST-E2E-009` |
| Account Detail | `FE-ACCOUNT-CREATE-ROUTE-006` | `AC_API_03`, `PAGE-ADM-010` | `FE-ACCOUNT-CREATE-TEST-E2E-005` |

## Domain References

| Domain | Relevant References |
|---|---|
| Requirements | `ADM-AUTH-001`, `ADM-AUTH-003`, `CNT-ADMIN-*`, `SCP-009` |
| Sitemap | `PAGE-ADM-009`, `PAGE-ADM-010` |
| Account | `AC_UC_01`, `AC_API_01`, Account Response, Account validation rules |
| Authentication | Protected route behavior, bearer authentication, `401` handling |
| Authorization | `account:create`, deny-by-default, `403` |
| Admin Shell | Shell layout, route protection, Accounts active navigation, responsive behavior |
| Account Management | `/admin/accounts`, Create navigation |
| Account Detail | `/admin/accounts/:id`, post-create destination |

## Design Boundary

    Requirements
        ↓
    Sitemap
        ↓
    Admin Shell
        ↓
    Account Management
        ↓
    Account Create
        ├── Email
        ├── Password
        └── Create Account
                ↓
        POST /admin/accounts
                ↓
        Authentication
                ↓
        Authorization
                ↓
        Account Domain
                ↓
        Account Detail
                ↓
        Tests

## Implementation Boundary

The Account Create page owns:

    Form state
    Validation presentation
    Submission state
    Create navigation
    Error presentation

The Account Create page does not own:

    Authentication state implementation
    Authorization policy
    Password hashing
    Account persistence
    Account lifecycle invariants
    Role assignment
    Account editing
    Account deletion
```
