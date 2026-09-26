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

| ID             | Requirement                                                                                         |
| -------------- | --------------------------------------------------------------------------------------------------- |
| `AU_REQ_FC_19` | Authentication must obtain authentication-relevant account data from the Account domain.            |
| `AU_REQ_FC_20` | Authentication must not own canonical account profile data.                                         |
| `AU_REQ_FC_21` | Authentication must not own the Account password hash.                                              |
| `AU_REQ_FC_22` | Authentication must provide the authenticated account ID to downstream consumers.                   |
| `AU_REQ_FC_23` | Authentication must provide an `AuthenticatedPrincipal` for protected bearer-authenticated requests.|
| `AU_REQ_FC_24` | Authentication must not decide roles or permissions.                                                |
| `AU_REQ_FC_25` | Authorization must consume the authenticated principal for access decisions.                        |
| `AU_REQ_FC_26` | Browser clients must keep access tokens in memory only.                                             |
| `AU_REQ_FC_27` | Browser clients must receive the refresh credential through an `HttpOnly` secure cookie.            |
| `AU_REQ_FC_28` | Browser clients must restore authenticated state after document reload through `POST /auth/refresh`.|
| `AU_REQ_FC_29` | Refresh credentials must be rotated after every successful refresh.                                 |
| `AU_REQ_FC_30` | Browser authentication must not expose the refresh credential to JavaScript application code.       |
| `AU_REQ_FC_31` | Browser-authenticated state-changing authentication endpoints must enforce origin/CSRF protections. |
| `AU_REQ_FC_32` | Logout must revoke the authentication session and expire the browser refresh cookie.                |

## 1.4 Non-Functional Requirements

| ID                 | Requirement                                                                                                                                                                                                                           |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `AU_REQ_NON_FC_01` | Passwords, raw access tokens, raw refresh tokens, and `Authorization` header values must not be logged.                                                                                                                               |
| `AU_REQ_NON_FC_02` | Raw access tokens must not be persisted by the application server and must remain in browser memory only.                                                                                                                            |
| `AU_REQ_NON_FC_03` | Raw refresh tokens must not be persisted by the application server in databases, files, caches, or logs. The browser may retain the opaque refresh credential only in the designated `HttpOnly; Secure` authentication cookie. |
| `AU_REQ_NON_FC_04` | Authentication endpoints must use HTTPS/TLS. TLS 1.3 must be preferred; TLS 1.2 may be supported for compatibility; TLS 1.0 and TLS 1.1 must not be negotiated. The browser-facing application must also use HTTPS in environments where secure authentication cookies are used. |
| `AU_REQ_NON_FC_05` | Failed password authentication attempts must be rate-limited.                                                                                                                                                                        |
| `AU_REQ_NON_FC_06` | Authentication failures must not reveal unnecessary account-existence information.                                                                                                                                                    |
| `AU_REQ_NON_FC_07` | Authentication responses must use `Cache-Control: no-store`.                                                                                                                                                                         |
| `AU_REQ_NON_FC_08` | Browser refresh credentials must use `HttpOnly`, `Secure`, `Path=/`, no `Domain`, and the `__Host-` cookie prefix.                                                                                                                   |
| `AU_REQ_NON_FC_09` | Browser refresh credentials must use `SameSite=Strict` by default. `SameSite=None; Secure` is permitted only for explicitly configured cross-site frontend/API deployments and requires the CSRF/origin protections defined by this contract. |
| `AU_REQ_NON_FC_10` | Browser-authenticated state-changing requests must validate an explicitly configured frontend `Origin` allowlist.                                                                                                                    |
| `AU_REQ_NON_FC_11` | When present, `Sec-Fetch-Site` must not indicate `cross-site` by default. `cross-site` may be accepted only for an explicitly configured cross-site frontend/API deployment using `SameSite=None; Secure`, and only when the request `Origin` matches the configured frontend-origin allowlist. |
| `AU_REQ_NON_FC_12` | Cross-origin browser requests using authentication cookies must use credentialed CORS with an explicit origin allowlist and must never use `Access-Control-Allow-Origin: *`.                                                        |

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
| `AU_SEC_REQ_09` | Authentication must not expose passwords, password hashes, raw refresh credentials outside the designated `HttpOnly; Secure` cookie, or token verifier material. Access tokens may be returned only in the successful token responses defined by this API contract. |

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

| ID                      | Decision        | Definition                                                                                                              |
| ----------------------- | --------------- | ----------------------------------------------------------------------------------------------------------------------- |
| `AU_SEC_DEC_REFRESH_01` | Purpose         | Obtain new access credentials for an existing authentication session.                                                   |
| `AU_SEC_DEC_REFRESH_02` | Type            | Opaque token.                                                                                                           |
| `AU_SEC_DEC_REFRESH_03` | Storage         | Persist only a SHA-256 verifier of the exact refresh-token string on the server.                                        |
| `AU_SEC_DEC_REFRESH_04` | Rotation        | Successful refresh invalidates the presented refresh token and issues a replacement.                                    |
| `AU_SEC_DEC_REFRESH_05` | Replay          | A previously used refresh token must be rejected.                                                                       |
| `AU_SEC_DEC_REFRESH_06` | Session Binding | Each refresh token belongs to exactly one authentication session.                                                       |
| `AU_SEC_DEC_REFRESH_07` | Revocation      | Session revocation invalidates its refresh tokens immediately.                                                          |
| `AU_SEC_DEC_REFRESH_08` | Expiration      | The refresh token must not outlive its bound authentication session.                                                    |
| `AU_SEC_DEC_REFRESH_09` | Lifetime        | For `AU_UC_01`, the initial refresh token expires no later than `authentication_session.expires_at`.                    |
| `AU_SEC_DEC_REFRESH_10` | Rotation Lifetime | A replacement refresh token must not receive an expiration later than the original refresh-token expiration or the bound session expiration. |
| `AU_SEC_DEC_REFRESH_11` | Browser Storage | Browser clients receive the raw refresh credential only through the designated `HttpOnly; Secure` cookie.               |
| `AU_SEC_DEC_REFRESH_12` | JavaScript Access | Browser JavaScript must never read, copy, render, or otherwise access the raw refresh credential.                     |

For browser authentication sessions:

```text
authentication_session.expires_at
        ↓
refresh token expiration boundary
        ↓
browser refresh-cookie Max-Age boundary
```

Refresh does not extend the authentication session.

A successful refresh must therefore never create a new 24-hour session window.

The refresh token remains opaque to the frontend application.

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
Login request
  ↓
Normalize and validate email
  ↓
Apply authentication rate limits
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

Authentication rate limiting occurs before account lookup and password verification, as defined by `AU_SEC_DEC_CREDENTIAL_06`.

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
* The initial refresh token expires at `authentication_session.expires_at`.
* A replacement refresh token expires no later than both the presented refresh token's `expires_at` and `authentication_session.expires_at`.
* Refresh-token expiration must never extend the authentication-session lifetime.
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
2. Apply the authentication rate-limiting policy defined by `AU_SEC_DEC_CREDENTIAL_06`. If either the normalized-email or source-IP limit is exceeded, reject the request with `429 AUTHENTICATION_RATE_LIMITED` and stop processing before account lookup or password verification.
3. Retrieve account authentication data.
4. Verify the password.
5. Require `status = active`.
6. Require `deleted_at IS NULL`.
7. Create an authentication session.
8. Set `expires_at = session_creation_time + 24 hours`.
9. Set `last_authenticated_at = current_time`.
10. Issue an access token with a 3600-second lifetime.
11. Issue an initial refresh token with `expires_at = authentication_session.expires_at`.
12. Return the authentication response.

## 5.2 Refresh Authentication

**ID:** `AU_UC_02`

| Item   | Definition                                      |
| ------ | ----------------------------------------------- |
| Actor  | Browser Client with Refresh Cookie              |
| Input  | `__Host-refresh_token` cookie                   |
| Result | New Access Token + rotated browser refresh cookie |

### Rules

1. Read the refresh credential from the `__Host-refresh_token` cookie.
2. Do not accept a refresh token from a query parameter or request body.
3. Compute the SHA-256 verifier of the supplied refresh token.
4. Look up the refresh-token record by `token_hash`.
5. Require the refresh-token record to exist.
6. Require `expires_at > current_time`.
7. Require `used_at IS NULL`.
8. Require `revoked_at IS NULL`.
9. Load the bound authentication session.
10. Require the session to exist.
11. Require `session.revoked_at IS NULL`.
12. Require `session.expires_at > current_time`.
13. Load Account authentication state.
14. Require `account.status = active`.
15. Require `account.deleted_at IS NULL`.
16. Mark the presented refresh token as used.
17. Issue a replacement refresh token.
18. Set the replacement refresh token expiration no later than both the original refresh-token expiration and `authentication_session.expires_at`.
19. Issue a new access token with a 3600-second lifetime.
20. Set the replacement `__Host-refresh_token` cookie with the same security attributes defined by the Browser Credential Contract.
21. Commit the refresh-token rotation and access-token issuance atomically.
22. Return the new access token.
23. Never return the refresh token in the JSON response.

A replayed refresh token must not issue new credentials.

A successful refresh must not modify:

```text
authentication_session.expires_at
authentication_session.last_authenticated_at
```

Refresh is not reauthentication and does not establish a new authentication session.

## 5.3 Revoke Authentication

**ID:** `AU_UC_03`

| Item   | Definition                                      |
| ------ | ----------------------------------------------- |
| Actor  | Authenticated Browser Client                    |
| Input  | Current `__Host-refresh_token` cookie           |
| Result | Authentication session revoked + cookie cleared |

### Rules

1. Read the current refresh credential from the `__Host-refresh_token` cookie.
2. Compute the SHA-256 verifier of the supplied refresh token.
3. Resolve the refresh-token record to its authentication session.
4. If the refresh token and session are valid, revoke the bound authentication session.
5. All access tokens bound to the session become invalid.
6. All refresh tokens bound to the session become invalid.
7. Expire the `__Host-refresh_token` cookie.
8. Commit session revocation atomically.
9. The endpoint is idempotent: a missing, expired, already-used, or already-revoked refresh credential does not reveal authentication state and still returns successful logout semantics.
10. Logout must not require an unexpired access token.

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

| Rule                          | Definition                                                                                                                                                                      |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Base Path                     | `/auth`                                                                                                                                                                         |
| Success Content-Type          | `application/json` for token responses; empty body for logout                                                                                                                   |
| Error Content-Type            | `application/problem+json`                                                                                                                                                      |
| Protected API Authentication  | `Authorization: Bearer <access-token>`                                                                                                                                          |
| Browser Session Credential    | `__Host-refresh_token` HTTP cookie                                                                                                                                              |
| Access Token Query Parameter  | Not accepted                                                                                                                                                                    |
| Refresh Token Query Parameter | Not accepted                                                                                                                                                                    |
| Refresh Token Body Parameter  | Not accepted                                                                                                                                                                    |
| Caching                       | `Cache-Control: no-store`                                                                                                                                                       |
| TLS                           | Required                                                                                                                                                                        |
| Browser Cookie                | `HttpOnly; Secure; Path=/; no Domain; SameSite=Strict` by default                                                                                                               |
| Cross-Site Cookie             | `SameSite=None; Secure` only when explicitly configured for a cross-site frontend/API deployment                                                                                |
| CORS                          | Explicit allowed origins + `Access-Control-Allow-Credentials: true`; wildcard origins forbidden                                                                                 |
| CSRF                          | Exact `Origin` allowlist required for browser-authenticated state-changing requests                                                                                             |
| Fetch Metadata                | `Sec-Fetch-Site: cross-site` rejected by default; accepted only for an explicitly configured cross-site frontend/API deployment whose `Origin` matches the configured allowlist |
| Authorization                 | Protected feature permissions remain handled by Authorization domain                                                                                                            |

## 6.2 Authentication Endpoints

| ID          | Method | Endpoint        | Use Case   |
| ----------- | ------ | --------------- | ---------- |
| `AU_API_01` | `POST` | `/auth/login`   | `AU_UC_01` |
| `AU_API_02` | `POST` | `/auth/refresh` | `AU_UC_02` |
| `AU_API_03` | `POST` | `/auth/logout`  | `AU_UC_03` |

Bearer validation remains part of the Authentication request boundary for protected Admin API requests. In the current implementation, protected handlers obtain `AuthenticatedPrincipal` through the Authentication request extractor.

All three browser authentication endpoints are state-changing and must enforce the Browser Credential and CSRF Contract.

### 6.3 Authenticate Account

### Success

**`200 OK`**

```json
{
  "access_token": "opaque-access-token",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

Headers:

```text
Content-Type: application/json
Cache-Control: no-store
Set-Cookie: __Host-refresh_token=<opaque-refresh-token>; Max-Age=<remaining-session-seconds>; Path=/; Secure; HttpOnly; SameSite=<configured-value>
```

The refresh credential is intentionally absent from the JSON response.

The default configured cookie value is:

```text
SameSite=Strict
```

For an explicitly configured cross-site frontend/API deployment:

```text
SameSite=None; Secure
```

### 6.4 Refresh Authentication

### Success

**`200 OK`**

```json
{
  "access_token": "new-opaque-access-token",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

Headers:

```text
Content-Type: application/json
Cache-Control: no-store
Set-Cookie: __Host-refresh_token=<replacement-refresh-token>; Max-Age=<remaining-session-seconds>; Path=/; Secure; HttpOnly; SameSite=<configured-value>
```

The `SameSite` value must use the same configured browser-cookie policy as the login response.

### 6.5 Revoke Authentication

### Success

**`204 No Content`**

Headers:

```text
Cache-Control: no-store
Set-Cookie: __Host-refresh_token=; Max-Age=0; Path=/; Secure; HttpOnly; SameSite=<configured-value>
```

The cookie expiration response must use the same configured cookie policy as the issued refresh cookie.

Logout is idempotent.

The browser cookie must be expired even when the server-side refresh credential is already invalid.

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

## 7.4 Browser Client Authentication Persistence Contract

The browser Admin application uses a split credential model:

```text
Access Token
    ↓
JavaScript memory only

Refresh Credential
    ↓
__Host-refresh_token cookie
    ↓
HttpOnly + Secure + Path=/ + no Domain
```

### Browser Bootstrap

On every application startup or full document reload:

1. Authentication state begins as `unknown`.
2. The frontend must not redirect `/admin` to `/login` while authentication state is `unknown`.
3. The frontend sends `POST /auth/refresh` with browser credentials enabled.
4. If refresh succeeds, the returned access token is stored in memory and authentication state becomes `authenticated`.
5. If refresh fails with `401`, the access-token memory state is empty and authentication state becomes `unauthenticated`.
6. The frontend then applies the route rules for the resulting authenticated state.

### Access Token Lifetime

The access token:

* is stored only in JavaScript memory;
* is never stored in `localStorage`;
* is never stored in `sessionStorage`;
* is never written to cookies;
* is never written to URLs;
* is never logged.

### Refresh Rotation

Every successful `POST /auth/refresh`:

* invalidates the presented refresh token;
* issues a replacement refresh token;
* updates the `__Host-refresh_token` cookie;
* returns only a new access token in the JSON response;
* never changes the authentication session expiration.

### Protected API Recovery

When a protected Admin API request receives `401 Unauthorized` because the access token is invalid or expired:

1. The frontend may perform one serialized `POST /auth/refresh`.
2. On successful refresh, the frontend replaces the in-memory access token.
3. The original protected request may be retried once.
4. If refresh fails, the frontend clears client authentication state and treats the user as unauthenticated.
5. The frontend must not loop indefinitely between refresh and protected-request retry.

### Logout

Logout:

1. sends `POST /auth/logout` with browser credentials;
2. relies on the server-side refresh cookie to identify the authentication session;
3. revokes the server-side authentication session;
4. expires the refresh cookie;
5. clears the in-memory access token;
6. navigates the frontend to `/login`.

A logout network failure must not be treated as successful logout. The client must retain its current authenticated state and permit a retry until the server-side logout has succeeded.

### Cross-Origin Browser Requests

When the frontend and API are on different origins:

* frontend requests to `/auth/login`, `/auth/refresh`, and `/auth/logout` must use `credentials: "include"`;
* protected API requests using bearer access tokens do not rely on the refresh cookie;
* the API must return an explicit `Access-Control-Allow-Origin` matching the configured frontend origin;
* the API must return `Access-Control-Allow-Credentials: true`;
* wildcard `Access-Control-Allow-Origin: *` is forbidden for credentialed requests;
* the server must validate the `Origin` header against its explicit frontend-origin allowlist.

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

| ID                 | Status         | Reason                                                                                                                                            |
| ------------------ | -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `AU_REQ_FC_01`     | 🟢 Implemented |                                                                                                                                                   |
| `AU_REQ_FC_02`     | 🟢 Implemented |                                                                                                                                                   |
| `AU_REQ_FC_03`     | 🟢 Implemented |                                                                                                                                                   |
| `AU_REQ_FC_04`     | 🟢 Implemented |                                                                                                                                                   |
| `AU_REQ_FC_05`     | 🟢 Implemented |                                                                                                                                                   |
| `AU_REQ_FC_06`     | 🟢 Implemented |                                                                                                                                                   |
| `AU_REQ_FC_07`     | 🟢 Implemented |                                                                                                                                                   |
| `AU_REQ_FC_08`     | 🟢 Implemented |                                                                                                                                                   |
| `AU_REQ_FC_09`     | 🟢 Implemented |                                                                                                                                                   |
| `AU_REQ_FC_10`     | 🟢 Implemented |                                                                                                                                                   |
| `AU_REQ_FC_11`     | 🟢 Implemented |                                                                                                                                                   |
| `AU_REQ_FC_12`     | 🟢 Implemented |                                                                                                                                                   |
| `AU_REQ_FC_13`     | 🟢 Implemented | Refresh-token generation, persistence, and refresh endpoint logic exist.                                                                          |
| `AU_REQ_FC_14`     | 🟢 Implemented | Successful refresh invalidates the presented refresh token and creates a replacement.                                                             |
| `AU_REQ_FC_15`     | 🟢 Implemented |                                                                                                                                                   |
| `AU_REQ_FC_16`     | 🟢 Implemented | Refresh validation rejects expired refresh tokens.                                                                                                |
| `AU_REQ_FC_17`     | 🟢 Implemented |                                                                                                                                                   |
| `AU_REQ_FC_18`     | 🟢 Implemented | Refresh validation rejects previously used refresh tokens.                                                                                        |
| `AU_REQ_FC_19`     | 🟢 Implemented |                                                                                                                                                   |
| `AU_REQ_FC_20`     | 🟢 Implemented |                                                                                                                                                   |
| `AU_REQ_FC_21`     | 🟢 Implemented |                                                                                                                                                   |
| `AU_REQ_FC_22`     | 🟢 Implemented |                                                                                                                                                   |
| `AU_REQ_FC_23`     | 🟢 Implemented |                                                                                                                                                   |
| `AU_REQ_FC_24`     | 🟢 Implemented |                                                                                                                                                   |
| `AU_REQ_FC_25`     | 🟢 Implemented |                                                                                                                                                   |
| `AU_REQ_FC_26`     | 🟢 Implemented | Browser access token is held only in frontend application memory.                                                                                 |
| `AU_REQ_FC_27`     | 🟡 In Progress | Server-managed `HttpOnly; Secure` refresh cookie is not implemented yet.                                                                          |
| `AU_REQ_FC_28`     | 🟡 In Progress | Browser reload bootstrap through `POST /auth/refresh` is not implemented yet.                                                                     |
| `AU_REQ_FC_29`     | 🟢 Implemented | Server-side refresh-token rotation is implemented. Browser-cookie transport remains incomplete.                                                   |
| `AU_REQ_FC_30`     | 🟡 In Progress | Current login response still exposes `refresh_token` to frontend JavaScript.                                                                      |
| `AU_REQ_FC_31`     | 🟡 In Progress | Origin/CSRF protections for cookie-authenticated state-changing authentication requests are not implemented yet.                                  |
| `AU_REQ_FC_32`     | 🟡 In Progress | Session revocation exists, but logout does not yet revoke through the browser refresh credential or expire the browser cookie.                    |
| `AU_REQ_NON_FC_01` | 🟢 Implemented | Request logging excludes Authorization headers, cookies, request bodies, and response bodies.                                                     |
| `AU_REQ_NON_FC_02` | 🟢 Implemented | Server stores only access-token verifiers; browser access token remains in memory.                                                                |
| `AU_REQ_NON_FC_03` | 🟡 In Progress | Server stores only refresh-token verifiers, but the raw refresh token is currently returned to browser JavaScript.                                |
| `AU_REQ_NON_FC_04` | 🟡 In Progress | Backend TLS is configured, but the current development browser application is served over HTTP.                                                   |
| `AU_REQ_NON_FC_05` | 🟡 In Progress | Authentication rate limiting is implemented, but the current counters include reserved in-flight login attempts rather than failed attempts only. |
| `AU_REQ_NON_FC_06` | 🟢 Implemented | Authentication failure responses are generic.                                                                                                     |
| `AU_REQ_NON_FC_07` | 🟢 Implemented | Authentication responses use `Cache-Control: no-store`.                                                                                           |
| `AU_REQ_NON_FC_08` | 🟡 In Progress | `__Host-refresh_token` cookie attributes are not implemented yet.                                                                                 |
| `AU_REQ_NON_FC_09` | 🟡 In Progress | `SameSite=Strict` browser refresh-cookie policy is not implemented yet.                                                                           |
| `AU_REQ_NON_FC_10` | 🟡 In Progress | Explicit frontend-origin validation is not implemented yet.                                                                                       |
| `AU_REQ_NON_FC_11` | 🟡 In Progress | `Sec-Fetch-Site` validation is not implemented yet.                                                                                               |
| `AU_REQ_NON_FC_12` | 🟡 In Progress | Credentialed cross-origin CORS configuration is not implemented yet.                                                                              |

## 8.2 Security Requirements

| ID              | Status         | Reason                                                                                                                           |
| --------------- | -------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| `AU_SEC_REQ_01` | 🟢 Implemented | Protected Admin API requests use HTTP Bearer authentication.                                                                     |
| `AU_SEC_REQ_02` | 🟢 Implemented | Bearer credentials are accepted through the `Authorization` header.                                                              |
| `AU_SEC_REQ_03` | 🟢 Implemented | Bearer credentials are not accepted through query parameters or request bodies.                                                  |
| `AU_SEC_REQ_04` | 🟢 Implemented | Missing authentication returns `401 Unauthorized`.                                                                               |
| `AU_SEC_REQ_05` | 🟢 Implemented | Invalid, expired, or revoked bearer authentication returns `401 Unauthorized`.                                                   |
| `AU_SEC_REQ_06` | 🟢 Implemented | Missing bearer authentication returns the required Bearer challenge.                                                             |
| `AU_SEC_REQ_07` | 🟢 Implemented | Invalid bearer authentication returns the required `invalid_token` challenge.                                                    |
| `AU_SEC_REQ_08` | 🟢 Implemented | Authentication errors use `application/problem+json`.                                                                            |
| `AU_SEC_REQ_09` | 🟡 In Progress | The implementation still returns the raw refresh credential in the login response instead of only through the designated cookie. |

## 8.3 Security Decisions

### Credential Verification

| Decision ID                | Status         | Reason                                                                                                                                     |
| -------------------------- | -------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `AU_SEC_DEC_CREDENTIAL_01` | 🟢 Implemented |                                                                                                                                            |
| `AU_SEC_DEC_CREDENTIAL_02` | 🟢 Implemented |                                                                                                                                            |
| `AU_SEC_DEC_CREDENTIAL_03` | 🟢 Implemented |                                                                                                                                            |
| `AU_SEC_DEC_CREDENTIAL_04` | 🟢 Implemented |                                                                                                                                            |
| `AU_SEC_DEC_CREDENTIAL_05` | 🟢 Implemented |                                                                                                                                            |
| `AU_SEC_DEC_CREDENTIAL_06` | 🟡 In Progress | Sliding-window rate limiting exists, but reserved in-flight attempts are currently counted instead of only failed authentication attempts. |

### Access Tokens

| Decision ID            | Status         | Reason |
| ---------------------- | -------------- | ------ |
| `AU_SEC_DEC_ACCESS_01` | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_02` | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_03` | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_04` | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_05` | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_06` | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_07` | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_08` | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_09` | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_10` | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_11` | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_12` | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_13` | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_14` | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_15` | 🟢 Implemented |        |
| `AU_SEC_DEC_ACCESS_16` | 🟢 Implemented |        |

### Refresh Tokens

| Decision ID             | Status         | Reason                                                                                                                 |
| ----------------------- | -------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `AU_SEC_DEC_REFRESH_01` | 🟢 Implemented | Refresh-token issuance and refresh flow exist.                                                                         |
| `AU_SEC_DEC_REFRESH_02` | 🟢 Implemented | Refresh tokens are opaque values.                                                                                      |
| `AU_SEC_DEC_REFRESH_03` | 🟢 Implemented | SHA-256 refresh-token verifiers are persisted.                                                                         |
| `AU_SEC_DEC_REFRESH_04` | 🟢 Implemented | Successful refresh rotates the presented refresh token.                                                                |
| `AU_SEC_DEC_REFRESH_05` | 🟢 Implemented | Previously used refresh tokens are rejected.                                                                           |
| `AU_SEC_DEC_REFRESH_06` | 🟢 Implemented | Refresh tokens are bound to authentication sessions.                                                                   |
| `AU_SEC_DEC_REFRESH_07` | 🟢 Implemented | Refresh validation rejects revoked sessions.                                                                           |
| `AU_SEC_DEC_REFRESH_08` | 🟡 In Progress | Current implementation uses a 30-day refresh-token lifetime instead of the session-bounded lifetime.                   |
| `AU_SEC_DEC_REFRESH_09` | 🟡 In Progress | Initial refresh-token expiration is not yet bounded by `authentication_session.expires_at`.                            |
| `AU_SEC_DEC_REFRESH_10` | 🟡 In Progress | Replacement refresh-token expiration is not yet bounded by both the presented token expiration and session expiration. |
| `AU_SEC_DEC_REFRESH_11` | 🟡 In Progress | Browser refresh credential is not yet delivered exclusively through the designated `HttpOnly; Secure` cookie.          |
| `AU_SEC_DEC_REFRESH_12` | 🟡 In Progress | Current login response exposes the refresh credential to frontend JavaScript.                                          |

### Authentication Failures

| Decision ID             | Status         | Reason |
| ----------------------- | -------------- | ------ |
| `AU_SEC_DEC_FAILURE_01` | 🟢 Implemented |        |
| `AU_SEC_DEC_FAILURE_02` | 🟢 Implemented |        |
| `AU_SEC_DEC_FAILURE_03` | 🟢 Implemented |        |
| `AU_SEC_DEC_FAILURE_04` | 🟢 Implemented |        |
| `AU_SEC_DEC_FAILURE_05` | 🟢 Implemented |        |
| `AU_SEC_DEC_FAILURE_06` | 🟢 Implemented |        |
| `AU_SEC_DEC_FAILURE_07` | 🟢 Implemented |        |

### Principal

| Decision ID               | Status         | Reason |
| ------------------------- | -------------- | ------ |
| `AU_SEC_DEC_PRINCIPAL_01` | 🟢 Implemented |        |
| `AU_SEC_DEC_PRINCIPAL_02` | 🟢 Implemented |        |
| `AU_SEC_DEC_PRINCIPAL_03` | 🟢 Implemented |        |
| `AU_SEC_DEC_PRINCIPAL_04` | 🟢 Implemented |        |
| `AU_SEC_DEC_PRINCIPAL_05` | 🟢 Implemented |        |

### Authentication Sessions

| Decision ID             | Status         | Reason                                                                  |
| ----------------------- | -------------- | ----------------------------------------------------------------------- |
| `AU_SEC_DEC_SESSION_01` | 🟢 Implemented |                                                                         |
| `AU_SEC_DEC_SESSION_02` | 🟢 Implemented |                                                                         |
| `AU_SEC_DEC_SESSION_03` | 🟢 Implemented |                                                                         |
| `AU_SEC_DEC_SESSION_04` | 🟢 Implemented |                                                                         |
| `AU_SEC_DEC_SESSION_05` | 🟢 Implemented | Access and refresh-token validation both require a valid bound session. |
| `AU_SEC_DEC_SESSION_06` | 🟢 Implemented |                                                                         |

### Browser Credential and CSRF Decisions

| Decision ID             | Status         | Reason                                                                                                      |
| ----------------------- | -------------- | ----------------------------------------------------------------------------------------------------------- |
| `AU_SEC_DEC_BROWSER_01` | 🟡 In Progress | Access tokens are memory-only, but refresh-cookie integration is not implemented.                           |
| `AU_SEC_DEC_BROWSER_02` | 🟡 In Progress | `__Host-refresh_token` has not yet been implemented.                                                        |
| `AU_SEC_DEC_BROWSER_03` | 🟡 In Progress | `SameSite=Strict` is defined by contract but not implemented.                                               |
| `AU_SEC_DEC_BROWSER_04` | 🟡 In Progress | Cross-site cookie configuration is not yet implemented as an explicit deployment option.                    |
| `AU_SEC_DEC_BROWSER_05` | 🟡 In Progress | Frontend `Origin` allowlist validation is not implemented.                                                  |
| `AU_SEC_DEC_BROWSER_06` | 🟡 In Progress | `Sec-Fetch-Site` validation is not implemented.                                                             |
| `AU_SEC_DEC_BROWSER_07` | 🟡 In Progress | Credentialed CORS with explicit origins is not implemented.                                                 |
| `AU_SEC_DEC_BROWSER_08` | 🟡 In Progress | Authentication bootstrap through `POST /auth/refresh` is not implemented in the Admin frontend.             |
| `AU_SEC_DEC_BROWSER_09` | 🟡 In Progress | Logout still authenticates through the access-token bearer path rather than the browser refresh credential. |

## 8.4 Design Decisions

| ID                              | Status         | Reason                                                                                                              |
| ------------------------------- | -------------- | ------------------------------------------------------------------------------------------------------------------- |
| Authentication responsibilities | 🟢 Implemented |                                                                                                                     |
| Authentication flow             | 🟡 In Progress | The flow exists, but login rate limiting still counts reserved in-flight attempts rather than failed attempts only. |
| Protected request flow          | 🟢 Implemented |                                                                                                                     |
| Refresh flow                    | 🟡 In Progress | Rotation and replay protection exist, but session-bounded lifetime and browser-cookie integration are incomplete.   |
| Browser authentication flow     | 🟡 In Progress | Browser bootstrap, cookie persistence, access-token recovery, and logout-cookie handling are not implemented.       |
| Browser CSRF/CORS flow          | 🟡 In Progress | Origin validation, Fetch Metadata validation, and credentialed cross-origin CORS are not implemented.               |

## 8.5 Data Model

| ID         | Data Model                     | Status         | Reason                                                                                                                                                  |
| ---------- | ------------------------------ | -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `AU_DM_01` | `authentication_session`       | 🟢 Implemented |                                                                                                                                                         |
| `AU_DM_02` | `authentication_refresh_token` | 🟡 In Progress | The table and session binding exist, but the implemented refresh-token expiration still uses a 30-day lifetime instead of the session-bounded lifetime. |
| `AU_DM_03` | `authentication_access_token`  | 🟢 Implemented |                                                                                                                                                         |

## 8.6 Use Cases

| ID         | Description                    | Status         | Reason                                                                                                                                             |
| ---------- | ------------------------------ | -------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| `AU_UC_01` | Authenticate Account           | 🟡 In Progress | Email/password authentication works, but the browser response contract must move the refresh credential from JSON to an `HttpOnly; Secure` cookie. |
| `AU_UC_02` | Refresh Authentication         | 🟡 In Progress | Server-side refresh works, but the request must consume the browser cookie and replacement lifetime must be session-bounded.                       |
| `AU_UC_03` | Revoke Authentication          | 🟡 In Progress | Session revocation exists, but logout must use the browser refresh credential and expire the cookie.                                               |
| `AU_UC_04` | Validate Bearer Authentication | 🟢 Implemented |                                                                                                                                                    |

## 8.7 API Contract

| ID               | Description              | Status         | Reason                                                                                                                              |
| ---------------- | ------------------------ | -------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| `AU_API_01`      | `POST /auth/login`       | 🟡 In Progress | Endpoint exists, but response must establish the browser refresh cookie and stop returning the raw refresh token in JSON.           |
| `AU_API_02`      | `POST /auth/refresh`     | 🟡 In Progress | Endpoint exists, but it must consume the refresh cookie, return only the access token, and use session-bounded refresh expiration.  |
| `AU_API_03`      | `POST /auth/logout`      | 🟡 In Progress | Endpoint exists, but it must revoke by refresh-cookie session and expire the cookie without requiring bearer access authentication. |
| `AU_CONTRACT_01` | `AuthenticatedPrincipal` | 🟢 Implemented |                                                                                                                                     |

## 8.8 Browser Integration Contract

| ID              | Description                                                   | Status         | Reason                                                                             |
| --------------- | ------------------------------------------------------------- | -------------- | ---------------------------------------------------------------------------------- |
| `AU_BROWSER_01` | Access token stored in frontend memory only                   | 🟢 Implemented | Frontend authentication state currently uses module memory.                        |
| `AU_BROWSER_02` | Refresh credential stored in `__Host-refresh_token` cookie    | 🟡 In Progress | Backend cookie issuance is not implemented.                                        |
| `AU_BROWSER_03` | Refresh credential inaccessible to JavaScript                 | 🟡 In Progress | Current login response still exposes `refresh_token` to JavaScript.                |
| `AU_BROWSER_04` | Authentication bootstrap after document reload                | 🟡 In Progress | Frontend does not currently call `/auth/refresh` during application bootstrap.     |
| `AU_BROWSER_05` | `unknown` authentication state during bootstrap               | 🟡 In Progress | Current route guard treats missing in-memory state as unauthenticated immediately. |
| `AU_BROWSER_06` | Serialized access-token refresh after protected-request `401` | 🟡 In Progress | No browser refresh/retry coordinator is currently implemented.                     |
| `AU_BROWSER_07` | Logout through browser refresh credential                     | 🟡 In Progress | Current logout depends on bearer access authentication.                            |
| `AU_BROWSER_08` | Refresh cookie expired after logout                           | 🟡 In Progress | Cookie expiration is not implemented.                                              |
| `AU_BROWSER_09` | No refresh-token persistence in Web Storage or IndexedDB      | 🟢 Implemented | No refresh-token persistence mechanism currently exists in browser storage.        |
| `AU_BROWSER_10` | Credentialed browser authentication requests                  | 🟡 In Progress | `credentials: "include"` is not currently defined for the authentication flow.     |

## 8.9 Verification Status

The following criteria are not yet verified in the browser/backend integration:

| ID             | Verification Criteria                                                                 | Status         |
| -------------- | ------------------------------------------------------------------------------------- | -------------- |
| `AU_VERIFY_01` | Login establishes an authenticated session and browser refresh cookie                 | 🟡 In Progress |
| `AU_VERIFY_02` | Reloading `/admin` restores authentication without showing `/login`                   | 🟡 In Progress |
| `AU_VERIFY_03` | Refresh rotates the browser refresh credential                                        | 🟡 In Progress |
| `AU_VERIFY_04` | Expired access token can be refreshed once and protected request retried once         | 🟡 In Progress |
| `AU_VERIFY_05` | Refresh-token replay is rejected                                                      | 🟡 In Progress |
| `AU_VERIFY_06` | Logout revokes the session and expires the browser refresh cookie                     | 🟡 In Progress |
| `AU_VERIFY_07` | Logout does not require an unexpired access token                                     | 🟡 In Progress |
| `AU_VERIFY_08` | Cookie is `HttpOnly; Secure; Path=/; SameSite=Strict` in default same-site deployment | 🟡 In Progress |
| `AU_VERIFY_09` | Cross-origin credentialed authentication uses explicit origins and CORS credentials   | 🟡 In Progress |
| `AU_VERIFY_10` | Browser authentication endpoints reject disallowed cross-site state-changing requests | 🟡 In Progress |

## 8.10 Implementation Status Legend

| Status         | Meaning                                                                                   |
| -------------- | ----------------------------------------------------------------------------------------- |
| ⚪ Not Started  | Criteria has not been implemented.                                                       |
| 🟡 In Progress | Implementation is partial, or contract completion remains incomplete.                     |
| 🟢 Implemented | Implementation is complete. Verification is tracked separately in Section 8.9.            |
| 🔴 Blocked     | Implementation cannot proceed because a required design or dependency remains unresolved. |

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
