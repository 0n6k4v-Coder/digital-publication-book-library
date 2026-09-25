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
11. [Implementation Criteria](#implementation-criteria)

---

# 1. Scope

This document defines the **Admin Login page** and its **frontend behavior**.

| Item | Reference |
| --- | --- |
| Frontend Scope | Admin Login page and frontend behavior |
| Authentication Source of Truth | Backend Authentication domain |
| Backend Use Case | `AU_UC_01 — Authenticate Account` |
| Backend API | `AU_API_01 — POST /auth/login` |
| Backend Owns | Authentication rules, credentials, tokens, sessions, security, and error contracts |

---

# 2. Route

| ID | Route | Access | Purpose | Behavior |
| --- | --- | --- | --- | --- |
| `FE_LOGIN_ROUTE_01` | `/login` | Unauthenticated | Admin authentication | Display Login page |
| `FE_LOGIN_ROUTE_02` | `/login` | Authenticated | Admin authentication | Redirect to `/admin` |
| `FE_LOGIN_ROUTE_03` | `/admin` | Unauthenticated | Admin application | Redirect to `/login` |

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

## Components

| ID               | Component      |
| ---------------- | -------------- |
| `FE_LOGIN_UI_01` | Email field    |
| `FE_LOGIN_UI_02` | Password field |
| `FE_LOGIN_UI_03` | Sign In action |

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

## Validation

| ID               | Requirement                                                                             |
| ---------------- | --------------------------------------------------------------------------------------- |
| `FE_LOGIN_UI_04` | Perform basic required-field validation                                                 |
| `FE_LOGIN_UI_05` | Authentication rules remain server-side                                                 |
| `FE_LOGIN_UI_06` | Do not add password-policy validation that could prevent a valid authentication request |

---

# 5. Authentication Flow

## Success Flow

| Step | Action                                          |
| ---- | ----------------------------------------------- |
| 1    | User opens `/login`                             |
| 2    | User enters email and password                  |
| 3    | User submits the Login form                     |
| 4    | Frontend sends `POST /auth/login`               |
| 5    | Authentication succeeds                         |
| 6    | Frontend establishes authenticated client state |
| 7    | Frontend navigates to `/admin`                  |

## Failure Flow

| Step | Action                                          |
| ---- | ----------------------------------------------- |
| 1    | User submits the Login form                     |
| 2    | Frontend sends `POST /auth/login`               |
| 3    | Authentication fails                            |
| 4    | User remains on `/login`                        |
| 5    | Frontend displays the appropriate generic error |

## Related Behavior

| Behavior                           | Source           |
| ---------------------------------- | ---------------- |
| Unauthenticated access to `/admin` | `admin-shell.md` |
| Logout behavior                    | `admin-shell.md` |

---

# 6. API Contract

## Request

| Property     | Value              |
| ------------ | ------------------ |
| Method       | `POST`             |
| Endpoint     | `/auth/login`      |
| Content-Type | `application/json` |

### Request Body

| Field      | Type     | Required | Description           |
| ---------- | -------- | -------- | --------------------- |
| `email`    | `string` | Yes      | Account email address |
| `password` | `string` | Yes      | Account password      |

```json
{
  "email": "admin@example.com",
  "password": "example-secure-password"
}
```

## Success

| Property      | Value              |
| ------------- | ------------------ |
| Status        | `200 OK`           |
| Content-Type  | `application/json` |
| Cache-Control | `no-store`         |

### Response Body

| Field                | Type     | Description                       |
| -------------------- | -------- | --------------------------------- |
| `access_token`       | `string` | Opaque access token               |
| `token_type`         | `string` | Authentication token type         |
| `expires_in`         | `number` | Access-token lifetime in seconds  |
| `refresh_token`      | `string` | Opaque refresh token              |
| `refresh_expires_in` | `number` | Refresh-token lifetime in seconds |

```json
{
  "access_token": "opaque-access-token",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "opaque-refresh-token",
  "refresh_expires_in": 2592000
}
```

| ID                | Requirement                                                                                 |
| ----------------- | ------------------------------------------------------------------------------------------- |
| `FE_LOGIN_API_01` | The frontend authentication service owns the received authentication state                  |
| `FE_LOGIN_API_02` | Raw authentication tokens must never be rendered, logged, or exposed through application UI |

## Errors

| ID                    | Status | Code                          | Frontend Behavior                    |
| --------------------- | ------ | ----------------------------- | ------------------------------------ |
| `FE_LOGIN_API_ERR_01` | `400`  | `INVALID_REQUEST`             | Display request/validation error     |
| `FE_LOGIN_API_ERR_02` | `401`  | `INVALID_CREDENTIALS`         | Display generic authentication error |
| `FE_LOGIN_API_ERR_03` | `429`  | `AUTHENTICATION_RATE_LIMITED` | Display generic rate-limit error     |

| ID                    | Requirement                                                 |
| --------------------- | ----------------------------------------------------------- |
| `FE_LOGIN_API_SEC_01` | The frontend must not distinguish whether an account exists |

---

# 7. UI States

| ID                  | State                | Behavior                                                      |
| ------------------- | -------------------- | ------------------------------------------------------------- |
| `FE_LOGIN_STATE_01` | Initial              | Show empty login form                                         |
| `FE_LOGIN_STATE_02` | Editing              | Allow credential entry                                        |
| `FE_LOGIN_STATE_03` | Submitting           | Disable submission and show loading state                     |
| `FE_LOGIN_STATE_04` | Invalid Request      | Display validation/request error                              |
| `FE_LOGIN_STATE_05` | Invalid Credentials  | Display generic authentication error                          |
| `FE_LOGIN_STATE_06` | Rate Limited         | Display generic rate-limit error                              |
| `FE_LOGIN_STATE_07` | Success              | Establish authenticated state and navigate to `/admin`        |
| `FE_LOGIN_STATE_08` | Duplicate Submission | Prevent duplicate submissions while authentication is pending |

---

# 8. Security

The frontend must follow the Authentication domain security contract.

| ID                | Security Rule                 | Requirement                                                                    |
| ----------------- | ----------------------------- | ------------------------------------------------------------------------------ |
| `FE_LOGIN_SEC_01` | Transport                     | Use HTTPS                                                                      |
| `FE_LOGIN_SEC_02` | Credentials                   | Send credentials only to `POST /auth/login`                                    |
| `FE_LOGIN_SEC_03` | Token Query Parameters        | Do not send tokens in query parameters                                         |
| `FE_LOGIN_SEC_04` | Token Endpoints               | Do not send tokens to unrelated endpoints                                      |
| `FE_LOGIN_SEC_05` | Password Logging              | Never log passwords                                                            |
| `FE_LOGIN_SEC_06` | Access Token Logging          | Never log access tokens                                                        |
| `FE_LOGIN_SEC_07` | Refresh Token Logging         | Never log refresh tokens                                                       |
| `FE_LOGIN_SEC_08` | Token Display                 | Never display tokens                                                           |
| `FE_LOGIN_SEC_09` | Error Messages                | Do not expose authentication credentials                                       |
| `FE_LOGIN_SEC_10` | Account Existence             | Do not reveal whether an account exists                                        |
| `FE_LOGIN_SEC_11` | Authentication Authority      | Treat the backend as the authority for authentication                          |
| `FE_LOGIN_SEC_12` | `401 Unauthorized`            | Treat `401 Unauthorized` as unauthenticated                                    |
| `FE_LOGIN_SEC_13` | Frontend Authentication Rules | Do not implement authentication or account-state rules independently in the UI |

### Backend Security References

| Frontend ID       | Backend Reference  |
| ----------------- | ------------------ |
| `FE_LOGIN_SEC_01` | `AU_REQ_NON_FC_01` |
| `FE_LOGIN_SEC_02` | `AU_REQ_NON_FC_04` |
| `FE_LOGIN_SEC_03` | `AU_REQ_NON_FC_06` |
| `FE_LOGIN_SEC_04` | `AU_REQ_NON_FC_06` |
| `FE_LOGIN_SEC_05` | `AU_REQ_NON_FC_07` |
| `FE_LOGIN_SEC_06` | `AU_SEC_REQ_08`    |
| `FE_LOGIN_SEC_07` | `AU_SEC_REQ_08`    |
| `FE_LOGIN_SEC_08` | `AU_SEC_REQ_08`    |
| `FE_LOGIN_SEC_09` | `AU_SEC_REQ_09`    |
| `FE_LOGIN_SEC_10` | `AU_SEC_REQ_09`    |
| `FE_LOGIN_SEC_11` | `AU_SEC_REQ_08`    |
| `FE_LOGIN_SEC_12` | `AU_SEC_REQ_09`    |
| `FE_LOGIN_SEC_13` | `AU_SEC_REQ_09`    |

---

# 9. Accessibility

| ID                 | Accessibility Requirement                                                        |
| ------------------ | -------------------------------------------------------------------------------- |
| `FE_LOGIN_A11Y_01` | Use a visible label for each form field                                          |
| `FE_LOGIN_A11Y_02` | Associate validation errors with their fields when applicable                    |
| `FE_LOGIN_A11Y_03` | Support keyboard submission                                                      |
| `FE_LOGIN_A11Y_04` | Provide visible focus states                                                     |
| `FE_LOGIN_A11Y_05` | Use semantic form controls                                                       |
| `FE_LOGIN_A11Y_06` | Communicate loading and error states to assistive technologies where appropriate |
| `FE_LOGIN_A11Y_07` | Keep the Sign In action accessible while the form is usable                      |

---

# 11. Implementation Criteria

### Status Values

| Status         | Meaning                                            |
| -------------- | -------------------------------------------------- |
| ⚪ Not Started  | Criteria has not been implemented or verified     |
| 🟡 In Progress | Implementation is in progress                      |
| 🟢 Implemented | Implementation is complete and verified            |
| 🔴 Blocked     | Implementation cannot proceed because of a blocker |

## Route

| ID                  | Criteria                                                            | Status        | Reason |
| ------------------- | ------------------------------------------------------------------- | ------------- | ------ |
| `FE_LOGIN_ROUTE_01` | `/login` displays the Admin Login page for unauthenticated users    | ⚪ Not Started |        |
| `FE_LOGIN_ROUTE_02` | Authenticated users accessing `/login` are redirected to `/admin`   | ⚪ Not Started |        |
| `FE_LOGIN_ROUTE_03` | Unauthenticated users accessing `/admin` are redirected to `/login` | ⚪ Not Started |        |

## Requirements

| ID            | Criteria                                                                                      | Status        | Reason |
| ------------- | --------------------------------------------------------------------------------------------- | ------------- | ------ |
| `FE_LOGIN_01` | Display the Admin Login page at `/login`                                                      | ⚪ Not Started |        |
| `FE_LOGIN_02` | Accept an email address and password                                                          | ⚪ Not Started |        |
| `FE_LOGIN_03` | Submit credentials to `POST /auth/login`                                                      | ⚪ Not Started |        |
| `FE_LOGIN_04` | Show a loading state while authentication is in progress                                      | ⚪ Not Started |        |
| `FE_LOGIN_05` | Show validation errors for invalid required input                                             | ⚪ Not Started |        |
| `FE_LOGIN_06` | Show a generic authentication error for invalid credentials                                   | ⚪ Not Started |        |
| `FE_LOGIN_07` | Show a generic rate-limit error for `429 AUTHENTICATION_RATE_LIMITED`                         | ⚪ Not Started |        |
| `FE_LOGIN_08` | Establish authenticated client state and navigate to `/admin` after successful authentication | ⚪ Not Started |        |
| `FE_LOGIN_09` | Do not expose authentication tokens in the UI                                                 | ⚪ Not Started |        |
| `FE_LOGIN_10` | Do not log passwords, access tokens, or refresh tokens                                        | ⚪ Not Started |        |

## UI

| ID               | Criteria                                                                                 | Status        | Reason |
| ---------------- | ---------------------------------------------------------------------------------------- | ------------- | ------ |
| `FE_LOGIN_UI_01` | Email field is implemented                                                               | ⚪ Not Started |        |
| `FE_LOGIN_UI_02` | Password field is implemented                                                            | ⚪ Not Started |        |
| `FE_LOGIN_UI_03` | Sign In action is implemented                                                            | ⚪ Not Started |        |
| `FE_LOGIN_UI_04` | Basic required-field validation is implemented                                           | ⚪ Not Started |        |
| `FE_LOGIN_UI_05` | Authentication rules remain server-side                                                  | ⚪ Not Started |        |
| `FE_LOGIN_UI_06` | No password-policy validation is added that could prevent a valid authentication request | ⚪ Not Started |        |

## API Contract

| ID                    | Criteria                                                                        | Status        | Reason |
| --------------------- | ------------------------------------------------------------------------------- | ------------- | ------ |
| `FE_LOGIN_API_01`     | Frontend authentication service owns the received authentication state          | ⚪ Not Started |        |
| `FE_LOGIN_API_02`     | Raw authentication tokens are never rendered, logged, or exposed through the UI | ⚪ Not Started |        |
| `FE_LOGIN_API_ERR_01` | `400 INVALID_REQUEST` is handled correctly                                      | ⚪ Not Started |        |
| `FE_LOGIN_API_ERR_02` | `401 INVALID_CREDENTIALS` is handled with a generic authentication error        | ⚪ Not Started |        |
| `FE_LOGIN_API_ERR_03` | `429 AUTHENTICATION_RATE_LIMITED` is handled with a generic rate-limit error    | ⚪ Not Started |        |
| `FE_LOGIN_API_SEC_01` | Frontend does not distinguish whether an account exists                         | ⚪ Not Started |        |

## UI States

| ID                  | Criteria                                                                | Status        | Reason |
| ------------------- | ----------------------------------------------------------------------- | ------------- | ------ |
| `FE_LOGIN_STATE_01` | Initial state shows the empty login form                                | ⚪ Not Started |        |
| `FE_LOGIN_STATE_02` | Editing state allows credential entry                                   | ⚪ Not Started |        |
| `FE_LOGIN_STATE_03` | Submitting state disables submission and shows loading state            | ⚪ Not Started |        |
| `FE_LOGIN_STATE_04` | Invalid Request state displays the validation/request error             | ⚪ Not Started |        |
| `FE_LOGIN_STATE_05` | Invalid Credentials state displays the generic authentication error     | ⚪ Not Started |        |
| `FE_LOGIN_STATE_06` | Rate Limited state displays the generic rate-limit error                | ⚪ Not Started |        |
| `FE_LOGIN_STATE_07` | Success state establishes authenticated state and navigates to `/admin` | ⚪ Not Started |        |
| `FE_LOGIN_STATE_08` | Duplicate submissions are prevented while authentication is pending     | ⚪ Not Started |        |

## Security

| ID                | Criteria                                                                       | Status        | Reason |
| ----------------- | ------------------------------------------------------------------------------ | ------------- | ------ |
| `FE_LOGIN_SEC_01` | Use HTTPS                                                                      | ⚪ Not Started |        |
| `FE_LOGIN_SEC_02` | Send credentials only to `POST /auth/login`                                    | ⚪ Not Started |        |
| `FE_LOGIN_SEC_03` | Do not send tokens in query parameters                                         | ⚪ Not Started |        |
| `FE_LOGIN_SEC_04` | Do not send tokens to unrelated endpoints                                      | ⚪ Not Started |        |
| `FE_LOGIN_SEC_05` | Never log passwords                                                            | ⚪ Not Started |        |
| `FE_LOGIN_SEC_06` | Never log access tokens                                                        | ⚪ Not Started |        |
| `FE_LOGIN_SEC_07` | Never log refresh tokens                                                       | ⚪ Not Started |        |
| `FE_LOGIN_SEC_08` | Never display tokens                                                           | ⚪ Not Started |        |
| `FE_LOGIN_SEC_09` | Do not expose authentication credentials through error messages                | ⚪ Not Started |        |
| `FE_LOGIN_SEC_10` | Do not reveal whether an account exists                                        | ⚪ Not Started |        |
| `FE_LOGIN_SEC_11` | Treat the backend as the authority for authentication                          | ⚪ Not Started |        |
| `FE_LOGIN_SEC_12` | Treat `401 Unauthorized` as unauthenticated                                    | ⚪ Not Started |        |
| `FE_LOGIN_SEC_13` | Do not implement authentication or account-state rules independently in the UI | ⚪ Not Started |        |

## Accessibility

| ID                 | Criteria                                                                         | Status        | Reason |
| ------------------ | -------------------------------------------------------------------------------- | ------------- | ------ |
| `FE_LOGIN_A11Y_01` | Use a visible label for each form field                                          | ⚪ Not Started |        |
| `FE_LOGIN_A11Y_02` | Associate validation errors with their fields when applicable                    | ⚪ Not Started |        |
| `FE_LOGIN_A11Y_03` | Support keyboard submission                                                      | ⚪ Not Started |        |
| `FE_LOGIN_A11Y_04` | Provide visible focus states                                                     | ⚪ Not Started |        |
| `FE_LOGIN_A11Y_05` | Use semantic form controls                                                       | ⚪ Not Started |        |
| `FE_LOGIN_A11Y_06` | Communicate loading and error states to assistive technologies where appropriate | ⚪ Not Started |        |
| `FE_LOGIN_A11Y_07` | Keep the Sign In action accessible while the form is usable                      | ⚪ Not Started |        |

## Testing — Unit

| ID                      | Criteria                                                      | Status        | Reason |
| ----------------------- | ------------------------------------------------------------- | ------------- | ------ |
| `FE_LOGIN_TEST_UNIT_01` | Login form renders correctly                                  | ⚪ Not Started |        |
| `FE_LOGIN_TEST_UNIT_02` | Required-field validation works                               | ⚪ Not Started |        |
| `FE_LOGIN_TEST_UNIT_03` | Loading state is displayed correctly                          | ⚪ Not Started |        |
| `FE_LOGIN_TEST_UNIT_04` | Error states are rendered correctly                           | ⚪ Not Started |        |
| `FE_LOGIN_TEST_UNIT_05` | Submit action is disabled while authentication is in progress | ⚪ Not Started |        |

## Testing — Integration

| ID                     | Criteria                                               | Status        | Reason |
| ---------------------- | ------------------------------------------------------ | ------------- | ------ |
| `FE_LOGIN_TEST_INT_01` | Form submission works correctly                        | ⚪ Not Started |        |
| `FE_LOGIN_TEST_INT_02` | `POST /auth/login` request is sent correctly           | ⚪ Not Started |        |
| `FE_LOGIN_TEST_INT_03` | Successful authentication is handled correctly         | ⚪ Not Started |        |
| `FE_LOGIN_TEST_INT_04` | `400 INVALID_REQUEST` is handled correctly             | ⚪ Not Started |        |
| `FE_LOGIN_TEST_INT_05` | `401 INVALID_CREDENTIALS` is handled correctly         | ⚪ Not Started |        |
| `FE_LOGIN_TEST_INT_06` | `429 AUTHENTICATION_RATE_LIMITED` is handled correctly | ⚪ Not Started |        |
| `FE_LOGIN_TEST_INT_07` | Successful authentication navigates to `/admin`        | ⚪ Not Started |        |
| `FE_LOGIN_TEST_INT_08` | Authenticated client state is created correctly        | ⚪ Not Started |        |

## Testing — E2E

| ID                     | Criteria                                         | Status        | Reason |
| ---------------------- | ------------------------------------------------ | ------------- | ------ |
| `FE_LOGIN_TEST_E2E_01` | User can open `/login`                           | ⚪ Not Started |        |
| `FE_LOGIN_TEST_E2E_02` | User can enter valid credentials                 | ⚪ Not Started |        |
| `FE_LOGIN_TEST_E2E_03` | User can submit the Login form                   | ⚪ Not Started |        |
| `FE_LOGIN_TEST_E2E_04` | Successful login displays the Admin Shell        | ⚪ Not Started |        |
| `FE_LOGIN_TEST_E2E_05` | Logout returns the user to `/login`              | ⚪ Not Started |        |
| `FE_LOGIN_TEST_E2E_06` | Invalid credentials do not enter the Admin Shell | ⚪ Not Started |        |

## Testing — Manual

| ID                        | Criteria                                                 | Status        | Reason |
| ------------------------- | -------------------------------------------------------- | ------------- | ------ |
| `FE_LOGIN_TEST_MANUAL_01` | Visual layout is correct                                 | ⚪ Not Started |        |
| `FE_LOGIN_TEST_MANUAL_02` | Keyboard interaction works correctly                     | ⚪ Not Started |        |
| `FE_LOGIN_TEST_MANUAL_03` | Loading behavior is correct                              | ⚪ Not Started |        |
| `FE_LOGIN_TEST_MANUAL_04` | Error messages are correct                               | ⚪ Not Started |        |
| `FE_LOGIN_TEST_MANUAL_05` | Responsive behavior is correct                           | ⚪ Not Started |        |
| `FE_LOGIN_TEST_MANUAL_06` | Accessibility behavior is correct                        | ⚪ Not Started |        |
| `FE_LOGIN_TEST_MANUAL_07` | Successful transition to the Admin Shell works correctly | ⚪ Not Started |        |
