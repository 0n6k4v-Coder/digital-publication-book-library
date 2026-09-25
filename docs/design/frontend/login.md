# Frontend Login

## Table of Contents

1. [Scope](#scope)
2. [Route](#route)
3. [Requirements](#requirements)
4. [UI](#ui)
5. [Authentication Flow](#authentication-flow)
6. [API Contract](#api-contract)
7. [UI States](#ui-states)
8. [Security](#security)
9. [Accessibility](#accessibility)
10. [Testing](#testing)
11. [Acceptance Criteria](#acceptance-criteria)

---

# 1. Scope

This document defines the Admin Login page and its frontend behavior.

The backend Authentication domain remains the source of truth for authentication rules, credentials, tokens, sessions, security, and error contracts.

Relevant backend use case:

```text
AU_UC_01 — Authenticate Account
```

Relevant backend API:

```text
AU_API_01 — POST /auth/login
```

---

# 2. Route

| Route    | Purpose              |
| -------- | -------------------- |
| `/login` | Admin authentication |

Unauthenticated users may access `/login`.

Authenticated users should be redirected to `/admin`.

---

# 3. Requirements

| ID            | Requirement                                                                                  |
| ------------- | -------------------------------------------------------------------------------------------- |
| `FE_LOGIN_01` | Display the Admin Login page at `/login`.                                                    |
| `FE_LOGIN_02` | Accept an email address and password.                                                        |
| `FE_LOGIN_03` | Submit credentials to `POST /auth/login`.                                                    |
| `FE_LOGIN_04` | Show a loading state while authentication is in progress.                                    |
| `FE_LOGIN_05` | Show validation errors for invalid required input.                                           |
| `FE_LOGIN_06` | Show a generic authentication error for invalid credentials.                                 |
| `FE_LOGIN_07` | Show a generic rate-limit error for `429 AUTHENTICATION_RATE_LIMITED`.                       |
| `FE_LOGIN_08` | On successful authentication, establish authenticated client state and navigate to `/admin`. |
| `FE_LOGIN_09` | Do not expose authentication tokens in the UI.                                               |
| `FE_LOGIN_10` | Do not log passwords, access tokens, or refresh tokens.                                      |

---

# 4. UI

The Login page contains:

```text
Login Page
├── Email
├── Password
└── Sign In
```

## Email

| Property     | Value      |
| ------------ | ---------- |
| Type         | `email`    |
| Required     | Yes        |
| Autocomplete | `username` |

## Password

| Property     | Value              |
| ------------ | ------------------ |
| Type         | `password`         |
| Required     | Yes                |
| Autocomplete | `current-password` |

The frontend performs basic required-field validation only.

Authentication rules remain server-side.

The frontend must not add password-policy validation to the login form that could prevent valid authentication requests.

---

# 5. Authentication Flow

```text
User opens /login
        ↓
Enter email and password
        ↓
Submit
        ↓
POST /auth/login
        ↓
Success
        ↓
Establish authenticated client state
        ↓
Navigate to /admin
```

Failure:

```text
POST /auth/login
        ↓
Authentication failure
        ↓
Remain on /login
        ↓
Display generic error
```

Unauthenticated access to `/admin` is handled by the Admin Shell design.

Logout behavior is defined by `admin-shell.md`.

---

# 6. API Contract

## Request

```http
POST /auth/login
Content-Type: application/json
```

```json
{
  "email": "admin@example.com",
  "password": "example-secure-password"
}
```

## Success

```text
200 OK
Cache-Control: no-store
Content-Type: application/json
```

```json
{
  "access_token": "opaque-access-token",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "opaque-refresh-token",
  "refresh_expires_in": 2592000
}
```

The frontend authentication service owns the received authentication state.

Raw tokens must never be rendered, logged, or exposed through application UI.

## Errors

| Status | Code                          | Frontend Behavior                    |
| ------ | ----------------------------- | ------------------------------------ |
| `400`  | `INVALID_REQUEST`             | Display request/validation error     |
| `401`  | `INVALID_CREDENTIALS`         | Display generic authentication error |
| `429`  | `AUTHENTICATION_RATE_LIMITED` | Display generic rate-limit error     |

The frontend must not distinguish whether an account exists.

---

# 7. UI States

The Login page must explicitly handle:

| State               | Behavior                                               |
| ------------------- | ------------------------------------------------------ |
| Initial             | Show empty login form                                  |
| Editing             | Allow credential entry                                 |
| Submitting          | Disable submission and show loading state              |
| Invalid Request     | Display validation/request error                       |
| Invalid Credentials | Display generic authentication error                   |
| Rate Limited        | Display generic rate-limit error                       |
| Success             | Establish authenticated state and navigate to `/admin` |

Duplicate submissions must be prevented while authentication is pending.

---

# 8. Security

The frontend must follow the Authentication domain security contract.

* Use HTTPS.
* Send credentials only to `POST /auth/login`.
* Do not send tokens in query parameters.
* Do not send tokens to unrelated endpoints.
* Never log passwords.
* Never log access tokens.
* Never log refresh tokens.
* Never display tokens.
* Do not expose authentication credentials through error messages.
* Do not reveal whether an account exists.
* Treat the backend as the authority for authentication.
* Treat `401 Unauthorized` as unauthenticated.
* Do not implement authentication or account-state rules independently in the UI.

Relevant backend security requirements include:

```text
AU_REQ_NON_FC_01
AU_REQ_NON_FC_04
AU_REQ_NON_FC_06
AU_REQ_NON_FC_07

AU_SEC_REQ_08
AU_SEC_REQ_09
```

---

# 9. Accessibility

The Login page must:

* Use a visible label for each form field.
* Associate validation errors with their fields when applicable.
* Support keyboard submission.
* Provide visible focus states.
* Use semantic form controls.
* Communicate loading and error states to assistive technologies where appropriate.
* Keep the Sign In action accessible while the form is usable.

---

# 10. Testing

## Unit

Test:

* Login form rendering.
* Required-field validation.
* Loading state.
* Error state rendering.
* Submit-button disabled state.

## Integration

Test:

* Form submission.
* `POST /auth/login` request.
* Successful authentication.
* `400` handling.
* `401` handling.
* `429` handling.
* Navigation to `/admin` after successful authentication.
* Authentication state creation.

## E2E

Test the complete flow:

```text
Open /login
    ↓
Enter valid credentials
    ↓
Submit
    ↓
Login succeeds
    ↓
Admin Shell appears
```

Test logout in the Admin Shell flow.

Verify that invalid credentials do not enter the Admin Shell.

## Manual

Verify:

* Visual layout.
* Keyboard interaction.
* Loading behavior.
* Error messages.
* Responsive behavior.
* Accessibility behavior.
* Successful transition to the Admin Shell.

---

# 11. Acceptance Criteria

The Login implementation is complete when:

```text
✓ /login is accessible
✓ Email and password fields are available
✓ Required-field validation works
✓ POST /auth/login is called correctly
✓ Successful authentication establishes client auth state
✓ Successful authentication navigates to /admin
✓ Invalid credentials remain on /login
✓ 400 responses are handled
✓ 401 responses are handled generically
✓ 429 responses are handled generically
✓ Authentication tokens are not displayed
✓ Authentication tokens are not logged
✓ Passwords are not logged
✓ Unauthenticated users cannot enter the Admin Shell
✓ Login tests exist at applicable test levels
✓ Manual verification is complete
```
