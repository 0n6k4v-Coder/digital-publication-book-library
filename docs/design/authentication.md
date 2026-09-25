# Authentication Domain Ready For Implementation Design

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

9. [Domain Boundary](#9-domain-boundary)

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

| ID                 | Requirement                                                                                             |
| ------------------ | -------------------------------------------------------------------------------------------------------- |
| `AU_REQ_NON_FC_01` | Passwords, raw access tokens, raw refresh tokens, and `Authorization` header values must not be logged. |
| `AU_REQ_NON_FC_02` | Raw access tokens must not be persisted.                                                                |
| `AU_REQ_NON_FC_03` | Raw refresh tokens must not be persisted.                                                               |
| `AU_REQ_NON_FC_04` | Authentication endpoints must use HTTPS/TLS. TLS terminates at the backend application through Rustls. TLS 1.3 must be preferred; TLS 1.2 may be supported for compatibility; TLS 1.0 and TLS 1.1 must not be negotiated. The backend must not expose Authentication endpoints through plaintext HTTP. If a reverse proxy or load balancer is used, its connection to the backend must also be TLS-protected. The application must not trust forwarded scheme headers as proof of HTTPS. Certificate and private-key provisioning is deployment-managed and private-key material must never be logged, returned, or committed to source control. |
| `AU_REQ_NON_FC_05` | Failed password authentication attempts must be rate-limited.                                           |
| `AU_REQ_NON_FC_06` | Authentication failures must not reveal unnecessary account-existence information.                      |
| `AU_REQ_NON_FC_07` | Authentication responses must use `Cache-Control: no-store`.                                             |

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
| `AU_SEC_DEC_CREDENTIAL_06` | Rate Limiting             | Failed password authentication attempts are throttled using two independent server-side sliding-window limits. The first limit allows at most 10 failed authentication attempts for the same normalized Account email identity within any rolling 15-minute window. The second limit allows at most 50 failed authentication attempts from the same source IP address within any rolling 15-minute window. Both limits are evaluated independently, and a login request is allowed only when both limits are below their thresholds. The source IP is the server-observed client connection address; forwarded client-address headers must not be trusted unless they are supplied through an explicitly configured trusted-proxy boundary. Rate-limit state must be maintained server-side and shared across all application instances. When either limit is exceeded, the request is rejected with `429 AUTHENTICATION_RATE_LIMITED` before credential verification continues. The response must remain generic and must not disclose which limit was exceeded, the current attempt count, remaining attempts, or a precise reset time. The email-identity failure counter is reset after successful authentication for that identity. The source-IP counter is not reset by another account's successful authentication and expires naturally as entries leave the rolling 15-minute window. Rate limiting must not change Account status, disable the Account, revoke existing sessions, or otherwise modify Account lifecycle state. The policy applies to `AU_UC_01` login authentication and does not apply to `AU_UC_02` refresh or `AU_UC_03` revoke authentication. |

### 2.2.2 Access Tokens

| ID                     | Decision               | Definition                                                                                                                                                 |
| ---------------------- | ---------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `AU_SEC_DEC_ACCESS_01` | Scheme                 | Access tokens use the HTTP Bearer scheme.                                                                                                                  |
| `AU_SEC_DEC_ACCESS_02` | Transport              | Access tokens are accepted only from the `Authorization` request header.                                                                                   |
| `AU_SEC_DEC_ACCESS_03` | Format                 | Header format is `Authorization: Bearer <token>`.                                                                                                          |
| `AU_SEC_DEC_ACCESS_04` | Type                   | Access tokens are opaque bearer strings; clients must not depend on their internal representation.                                                         |
| `AU_SEC_DEC_ACCESS_05` | Generation             | Access tokens are generated from a cryptographically secure random source.                                                                                 |
| `AU_SEC_DEC_ACCESS_06` | Storage                | Only a one-way verifier representation is persisted; the raw access token is never stored.                                                                 |
| `AU_SEC_DEC_ACCESS_07` | Expiration             | Access tokens expire 3600 seconds after issuance.                                                                                                          |
| `AU_SEC_DEC_ACCESS_08` | Revocation             | Revoking an authentication session invalidates all access tokens bound to that session.                                                                    |
| `AU_SEC_DEC_ACCESS_09` | Authorization Boundary | An access token authenticates an account/session principal; it does not grant roles or permissions.                                                        |
| `AU_SEC_DEC_ACCESS_10` | Entropy                | Each access token is generated from at least 256 bits of cryptographically secure random entropy.                                                          |
| `AU_SEC_DEC_ACCESS_11` | Verifier               | The persisted access-token verifier is the SHA-256 digest of the exact issued token string.                                                                |
| `AU_SEC_DEC_ACCESS_12` | Session Binding        | Each access token is bound to exactly one `authentication_session`.                                                                                        |
| `AU_SEC_DEC_ACCESS_13` | Validation             | Validation must verify the token, token expiry, bound session, session state, and account authentication state together.                                   |
| `AU_SEC_DEC_ACCESS_14` | Lookup                 | Token lookup is performed exclusively by the server-computed SHA-256 verifier; client-supplied identity, session, role, or permission data is not trusted. |
| `AU_SEC_DEC_ACCESS_15` | Reuse                  | A valid access token may be reused until its expiration or until its bound session/account becomes invalid.                                                |
| `AU_SEC_DEC_ACCESS_16` | Raw Token Handling     | The raw access token is returned only at issuance and must never be persisted or logged.                                                                   |

Access-token generation and storage:

```text
cryptographically secure random bytes
        ↓
opaque access token
        ↓
SHA-256(exact token string)
        ↓
persist verifier + session binding + expiration
```

The database stores only the verifier.

### 2.2.3 Refresh Tokens

| ID                      | Decision        | Definition                                                                           |
| ----------------------- | --------------- | ------------------------------------------------------------------------------------ |
| `AU_SEC_DEC_REFRESH_01` | Purpose         | Obtain new access credentials for an existing authentication session.                |
| `AU_SEC_DEC_REFRESH_02` | Type            | Opaque token.                                                                        |
| `AU_SEC_DEC_REFRESH_03` | Storage         | Persist only a SHA-256 verifier of the exact refresh-token string.                   |
| `AU_SEC_DEC_REFRESH_04` | Rotation        | Successful refresh invalidates the presented refresh token and issues a replacement. |
| `AU_SEC_DEC_REFRESH_05` | Replay          | A previously used refresh token must be rejected.                                    |
| `AU_SEC_DEC_REFRESH_06` | Session Binding | Each refresh token belongs to exactly one authentication session.                    |
| `AU_SEC_DEC_REFRESH_07` | Revocation      | Session revocation invalidates its refresh tokens.                                   |
| `AU_SEC_DEC_REFRESH_08` | Expiration      | Every refresh token expires 2592000 seconds after issuance.                          |

### 2.2.4 Authentication Failures

| ID                      | Decision           | Definition                                                                   |
| ----------------------- | ------------------ | ---------------------------------------------------------------------------- |
| `AU_SEC_DEC_FAILURE_01` | Missing Credential | `401` + `WWW-Authenticate: Bearer realm="admin-api"`.                        |
| `AU_SEC_DEC_FAILURE_02` | Invalid Credential | `401` + `WWW-Authenticate: Bearer realm="admin-api", error="invalid_token"`. |
| `AU_SEC_DEC_FAILURE_03` | Expired Token      | Treat as invalid bearer authentication.                                      |
| `AU_SEC_DEC_FAILURE_04` | Revoked Session    | Treat as invalid bearer authentication.                                      |
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

### 2.2.6 Authentication Sessions

| ID                      | Decision     | Definition                                                                                                                   |
| ----------------------- | ------------ | ---------------------------------------------------------------------------------------------------------------------------- |
| `AU_SEC_DEC_SESSION_01` | Purpose      | An authentication session is the server-side lifecycle record binding an authenticated account to its authentication tokens. |
| `AU_SEC_DEC_SESSION_02` | Identifier   | Each authentication session has a unique `id`.                                                                               |
| `AU_SEC_DEC_SESSION_03` | Expiration   | Every authentication session created by `AU_UC_01` has a fixed absolute lifetime of 24 hours. The lifetime begins at the time successful email/password authentication creates the session, and `expires_at` is set to that session creation time plus 24 hours. A session is invalid when `expires_at <= current_time`. The expiration is absolute and does not slide with request activity or token refresh. `AU_UC_02` refresh is not reauthentication and must not extend `expires_at` or establish a new session. A newly successful `AU_UC_01` authentication creates a new session with a new 24-hour expiration. |
| `AU_SEC_DEC_SESSION_04` | Revocation   | A revoked authentication session is invalid immediately.                                                                     |
| `AU_SEC_DEC_SESSION_05` | Token Scope  | Access and refresh tokens are valid only while their bound authentication session remains valid.                             |
| `AU_SEC_DEC_SESSION_06` | Account Bind | Each authentication session belongs to exactly one Account.                                                                  |

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

Authentication may read Account-owned authentication data required for credential verification and account-state validation.

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
Create authentication session
  ↓
Set session expiration according to session policy
  ↓
Issue access token
  ↓
Issue refresh token
  ↓
Return authentication response
```

## 3.3 Protected Request Flow

Protected requests follow this authoritative sequence:

```text
HTTP request
    ↓
Require Authorization header
    ↓
Require Bearer scheme
    ↓
Extract opaque token
    ↓
Compute SHA-256(exact token string)
    ↓
Lookup authentication_access_token by token_hash
    ↓
Require token exists
    ↓
Require token expires_at > current time
    ↓
Load authentication_session by session_id
    ↓
Require session exists
    ↓
Require session.revoked_at IS NULL
    ↓
Require session.expires_at > current time
    ↓
Load Account authentication state by account_id
    ↓
Require account.status = active
    ↓
Require account.deleted_at IS NULL
    ↓
Create AuthenticatedPrincipal
    ↓
Authorization
```

Authentication is authoritative for the resulting principal.

Authentication does not evaluate roles or permissions.

Failure of any authentication condition returns `401 Unauthorized`.

The client cannot supply or override:

```text
account_id
session_id
roles
permissions
authorization decision
```

## 3.4 Refresh Flow

```text
Refresh request
    ↓
Compute SHA-256(refresh token)
    ↓
Lookup refresh-token record
    ↓
Require token exists
    ↓
Require token not expired
    ↓
Require token not used
    ↓
Require token not revoked
    ↓
Load authentication session
    ↓
Require session exists
    ↓
Require session not revoked
    ↓
Require session not expired
    ↓
Validate Account authentication state
    ↓
Mark presented refresh token used
    ↓
Issue replacement refresh token
    ↓
Issue new access token
    ↓
Commit atomically
```

A successful refresh must invalidate the presented refresh token before the new credentials become usable.

---

# 4. Data Model

## 4.1 `authentication_session`

**ID:** `AU_DM_01`

| Column                  | Type          | Null | Constraint                             |
| ----------------------- | ------------- | ---: | -------------------------------------- |
| `id`                    | `uuid`        |   No | PK                                     |
| `account_id`            | `uuid`        |   No | FK → `account.id`; `ON DELETE CASCADE` |
| `created_at`            | `timestamptz` |   No |                                        |
| `expires_at`            | `timestamptz` |   No |                                        |
| `last_authenticated_at` | `timestamptz` |   No |                                        |
| `revoked_at`            | `timestamptz` |  Yes |                                        |
| `revocation_reason`     | `text`        |  Yes |                                        |

Rules:

* `expires_at` defines the authentication-session expiration time.
* Every authentication session created by `AU_UC_01` must use a fixed absolute lifetime of 24 hours.
* The 24-hour lifetime begins when successful email/password authentication creates the authentication session.
* `expires_at` must therefore equal the session creation time plus 24 hours.
* A session is invalid when `expires_at <= current_time`.
* The session expiration is absolute and does not slide with request activity.
* `AU_UC_02` refresh does not extend or reset `expires_at`.
* `AU_UC_02` refresh does not modify `last_authenticated_at`.
* `AU_UC_02` refresh is not a new authentication session and does not establish a new session expiration period.
* A newly successful `AU_UC_01` authentication creates a new authentication session with its own new 24-hour expiration.
* A revoked session is invalid regardless of `expires_at`.
* `last_authenticated_at` is set when account authentication succeeds.
* `last_authenticated_at` is not modified by refresh.
* Each session belongs to exactly one Account.
* When an Account is hard-deleted, its Authentication sessions are deleted automatically through `ON DELETE CASCADE`.
* Authentication session deletion is part of the same database transaction as Account hard deletion.

## 4.2 `authentication_refresh_token`

**ID:** `AU_DM_02`

| Column       | Type          | Null | Constraint                                            |
| ------------ | ------------- | ---: | ----------------------------------------------------- |
| `id`         | `uuid`        |   No | PK                                                    |
| `session_id` | `uuid`        |   No | FK → `authentication_session.id`; `ON DELETE CASCADE` |
| `token_hash` | `text`        |   No | SHA-256 verifier; unique                              |
| `created_at` | `timestamptz` |   No |                                                       |
| `expires_at` | `timestamptz` |   No |                                                       |
| `used_at`    | `timestamptz` |  Yes |                                                       |
| `revoked_at` | `timestamptz` |  Yes |                                                       |

Rules:

* One refresh-token record belongs to exactly one authentication session.
* The raw refresh token is never persisted.
* `token_hash` is the SHA-256 digest of the exact refresh-token string.
* A refresh token is invalid when `expires_at <= current_time`.
* A refresh token is invalid when `used_at IS NOT NULL`.
* A refresh token is invalid when `revoked_at IS NOT NULL`.
* A refresh token is invalid when its authentication session is revoked or expired.
* Each newly issued refresh token expires 2592000 seconds after issuance.
* Deleting an authentication session deletes its refresh-token records through `ON DELETE CASCADE`.

## 4.3 `authentication_access_token`

**ID:** `AU_DM_03`

| Field        | Type        | Rules                                                            |
| ------------ | ----------- | ---------------------------------------------------------------- |
| `id`         | uuid        | Primary key.                                                     |
| `session_id` | uuid        | Required FK to `authentication_session.id`; `ON DELETE CASCADE`. |
| `token_hash` | text        | Required SHA-256 verifier of the exact token string; unique.     |
| `created_at` | timestamptz | Required issuance timestamp.                                     |
| `expires_at` | timestamptz | Required expiration timestamp; must be later than `created_at`.  |

Rules:

* One access-token record belongs to exactly one authentication session.
* A session may have multiple access-token records.
* Only the SHA-256 verifier is persisted.
* The raw access token is never persisted.
* A token is invalid when `expires_at <= current_time`.
* A token is invalid when its session is revoked or expired.
* A token is invalid when its account is inactive or soft-deleted.
* Deleting an authentication session deletes its access-token records through `ON DELETE CASCADE`.
* Validation performs exact lookup using the server-computed `token_hash`.

## 4.4 Token Storage Rule

For every issued access or refresh token:

```text
raw token
    ↓
SHA-256(raw token)
    ↓
persist verifier
```

Raw access and refresh tokens are never stored in the database or logs.

Access-token and refresh-token verifiers are stored independently.

Client-provided identity or authorization data must never replace or bypass server-side token lookup.

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
2. Retrieve account authentication data.
3. Verify the password.
4. Require `status = active`.
5. Require `deleted_at IS NULL`.
6. Apply the authentication rate-limiting policy defined by `AU_SEC_DEC_CREDENTIAL_06`. If either the normalized-email or source-IP limit is exceeded, reject the request with `429 AUTHENTICATION_RATE_LIMITED` and do not continue authentication processing.
7. Create an authentication session.
8. Set `expires_at = session_creation_time + 24 hours`.
9. Set `last_authenticated_at = current_time`.
10. Issue an access token with a 3600-second lifetime.
11. Issue a refresh token with a 2592000-second lifetime.
12. Return the authentication response.

## 5.2 Refresh Authentication

**ID:** `AU_UC_02`

| Item   | Definition                           |
| ------ | ------------------------------------ |
| Actor  | Client with Refresh Token            |
| Input  | Refresh Token                        |
| Result | New Access Token + new Refresh Token |

### Rules

1. Compute the SHA-256 verifier of the supplied refresh token.
2. Look up the refresh-token record by `token_hash`.
3. Require the refresh-token record to exist.
4. Require `expires_at > current_time`.
5. Require `used_at IS NULL`.
6. Require `revoked_at IS NULL`.
7. Load the bound authentication session.
8. Require the session to exist.
9. Require `session.revoked_at IS NULL`.
10. Require `session.expires_at > current_time`.
11. Load Account authentication state.
12. Require `account.status = active`.
13. Require `account.deleted_at IS NULL`.
14. Mark the presented refresh token as used.
15. Issue a replacement refresh token with a 2592000-second lifetime.
16. Issue a new access token with a 3600-second lifetime.
17. Commit all state changes atomically.
18. Return the authentication response.

A replayed refresh token must not issue new credentials.

A successful refresh must not modify `authentication_session.expires_at` or `authentication_session.last_authenticated_at`. Refresh is not reauthentication and does not establish a new authentication session or reset the session's 24-hour absolute lifetime.

## 5.3 Revoke Authentication

**ID:** `AU_UC_03`

| Item   | Definition                     |
| ------ | ------------------------------ |
| Actor  | Authenticated Principal        |
| Input  | Current Authentication Session |
| Result | Session revoked                |

### Rules

1. Obtain `session_id` from the authenticated principal.
2. Load the authentication session.
3. Require the session to belong to the principal's `account_id`.
4. Mark the session revoked.
5. Set `revocation_reason` when available.
6. Commit the session revocation atomically.
7. All access tokens bound to the session become invalid.
8. All refresh tokens bound to the session become invalid.

## 5.4 Validate Bearer Authentication

**ID:** `AU_UC_04`

| Item   | Definition                        |
| ------ | --------------------------------- |
| Actor  | Protected API Request             |
| Input  | `Authorization: Bearer <token>`   |
| Result | `AuthenticatedPrincipal` or `401` |

### Rules

1. Require the `Authorization` header.
2. Require the `Bearer` authentication scheme.
3. Extract the bearer token.
4. Compute `SHA-256` over the exact token string.
5. Look up `authentication_access_token` by `token_hash`.
6. Require the access-token record to exist.
7. Require `authentication_access_token.expires_at > current_time`.
8. Load `authentication_session` using `authentication_access_token.session_id`.
9. Require the authentication session to exist.
10. Require `authentication_session.revoked_at IS NULL`.
11. Require `authentication_session.expires_at > current_time`.
12. Load Account authentication state.
13. Require `account.status = active`.
14. Require `account.deleted_at IS NULL`.
15. Create `AuthenticatedPrincipal { account_id, session_id, authenticated_at }`.
16. Pass the principal to Authorization.
17. Do not evaluate roles or permissions in Authentication.

If any authentication rule fails, return `401 Unauthorized`.

Authentication must not accept or trust client-supplied `account_id`, `session_id`, roles, permissions, or authorization decisions.

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

Headers:

```text
Content-Type: application/json
Cache-Control: no-store
```

### Errors

| Status | Code                    |
| ------ | ----------------------- |
| `400`  | `INVALID_REQUEST`       |
| `401`  | `INVALID_REFRESH_TOKEN` |

## 6.5 Revoke Authentication

### Request

`POST /auth/logout`

The session identified by the authenticated principal is revoked.

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

```rust
AuthenticatedPrincipal {
    account_id,
    session_id,
    authenticated_at,
}
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

Authentication may consume Account-owned authentication state required for authentication and protected-request validation:

* account identifier
* password hash for login verification
* account status
* soft-delete state

Authentication does not own:

* canonical Account profile data
* canonical email storage
* Account password hash storage
* Account lifecycle

### Account Hard-Delete Contract

Account hard deletion is authoritative for invalidating Authentication state belonging to the deleted Account.

Authentication persistence must therefore define:

```text
authentication_session.account_id
    REFERENCES account(id)
    ON DELETE CASCADE
```

Authentication token persistence must define:

```text
authentication_access_token.session_id
    REFERENCES authentication_session(id)
    ON DELETE CASCADE

authentication_refresh_token.session_id
    REFERENCES authentication_session(id)
    ON DELETE CASCADE
```

Consequently:

```text
DELETE account
    ↓
CASCADE authentication_session
    ↓
CASCADE authentication_access_token
CASCADE authentication_refresh_token
```

Authentication must not require a separate post-delete cleanup call from the Account service.

The Account hard-delete transaction is the transaction boundary for these cascades. If the transaction rolls back, Authentication state remains unchanged. If the transaction commits, no session or token record belonging to the deleted Account remains.

This contract is consistent with PostgreSQL's `CASCADE` semantics for dependent records that cannot exist independently. ([PostgreSQL][4])

Authentication does not define or execute the Account administrator-invariant check. Account owns that invariant.

## 7.2 Authentication → Authorization

Authentication provides:

```rust
AuthenticatedPrincipal {
    account_id,
    session_id,
    authenticated_at,
}
```

Authentication does not provide or determine:

* roles
* permissions
* authorization decisions

Authorization consumes the principal and performs the permission check.

## 7.3 Access-Token Validation Contract

Authentication owns:

* access-token generation
* access-token verifier creation
* access-token persistence
* access-token expiration
* token-to-session binding
* access-token validation
* authentication-session validation
* Account authentication-state validation
* principal creation

The authoritative validation sequence is:

```text
Bearer token
    → SHA-256 verifier
    → authentication_access_token.token_hash
    → session_id
    → authentication_session
    → account_id
    → Account authentication state
    → AuthenticatedPrincipal
```

All authentication identity and lifecycle values come from server-side state.

Client-supplied identity, session, role, permission, or authorization data cannot override this state.

### Hard-Delete Invalidation Guarantee

After successful Account hard-delete commit:

```text
authentication_session row       = absent
authentication_access_token rows  = absent
authentication_refresh_token rows = absent
```

Therefore no Authentication credential belonging to the deleted Account remains usable.

A transaction failure must not produce partial Authentication cleanup. SQLx transactions roll back when they are not successfully committed. ([Docs.rs][2])

## 7.4 Token Lifecycle

### Login

```text
Authenticate Account
    → create authentication session
    → establish session expiration
    → generate access token
    → persist access-token verifier
    → generate refresh token
    → persist refresh-token verifier
    → return raw tokens
```

### Protected Request

```text
Bearer token
    → validate access token
    → validate session
    → validate account state
    → create principal
```

### Refresh

```text
Refresh token
    → validate refresh token
    → validate session
    → validate account state
    → mark presented refresh token used
    → issue replacement refresh token
    → issue new access token
```

## 7.5 Request Pipeline

```text
Request
    → Authentication
    → AuthenticatedPrincipal
    → Authorization
    → Required Permission
    → Handler
    → Service
```

Authentication failure returns `401 Unauthorized`.

Authorization failure for an authenticated principal returns `403 Forbidden`.

Authorization must not execute before Authentication has established a valid principal.

## 7.6 Protected Resource Contract

For `POST /admin/accounts`:

```text
HTTP request
    → Authentication
    → AuthenticatedPrincipal
    → Authorization
    → account:create
    → Account::Create Account
```

Account remains responsible for account-creation business rules and persistence.

Authorization remains responsible for the permission decision.

Authentication remains responsible for establishing the authenticated principal.

---

# 8. Implementation Status

## 8.1 Requirements

| ID                 | Status         | Reason |
| ------------------ | -------------- | ------ |
| `AU_REQ_FC_01`     | 🟢 Implemented |        |
| `AU_REQ_FC_02`     | 🟢 Implemented |        |
| `AU_REQ_FC_03`     | 🟢 Implemented |        |
| `AU_REQ_FC_04`     | 🟢 Implemented |        |
| `AU_REQ_FC_05`     | 🟢 Implemented |        |
| `AU_REQ_FC_06`     | 🟢 Implemented |        |
| `AU_REQ_FC_07`     | 🟢 Implemented |        |
| `AU_REQ_FC_08`     | 🟢 Implemented |        |
| `AU_REQ_FC_09`     | 🟢 Implemented |        |
| `AU_REQ_FC_10`     | 🟢 Implemented |        |
| `AU_REQ_FC_11`     | 🟢 Implemented |        |
| `AU_REQ_FC_12`     | 🟢 Implemented |        |
| `AU_REQ_FC_13`     | 🟢 Implemented |        |
| `AU_REQ_FC_14`     | 🟢 Implemented |        |
| `AU_REQ_FC_15`     | 🟢 Implemented |        |
| `AU_REQ_FC_16`     | 🟢 Implemented |        |
| `AU_REQ_FC_17`     | 🟢 Implemented |        |
| `AU_REQ_FC_18`     | 🟢 Implemented |        |
| `AU_REQ_FC_19`     | 🟢 Implemented |        |
| `AU_REQ_FC_20`     | 🟢 Implemented |        |
| `AU_REQ_FC_21`     | 🟢 Implemented |        |
| `AU_REQ_FC_22`     | 🟢 Implemented |        |
| `AU_REQ_FC_23`     | 🟢 Implemented |        |
| `AU_REQ_FC_24`     | 🟢 Implemented |        |
| `AU_REQ_FC_25`     | 🟢 Implemented |        |
| `AU_REQ_NON_FC_01` | 🟢 Implemented |        |
| `AU_REQ_NON_FC_02` | 🟢 Implemented |        |
| `AU_REQ_NON_FC_03` | 🟢 Implemented |        |
| `AU_REQ_NON_FC_04` | 🟢 Implemented |        |
| `AU_REQ_NON_FC_05` | 🟢 Implemented |        |
| `AU_REQ_NON_FC_06` | 🟢 Implemented |        |
| `AU_REQ_NON_FC_07` | 🟢 Implemented |        |

## 8.2 Security Decisions

| Decision ID                | Status         | Reason |
| -------------------------- | -------------- | ------ |
| `AU_SEC_DEC_CREDENTIAL_01` | 🟢 Implemented |        |
| `AU_SEC_DEC_CREDENTIAL_02` | 🟢 Implemented |        |
| `AU_SEC_DEC_CREDENTIAL_03` | 🟢 Implemented |        |
| `AU_SEC_DEC_CREDENTIAL_04` | 🟢 Implemented |        |
| `AU_SEC_DEC_CREDENTIAL_05` | 🟢 Implemented |        |
| `AU_SEC_DEC_CREDENTIAL_06` | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_01`     | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_02`     | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_03`     | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_04`     | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_05`     | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_06`     | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_07`     | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_08`     | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_09`     | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_10`     | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_11`     | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_12`     | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_13`     | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_14`     | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_15`     | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_16`     | 🟢 Implemented |        |
| `AU_SEC_DEC_REFRESH_01`    | 🟢 Implemented |        |
| `AU_SEC_DEC_REFRESH_02`    | 🟢 Implemented |        |
| `AU_SEC_DEC_REFRESH_03`    | 🟢 Implemented |        |
| `AU_SEC_DEC_REFRESH_04`    | 🟢 Implemented |        |
| `AU_SEC_DEC_REFRESH_05`    | 🟢 Implemented |        |
| `AU_SEC_DEC_REFRESH_06`    | 🟢 Implemented |        |
| `AU_SEC_DEC_REFRESH_07`    | 🟢 Implemented |        |
| `AU_SEC_DEC_REFRESH_08`    | 🟢 Implemented |        |
| `AU_SEC_DEC_FAILURE_01`    | 🟢 Implemented |        |
| `AU_SEC_DEC_FAILURE_02`    | 🟢 Implemented |        |
| `AU_SEC_DEC_FAILURE_03`    | 🟢 Implemented |        |
| `AU_SEC_DEC_FAILURE_04`    | 🟢 Implemented |        |
| `AU_SEC_DEC_FAILURE_05`    | 🟢 Implemented |        |
| `AU_SEC_DEC_FAILURE_06`    | 🟢 Implemented |        |
| `AU_SEC_DEC_FAILURE_07`    | 🟢 Implemented |        |
| `AU_SEC_DEC_PRINCIPAL_01`  | 🟢 Implemented |        |
| `AU_SEC_DEC_PRINCIPAL_02`  | 🟢 Implemented |        |
| `AU_SEC_DEC_PRINCIPAL_03`  | 🟢 Implemented |        |
| `AU_SEC_DEC_PRINCIPAL_04`  | 🟢 Implemented |        |
| `AU_SEC_DEC_PRINCIPAL_05`  | 🟢 Implemented |        |
| `AU_SEC_DEC_SESSION_01`    | 🟢 Implemented |        |
| `AU_SEC_DEC_SESSION_02`    | 🟢 Implemented |        |
| `AU_SEC_DEC_SESSION_03`    | 🟢 Implemented |        |
| `AU_SEC_DEC_SESSION_04`    | 🟢 Implemented |        |
| `AU_SEC_DEC_SESSION_05`    | 🟢 Implemented |        |
| `AU_SEC_DEC_SESSION_06`    | 🟢 Implemented |        |

## 8.3 Design Decisions

| ID                              | Status         | Reason |
| ------------------------------- | -------------- | ------ |
| Authentication responsibilities | 🟢 Implemented |        |
| Authentication flow             | 🟢 Implemented |        |
| Protected request flow          | 🟢 Implemented |        |
| Refresh flow                    | 🟢 Implemented |        |

## 8.4 Data Model

| ID         | Data Model                     | Status         | Reason |
| ---------- | ------------------------------ | -------------- | ------ |
| `AU_DM_01` | `authentication_session`       | 🟢 Implemented |        |
| `AU_DM_02` | `authentication_refresh_token` | 🟢 Implemented |        |
| `AU_DM_03` | `authentication_access_token`  | 🟢 Implemented |        |

## 8.5 Use Cases

| ID         | Description                    | Status         | Reason |
| ---------- | ------------------------------ | -------------- | ------ |
| `AU_UC_01` | Authenticate Account           | 🟢 Implemented |        |
| `AU_UC_02` | Refresh Authentication         | 🟢 Implemented |        |
| `AU_UC_03` | Revoke Authentication          | 🟢 Implemented |        |
| `AU_UC_04` | Validate Bearer Authentication | 🟢 Implemented |        |

## 8.6 API Contract

| ID               | Description              | Status         | Reason |
| ---------------- | ------------------------ | -------------- | ------ |
| `AU_API_01`      | `POST /auth/login`       | 🟢 Implemented |        |
| `AU_API_02`      | `POST /auth/refresh`     | 🟢 Implemented |        |
| `AU_API_03`      | `POST /auth/logout`      | 🟢 Implemented |        |
| `AU_CONTRACT_01` | `AuthenticatedPrincipal` | 🟢 Implemented |        |

---

# 9. Domain Boundary

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

[2]: https://docs.rs/sqlx/latest/sqlx/struct.Transaction.html?utm_source=chatgpt.com "Transaction in sqlx - Rust"
[4]: https://www.postgresql.org/docs/18/ddl-constraints.html?utm_source=chatgpt.com "PostgreSQL: Documentation: 18: 5.5. Constraints"
