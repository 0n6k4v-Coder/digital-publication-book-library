# Authentication Domain Ready For Implement Design

## Table of Contents

1. [Requirements](#1-requirements)

   * [1.1 Authentication](#11-authentication)
   * [1.2 Token and Session Lifecycle](#12-token-and-session-lifecycle)
   * [1.3 Integration](#13-integration)
   * [1.4 Non-Functional Requirements](#14-non-functional-requirements)

2. [Security](#2-security)

   * [2.1 Security Requirements](#21-security-requirements)
   * [2.2 Security Decisions](#22-security-decisions)

3. [Design Decisions](#3-design-decisions)

4. [Data Model](#4-data-model)

5. [Use Cases](#5-use-cases)

6. [API Contract](#6-api-contract)

7. [Integration Contract](#7-integration-contract)

8. [Implementation Status](#8-implementation-status)

---

# 1. Requirements

## 1.1 Authentication

| ID             | Requirement                                                               |
| -------------- | ------------------------------------------------------------------------- |
| `AU_REQ_FC_01` | Authenticate an account using email and password.                         |
| `AU_REQ_FC_02` | Use the Account domain's email identity as the authentication identifier. |
| `AU_REQ_FC_03` | Verify the supplied password against the stored password hash.            |
| `AU_REQ_FC_04` | Reject authentication for inactive accounts.                              |
| `AU_REQ_FC_05` | Reject authentication for soft-deleted accounts.                          |
| `AU_REQ_FC_06` | Create an authenticated principal after successful authentication.        |
| `AU_REQ_FC_07` | Create an authentication session after successful authentication.         |
| `AU_REQ_FC_08` | Issue a bearer access token after successful authentication.              |
| `AU_REQ_FC_09` | Validate bearer access tokens for protected API requests.                 |
| `AU_REQ_FC_10` | Revoke authentication sessions.                                           |

## 1.2 Token and Session Lifecycle

| ID             | Requirement                                                |
| -------------- | ---------------------------------------------------------- |
| `AU_REQ_FC_11` | Assign a unique identifier to each authentication session. |
| `AU_REQ_FC_12` | Access tokens must expire.                                 |
| `AU_REQ_FC_13` | Support refresh tokens.                                    |
| `AU_REQ_FC_14` | Rotate refresh tokens after successful refresh.            |
| `AU_REQ_FC_15` | Reject expired access tokens.                              |
| `AU_REQ_FC_16` | Reject expired refresh tokens.                             |
| `AU_REQ_FC_17` | Reject revoked authentication sessions.                    |
| `AU_REQ_FC_18` | Reject replayed refresh tokens.                            |

## 1.3 Integration

| ID             | Requirement                                                                              |
| -------------- | ---------------------------------------------------------------------------------------- |
| `AU_REQ_FC_19` | Authentication must obtain authentication-relevant account data from the Account domain. |
| `AU_REQ_FC_20` | Authentication must not own canonical account profile data.                              |
| `AU_REQ_FC_21` | Authentication must not own the Account password hash.                                   |
| `AU_REQ_FC_22` | Authentication must provide the authenticated account ID to downstream consumers.        |
| `AU_REQ_FC_23` | Authentication must provide an `AuthenticatedPrincipal`.                                 |
| `AU_REQ_FC_24` | Authentication must not decide roles or permissions.                                     |
| `AU_REQ_FC_25` | Authorization must consume the authenticated principal for access decisions.             |

## 1.4 Non-Functional Requirements

| ID                 | Requirement                                                                             |
| ------------------ | --------------------------------------------------------------------------------------- |
| `AU_REQ_NON_FC_01` | Passwords, access tokens, refresh tokens, and authorization headers must not be logged. |
| `AU_REQ_NON_FC_02` | Raw access tokens must not be persisted.                                                |
| `AU_REQ_NON_FC_03` | Raw refresh tokens must not be persisted.                                               |
| `AU_REQ_NON_FC_04` | Authentication endpoints must use HTTPS/TLS.                                            |
| `AU_REQ_NON_FC_05` | Failed password authentication attempts must be rate-limited.                           |
| `AU_REQ_NON_FC_06` | Authentication failures must not reveal unnecessary account-existence information.      |
| `AU_REQ_NON_FC_07` | Authentication responses must use `Cache-Control: no-store`.                            |

---

# 2. Security

## 2.1 Security Requirements

| ID              | Requirement                                                                                                    |
| --------------- | -------------------------------------------------------------------------------------------------------------- |
| `AU_SEC_REQ_01` | Authenticate using the HTTP Bearer scheme for protected Admin API requests.                                    |
| `AU_SEC_REQ_02` | Accept bearer credentials only in the `Authorization` header.                                                  |
| `AU_SEC_REQ_03` | Do not accept bearer credentials in query parameters or request bodies.                                        |
| `AU_SEC_REQ_04` | Missing authentication must return `401 Unauthorized`.                                                         |
| `AU_SEC_REQ_05` | Invalid, expired, or revoked bearer authentication must return `401 Unauthorized`.                             |
| `AU_SEC_REQ_06` | Missing authentication must return `WWW-Authenticate: Bearer realm="admin-api"`.                               |
| `AU_SEC_REQ_07` | Invalid bearer authentication must return `WWW-Authenticate: Bearer realm="admin-api", error="invalid_token"`. |
| `AU_SEC_REQ_08` | Authentication errors must use `application/problem+json`.                                                     |
| `AU_SEC_REQ_09` | Authentication must not expose passwords, password hashes, tokens, or token verifier material.                 |

## 2.2 Security Decisions

### 2.2.1 Credential Verification

| ID                         | Decision                  | Definition                                                                        |
| -------------------------- | ------------------------- | --------------------------------------------------------------------------------- |
| `AU_SEC_DEC_CREDENTIAL_01` | Authentication Identifier | Email.                                                                            |
| `AU_SEC_DEC_CREDENTIAL_02` | Email Handling            | Use the Account domain's canonical and normalized email rules.                    |
| `AU_SEC_DEC_CREDENTIAL_03` | Password Verification     | Verify the supplied password against the Account password hash.                   |
| `AU_SEC_DEC_CREDENTIAL_04` | Account State             | Authentication requires `status = active` and `deleted_at IS NULL`.               |
| `AU_SEC_DEC_CREDENTIAL_05` | Failed Authentication     | Return a generic credential failure without revealing whether the account exists. |
| `AU_SEC_DEC_CREDENTIAL_06` | Rate Limiting             | Rate-limit failed authentication attempts.                                        |

### 2.2.2 Access Tokens

| ID                     | Decision      | Definition                                                                            |
| ---------------------- | ------------- | ------------------------------------------------------------------------------------- |
| `AU_SEC_DEC_ACCESS_01` | Scheme        | HTTP Bearer.                                                                          |
| `AU_SEC_DEC_ACCESS_02` | Location      | `Authorization` header only.                                                          |
| `AU_SEC_DEC_ACCESS_03` | Format        | `Authorization: Bearer <token>`.                                                      |
| `AU_SEC_DEC_ACCESS_04` | Type          | Opaque bearer token.                                                                  |
| `AU_SEC_DEC_ACCESS_05` | Generation    | Cryptographically secure random generation.                                           |
| `AU_SEC_DEC_ACCESS_06` | Storage       | Store only a non-reversible verifier representation.                                  |
| `AU_SEC_DEC_ACCESS_07` | Expiration    | Every access token has an expiration time.                                            |
| `AU_SEC_DEC_ACCESS_08` | Revocation    | Revocation of the session invalidates its access token.                               |
| `AU_SEC_DEC_ACCESS_09` | Authorization | Access tokens authenticate the principal; they do not define application permissions. |

### 2.2.3 Refresh Tokens

| ID                      | Decision        | Definition                                                                           |
| ----------------------- | --------------- | ------------------------------------------------------------------------------------ |
| `AU_SEC_DEC_REFRESH_01` | Purpose         | Obtain new access credentials for an existing session.                               |
| `AU_SEC_DEC_REFRESH_02` | Type            | Opaque token.                                                                        |
| `AU_SEC_DEC_REFRESH_03` | Storage         | Store only a non-reversible verifier representation.                                 |
| `AU_SEC_DEC_REFRESH_04` | Rotation        | Successful refresh invalidates the presented refresh token and issues a replacement. |
| `AU_SEC_DEC_REFRESH_05` | Replay          | A previously used refresh token must be rejected.                                    |
| `AU_SEC_DEC_REFRESH_06` | Session Binding | Each refresh token belongs to one authentication session.                            |
| `AU_SEC_DEC_REFRESH_07` | Revocation      | Session revocation invalidates its refresh tokens.                                   |
| `AU_SEC_DEC_REFRESH_08` | Expiration      | Every refresh token has an expiration time.                                          |

### 2.2.4 Authentication Failures

| ID                      | Decision           | Definition                                                                   |
| ----------------------- | ------------------ | ---------------------------------------------------------------------------- |
| `AU_SEC_DEC_FAILURE_01` | Missing Credential | `401` + `WWW-Authenticate: Bearer realm="admin-api"`.                        |
| `AU_SEC_DEC_FAILURE_02` | Invalid Credential | `401` + `WWW-Authenticate: Bearer realm="admin-api", error="invalid_token"`. |
| `AU_SEC_DEC_FAILURE_03` | Expired Token      | Treat as invalid bearer authentication.                                      |
| `AU_SEC_DEC_FAILURE_04` | Revoked Token      | Treat as invalid bearer authentication.                                      |
| `AU_SEC_DEC_FAILURE_05` | Invalid Login      | Return generic authentication failure.                                       |
| `AU_SEC_DEC_FAILURE_06` | Account Disabled   | Authentication must fail.                                                    |
| `AU_SEC_DEC_FAILURE_07` | Account Deleted    | Authentication must fail.                                                    |

### 2.2.5 Principal

| ID                        | Decision            | Definition                                   |
| ------------------------- | ------------------- | -------------------------------------------- |
| `AU_SEC_DEC_PRINCIPAL_01` | Identity            | Principal contains `account_id`.             |
| `AU_SEC_DEC_PRINCIPAL_02` | Session             | Principal contains `session_id`.             |
| `AU_SEC_DEC_PRINCIPAL_03` | Authentication Time | Principal contains `authenticated_at`.       |
| `AU_SEC_DEC_PRINCIPAL_04` | Roles               | Roles are not owned by Authentication.       |
| `AU_SEC_DEC_PRINCIPAL_05` | Permissions         | Permissions are not owned by Authentication. |

---

# 3. Design Decisions

## 3.1 Authentication Responsibilities

Authentication owns:

```text
Credential verification
Authentication session creation
Access-token issuance
Access-token validation
Refresh-token issuance
Refresh-token rotation
Session revocation
AuthenticatedPrincipal creation
```

Authentication does not own:

```text
Account profile
Account lifecycle
Canonical email storage
Password hash storage
Roles
Permissions
Authorization policies
```

## 3.2 Authentication Flow

```text
Login
  ↓
Find account by email
  ↓
Verify password
  ↓
Check account is active and not deleted
  ↓
Create session
  ↓
Issue access token + refresh token
  ↓
Return authentication response
```

## 3.3 Protected Request Flow

```text
HTTP Request
  ↓
Extract Bearer token
  ↓
Validate token
  ↓
Validate session
  ↓
Validate account authentication state
  ↓
Create AuthenticatedPrincipal
  ↓
Authorization
```

Authentication stops before the authorization decision.

---

# 4. Data Model

## 4.1 `authentication_session`

**ID:** `AU_DM_01`

| Column                  | Type          | Null | Constraint              |
| ----------------------- | ------------- | ---: | ----------------------- |
| `id`                    | `uuid`        |   No | PK                      |
| `account_id`            | `uuid`        |   No | References `account.id` |
| `created_at`            | `timestamptz` |   No |                         |
| `expires_at`            | `timestamptz` |   No |                         |
| `last_authenticated_at` | `timestamptz` |   No |                         |
| `revoked_at`            | `timestamptz` |  Yes |                         |
| `revocation_reason`     | `text`        |  Yes |                         |

## 4.2 `authentication_refresh_token`

**ID:** `AU_DM_02`

| Column       | Type          | Null | Constraint                       |
| ------------ | ------------- | ---: | -------------------------------- |
| `id`         | `uuid`        |   No | PK                               |
| `session_id` | `uuid`        |   No | FK → `authentication_session.id` |
| `token_hash` | `text`        |   No | Unique verifier representation   |
| `created_at` | `timestamptz` |   No |                                  |
| `expires_at` | `timestamptz` |   No |                                  |
| `used_at`    | `timestamptz` |  Yes |                                  |
| `revoked_at` | `timestamptz` |  Yes |                                  |

## 4.3 Token Storage Rule

```text
Raw token
   ↓
One-way verifier
   ↓
Persist verifier only
```

Raw access and refresh tokens must never be stored.

---

# 5. Use Cases

## 5.1 Authenticate Account

**ID:** `AU_UC_01`

| Item   | Definition                                            |
| ------ | ----------------------------------------------------- |
| Actor  | Unauthenticated Client                                |
| Input  | Email, Password                                       |
| Result | Authentication session + access token + refresh token |

### Rules

1. Process email using Account email rules.
2. Retrieve account credential information.
3. Verify password.
4. Require `status = active`.
5. Require `deleted_at IS NULL`.
6. Apply authentication rate limiting.
7. Create authentication session.
8. Issue access token.
9. Issue refresh token.

## 5.2 Refresh Authentication

**ID:** `AU_UC_02`

| Item   | Definition                           |
| ------ | ------------------------------------ |
| Actor  | Client with Refresh Token            |
| Input  | Refresh Token                        |
| Result | New Access Token + new Refresh Token |

### Rules

1. Validate refresh token.
2. Require token not expired.
3. Require token not already used.
4. Require session not revoked.
5. Require account authentication state to remain valid.
6. Mark current refresh token used.
7. Issue replacement refresh token.
8. Issue new access token.

## 5.3 Revoke Authentication

**ID:** `AU_UC_03`

| Item   | Definition              |
| ------ | ----------------------- |
| Actor  | Authenticated Principal |
| Input  | Authentication Session  |
| Result | Session revoked         |

### Rules

1. Identify the session.
2. Verify the session belongs to the authenticated principal.
3. Mark the session revoked.
4. Invalidate associated authentication tokens.

## 5.4 Validate Bearer Authentication

**ID:** `AU_UC_04`

| Item   | Definition                        |
| ------ | --------------------------------- |
| Actor  | Protected API Request             |
| Input  | `Authorization: Bearer <token>`   |
| Result | `AuthenticatedPrincipal` or `401` |

### Rules

1. Require `Authorization` header.
2. Require `Bearer` scheme.
3. Validate token.
4. Validate expiration.
5. Validate session state.
6. Validate account authentication state.
7. Create `AuthenticatedPrincipal`.

This use case must not evaluate roles or permissions.

---

# 6. API Contract

## 6.1 API Rules

| Rule                  | Definition                      |
| --------------------- | ------------------------------- |
| Base Path             | `/auth`                         |
| Success Content-Type  | `application/json`              |
| Error Content-Type    | `application/problem+json`      |
| Authentication        | `Authorization: Bearer <token>` |
| Token Query Parameter | Not accepted                    |
| Token Body Parameter  | Not accepted                    |
| Caching               | `Cache-Control: no-store`       |
| TLS                   | Required                        |
| Authorization         | Handled by Authorization domain |

## 6.2 Authentication Endpoints

| ID          | Method | Endpoint        | Use Case   |
| ----------- | ------ | --------------- | ---------- |
| `AU_API_01` | `POST` | `/auth/login`   | `AU_UC_01` |
| `AU_API_02` | `POST` | `/auth/refresh` | `AU_UC_02` |
| `AU_API_03` | `POST` | `/auth/logout`  | `AU_UC_03` |

Bearer validation is middleware, not a public endpoint.

## 6.3 Authenticate Account

### Request

`POST /auth/login`

```json
{
  "email": "admin@example.com",
  "password": "example-secure-password"
}
```

### Success

**`200 OK`**

```json
{
  "access_token": "opaque-access-token",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "opaque-refresh-token",
  "refresh_expires_in": 2592000
}
```

Headers:

```text
Content-Type: application/json
Cache-Control: no-store
```

### Errors

| Status | Code                          |
| ------ | ----------------------------- |
| `400`  | `INVALID_REQUEST`             |
| `401`  | `INVALID_CREDENTIALS`         |
| `429`  | `AUTHENTICATION_RATE_LIMITED` |

## 6.4 Refresh Authentication

### Request

`POST /auth/refresh`

```json
{
  "refresh_token": "opaque-refresh-token"
}
```

### Success

**`200 OK`**

```json
{
  "access_token": "new-opaque-access-token",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "new-opaque-refresh-token",
  "refresh_expires_in": 2592000
}
```

### Errors

| Status | Code                    |
| ------ | ----------------------- |
| `400`  | `INVALID_REQUEST`       |
| `401`  | `INVALID_REFRESH_TOKEN` |

## 6.5 Revoke Authentication

### Request

`POST /auth/logout`

The current authenticated session is revoked.

### Success

**`204 No Content`**

Headers:

```text
Cache-Control: no-store
```

### Errors

| Status | Code           |
| ------ | -------------- |
| `401`  | `UNAUTHORIZED` |

## 6.6 Protected Request Authentication

### Missing authentication

```text
HTTP/1.1 401 Unauthorized
WWW-Authenticate: Bearer realm="admin-api"
Content-Type: application/problem+json
Cache-Control: no-store
```

### Invalid authentication

```text
HTTP/1.1 401 Unauthorized
WWW-Authenticate: Bearer realm="admin-api", error="invalid_token"
Content-Type: application/problem+json
Cache-Control: no-store
```

## 6.7 Authenticated Principal

**ID:** `AU_CONTRACT_01`

```text
AuthenticatedPrincipal
```

| Field              | Definition                          |
| ------------------ | ----------------------------------- |
| `account_id`       | Authenticated Account ID            |
| `session_id`       | Authentication session ID           |
| `authenticated_at` | Time authentication was established |

The principal contains no roles or permissions.

---

# 7. Integration Contract

## 7.1 Account → Authentication

Account provides:

```text
Authentication identifier
Password hash
Account status
Deleted state
```

Authentication consumes this data for verification.

Authentication does not update Account credentials.

## 7.2 Authentication → Authorization

Authentication provides:

```text
AuthenticatedPrincipal
```

Authorization uses the principal to evaluate permissions.

Authentication does not contain role or permission logic.

## 7.3 Application Request Pipeline

```text
Request
  ↓
Authentication
  ↓
AuthenticatedPrincipal
  ↓
Authorization
  ↓
Domain Handler
```

Failure at Authentication:

```text
401 Unauthorized
```

Failure at Authorization:

```text
403 Forbidden
```

---

# 8. Implementation Status

## 8.1 Requirements

| ID                                    | Status             | Reason                                     |
| ------------------------------------- | ------------------ | ------------------------------------------ |
| `AU_REQ_FC_01`–`AU_REQ_FC_25`         | 🔴 Not Implemented | Authentication domain not implemented yet. |
| `AU_REQ_NON_FC_01`–`AU_REQ_NON_FC_07` | 🔴 Not Implemented | Authentication domain not implemented yet. |

## 8.2 Security

| ID                              | Status             | Reason                                                                                                                    |
| ------------------------------- | ------------------ | ------------------------------------------------------------------------------------------------------------------------- |
| `AU_SEC_REQ_01`–`AU_SEC_REQ_09` | 🔴 Not Implemented | Authentication enforcement not implemented.                                                                               |
| `AU_SEC_DEC_CREDENTIAL_01`–`06` | 🔴 Not Implemented | Credential authentication flow not implemented.                                                                           |
| `AU_SEC_DEC_ACCESS_01`–`09`     | 🔴 Not Implemented | Access-token system not implemented.                                                                                      |
| `AU_SEC_DEC_REFRESH_01`–`08`    | 🔴 Not Implemented | Refresh-token system not implemented.                                                                                     |
| `AU_SEC_DEC_FAILURE_01`–`07`    | 🟡 Partial         | Shared error handling exists, but complete Authentication behavior is not implemented.                                    |
| `AU_SEC_DEC_PRINCIPAL_01`–`05`  | 🟡 Partial         | `AuthenticatedAdmin` exists as a temporary upstream identity boundary; final Authentication principal is not implemented. |

## 8.3 Design Decisions

| ID                              | Status             | Reason |
| ------------------------------- | ------------------ | ------ |
| Authentication responsibilities | 🔴 Not Implemented | —      |
| Authentication flow             | 🔴 Not Implemented | —      |
| Protected request flow          | 🔴 Not Implemented | —      |

## 8.4 Data Model

| ID         | Description                    | Status             | Reason |
| ---------- | ------------------------------ | ------------------ | ------ |
| `AU_DM_01` | `authentication_session`       | 🔴 Not Implemented | —      |
| `AU_DM_02` | `authentication_refresh_token` | 🔴 Not Implemented | —      |

## 8.5 Use Cases

| ID         | Description                    | Status             | Reason |
| ---------- | ------------------------------ | ------------------ | ------ |
| `AU_UC_01` | Authenticate Account           | 🔴 Not Implemented | —      |
| `AU_UC_02` | Refresh Authentication         | 🔴 Not Implemented | —      |
| `AU_UC_03` | Revoke Authentication          | 🔴 Not Implemented | —      |
| `AU_UC_04` | Validate Bearer Authentication | 🔴 Not Implemented | —      |

## 8.6 API Contract

| ID               | Description              | Status             | Reason                                                                 |
| ---------------- | ------------------------ | ------------------ | ---------------------------------------------------------------------- |
| `AU_API_01`      | `POST /auth/login`       | 🔴 Not Implemented | —                                                                      |
| `AU_API_02`      | `POST /auth/refresh`     | 🔴 Not Implemented | —                                                                      |
| `AU_API_03`      | `POST /auth/logout`      | 🔴 Not Implemented | —                                                                      |
| `AU_CONTRACT_01` | `AuthenticatedPrincipal` | 🟡 Partial         | Temporary `AuthenticatedAdmin` exists; final contract not implemented. |

---

# Domain Boundary

```text
ACCOUNT
  Owns:
    account
    account_credentials
    email
    password hash
    account lifecycle

AUTHENTICATION
  Owns:
    credential verification
    authentication sessions
    access tokens
    refresh tokens
    token validation
    session revocation
    AuthenticatedPrincipal

AUTHORIZATION
  Owns:
    roles
    permissions
    role assignments
    authorization policies
    access decisions
```

Authentication answers:

```text
WHO IS THE CALLER?
```

Authorization answers:

```text
WHAT MAY THE CALLER DO?
```

The business domain answers:

```text
WHAT HAPPENS WHEN ACCESS IS ALLOWED?
```
