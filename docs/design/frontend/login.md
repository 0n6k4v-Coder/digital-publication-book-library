# Frontend Login

## Table of Contents

1. [Scope](#1-scope)
2. [Route Contract](#2-route-contract)
3. [Requirements](#3-requirements)
4. [UI Contract](#4-ui-contract)
5. [Authentication Flow](#5-authentication-flow)
6. [API Contract](#6-api-contract)
7. [Authentication State](#7-authentication-state)
8. [Security Contract](#8-security-contract)
9. [Accessibility](#9-accessibility)
10. [Implementation Criteria](#10-implementation-criteria)

---

# 1. Scope

This document is the frontend source of truth for the Admin Login page and browser authentication behavior.

| ID                  | Item                 | Definition                                                                                                   |
| ------------------- | -------------------- | ------------------------------------------------------------------------------------------------------------ |
| `FE_LOGIN_SCOPE_01` | Frontend Scope       | Admin Login page and frontend authentication behavior                                                        |
| `FE_LOGIN_SCOPE_02` | Auth Source of Truth | Backend Authentication domain                                                                                |
| `FE_LOGIN_SCOPE_03` | Backend Use Case     | `AU_UC_01 — Authenticate Account`                                                                            |
| `FE_LOGIN_SCOPE_04` | Login API            | `POST /auth/login`                                                                                           |
| `FE_LOGIN_SCOPE_05` | Access Token         | In-memory only                                                                                               |
| `FE_LOGIN_SCOPE_06` | Refresh Credential   | Server-managed `HttpOnly` cookie                                                                             |
| `FE_LOGIN_SCOPE_07` | Frontend Ownership   | Frontend owns only in-memory access-token state                                                              |
| `FE_LOGIN_SCOPE_08` | Backend Ownership    | Authentication rules, credentials, tokens, sessions, rotation, replay detection, and authentication security |

The frontend must never read, copy, manually persist, or manually send the refresh credential.

---

# 2. Route Contract

| ID                  | Route    | State                  | Behavior                                                                        |
| ------------------- | -------- | ---------------------- | ------------------------------------------------------------------------------- |
| `FE_LOGIN_ROUTE_01` | `/login` | `unknown`              | Show bootstrap pending state; do not redirect                                   |
| `FE_LOGIN_ROUTE_02` | `/login` | `unauthenticated`      | Show Login page                                                                 |
| `FE_LOGIN_ROUTE_03` | `/login` | `authenticated`        | Redirect to `/admin`                                                            |
| `FE_LOGIN_ROUTE_04` | `/login` | `authentication-error` | Show Login page with retry/error UI                                             |
| `FE_LOGIN_ROUTE_05` | `/admin` | `unknown`              | Show bootstrap pending state; do not redirect                                   |
| `FE_LOGIN_ROUTE_06` | `/admin` | `unauthenticated`      | Redirect to `/login`                                                            |
| `FE_LOGIN_ROUTE_07` | `/admin` | `authenticated`        | Show Admin Shell                                                                |
| `FE_LOGIN_ROUTE_08` | `/admin` | `authentication-error` | Show authentication retry/error UI; do not redirect solely because of the error |

Route protection must wait for authentication bootstrap to resolve.

---

# 3. Requirements

## Authentication

| ID                | Requirement                                                              |
| ----------------- | ------------------------------------------------------------------------ |
| `FE_LOGIN_REQ_01` | Display Admin Login at `/login`.                                         |
| `FE_LOGIN_REQ_02` | Accept email and password.                                               |
| `FE_LOGIN_REQ_03` | Submit credentials to `POST /auth/login`.                                |
| `FE_LOGIN_REQ_04` | Restore authentication after document reload using `POST /auth/refresh`. |
| `FE_LOGIN_REQ_05` | Keep the access token in memory only.                                    |
| `FE_LOGIN_REQ_06` | Keep the refresh credential inaccessible to JavaScript.                  |
| `FE_LOGIN_REQ_07` | Establish authenticated state after successful login or bootstrap.       |
| `FE_LOGIN_REQ_08` | Navigate authenticated users to `/admin`.                                |

## Error Handling

| ID                | Requirement                                                           |
| ----------------- | --------------------------------------------------------------------- |
| `FE_LOGIN_REQ_09` | Show generic invalid-credentials errors.                              |
| `FE_LOGIN_REQ_10` | Show generic `429 AUTHENTICATION_RATE_LIMITED` errors.                |
| `FE_LOGIN_REQ_11` | Treat refresh `401` as unauthenticated.                               |
| `FE_LOGIN_REQ_12` | Do not treat non-`401` bootstrap/refresh failures as unauthenticated. |
| `FE_LOGIN_REQ_13` | Provide a retryable `authentication-error` state.                     |

## Refresh and Retry

| ID                | Requirement                                                                                                                   |
| ----------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `FE_LOGIN_REQ_14` | Allow only one shared in-flight protected-request refresh per browser application context.                                    |
| `FE_LOGIN_REQ_15` | Retry each affected protected request at most once.                                                                           |
| `FE_LOGIN_REQ_16` | Never recursively refresh the refresh request itself.                                                                         |
| `FE_LOGIN_REQ_17` | Never loop indefinitely between `401`, refresh, and retry.                                                                    |
| `FE_LOGIN_REQ_18` | Do not automatically retry a failed bootstrap request. Bootstrap retry is user-triggered through the retry UI.               |
| `FE_LOGIN_REQ_19` | Do not automatically retry a failed refresh used for protected-request recovery.                                              |
| `FE_LOGIN_REQ_20` | Cross-context refresh coordination is not provided. Each browser application context independently follows refresh-token rotation and replay rules. |

## Secret Handling

| ID                | Requirement                                                 |
| ----------------- | ----------------------------------------------------------- |
| `FE_LOGIN_REQ_21` | Never expose authentication tokens in UI.                   |
| `FE_LOGIN_REQ_22` | Never log passwords, access tokens, or refresh credentials. |
| `FE_LOGIN_REQ_23` | Never send authentication tokens in query parameters.       |

---

# 4. UI Contract

## Form

| ID               | Element         | Requirement                                                                                         |
| ---------------- | --------------- | --------------------------------------------------------------------------------------------------- |
| `FE_LOGIN_UI_01` | Email           | `type="email"`, required, `autocomplete="username"`                                                 |
| `FE_LOGIN_UI_02` | Password        | `type="password"`, required, `autocomplete="current-password"`                                      |
| `FE_LOGIN_UI_03` | Submit          | Sign In action                                                                                      |
| `FE_LOGIN_UI_04` | Validation      | Browser-native required/email validation only; no additional client-side business validation        |
| `FE_LOGIN_UI_05` | Server Rules    | Authentication rules remain server-side                                                             |
| `FE_LOGIN_UI_06` | Password Policy | Frontend must not reject a request using additional password-policy rules                           |

## UI Error States

| ID               | State                | Behavior                                       |
| ---------------- | -------------------- | ---------------------------------------------- |
| `FE_LOGIN_UI_07` | Invalid Request      | Show request/validation error                  |
| `FE_LOGIN_UI_08` | Invalid Credentials  | Show generic authentication error              |
| `FE_LOGIN_UI_09` | Rate Limited         | Show generic rate-limit error                  |
| `FE_LOGIN_UI_10` | Authentication Error | Show retryable bootstrap/refresh error         |
| `FE_LOGIN_UI_11` | Loading              | Prevent duplicate submission and show progress |

---

# 5. Authentication Flow

## 5.1 Bootstrap

Browser authentication bootstrap resolves the authentication state for the current application load.

```mermaid
flowchart TD
    A["Application starts"] --> B["authStatus = unknown"]
    B --> C{"Bootstrap in flight?"}
    C -->|Yes| D["Await shared operation"]
    C -->|No| E["POST /auth/refresh"]
    D --> F["Resolve result"]
    E --> F

    F -->|200 + valid access token| G["Store access token in memory"]
    G --> H["authStatus = authenticated"]

    F -->|401| I["Clear access token"]
    I --> J["authStatus = unauthenticated"]

    F -->|Other failure| K["Clear access token"]
    K --> L["authStatus = authentication-error"]
    L --> M["Show retry UI"]
    M -->|Explicit retry| B
```

### Bootstrap Request Contract

| ID                          | Requirement                                                                      |
| --------------------------- | -------------------------------------------------------------------------------- |
| `FE_LOGIN_BOOTSTRAP_REQ_01` | Method is `POST`.                                                                |
| `FE_LOGIN_BOOTSTRAP_REQ_02` | Endpoint is `/auth/refresh`.                                                     |
| `FE_LOGIN_BOOTSTRAP_REQ_03` | Request body is empty.                                                           |
| `FE_LOGIN_BOOTSTRAP_REQ_04` | Request uses `credentials: "include"`.                                           |
| `FE_LOGIN_BOOTSTRAP_REQ_05` | Request uses `cache: "no-store"`.                                                |
| `FE_LOGIN_BOOTSTRAP_REQ_06` | Authentication is provided by the browser-managed refresh credential.            |
| `FE_LOGIN_BOOTSTRAP_REQ_07` | Frontend code never reads, constructs, stores, or parses the refresh credential. |

### Bootstrap Behavior

| ID                      | Requirement                                                                                              |
| ----------------------- | -------------------------------------------------------------------------------------------------------- |
| `FE_LOGIN_BOOTSTRAP_01` | Initial authentication state is `unknown`.                                                               |
| `FE_LOGIN_BOOTSTRAP_02` | Bootstrap is application-scoped and independent of the current route.                                    |
| `FE_LOGIN_BOOTSTRAP_03` | Only one bootstrap request may be active at a time.                                                      |
| `FE_LOGIN_BOOTSTRAP_04` | Concurrent bootstrap callers reuse the same in-flight operation.                                         |
| `FE_LOGIN_BOOTSTRAP_05` | Component mount/unmount must not create duplicate bootstrap requests.                                    |
| `FE_LOGIN_BOOTSTRAP_06` | `200 OK` with a valid access-token response transitions to `authenticated`.                              |
| `FE_LOGIN_BOOTSTRAP_07` | The access token is stored only in frontend runtime memory.                                              |
| `FE_LOGIN_BOOTSTRAP_08` | `200 OK` with an invalid or incomplete response transitions to `authentication-error`.                   |
| `FE_LOGIN_BOOTSTRAP_09` | `401 Unauthorized` clears the in-memory access token and transitions to `unauthenticated`.               |
| `FE_LOGIN_BOOTSTRAP_10` | Any other bootstrap failure clears the in-memory access token and transitions to `authentication-error`. |
| `FE_LOGIN_BOOTSTRAP_11` | Bootstrap failures are not automatically retried.                                                        |
| `FE_LOGIN_BOOTSTRAP_12` | Bootstrap retry occurs only after an explicit user action.                                               |
| `FE_LOGIN_BOOTSTRAP_13` | `unknown` never redirects to `/login`.                                                                   |
| `FE_LOGIN_BOOTSTRAP_14` | `unknown` never redirects to `/admin`.                                                                   |
| `FE_LOGIN_BOOTSTRAP_15` | `unauthenticated` allows `/login` and redirects protected routes to `/login`.                            |
| `FE_LOGIN_BOOTSTRAP_16` | `authentication-error` renders retryable authentication UI and does not automatically redirect.          |

### React Integration Contract

| ID                            | Requirement                                                                                                                   |
| ----------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `FE_LOGIN_BOOTSTRAP_REACT_01` | Authentication state is exposed through one shared external store.                                                            |
| `FE_LOGIN_BOOTSTRAP_REACT_02` | The store provides `subscribe()` and `getSnapshot()`.                                                                         |
| `FE_LOGIN_BOOTSTRAP_REACT_03` | The store provides a bootstrap operation shared across consumers.                                                             |
| `FE_LOGIN_BOOTSTRAP_REACT_04` | Route components consume the shared authentication state rather than maintaining independent authentication state.            |
| `FE_LOGIN_BOOTSTRAP_REACT_05` | Re-renders, route changes, and component remounts must not start a new bootstrap attempt while the current attempt is active. |
| `FE_LOGIN_BOOTSTRAP_REACT_06` | After a bootstrap attempt settles, a new bootstrap attempt may begin only through the explicit retry flow.                    |

### Required Bootstrap Call

```ts
fetch("/auth/refresh", {
    method: "POST",
    credentials: "include",
    cache: "no-store",
});
```

### Verification Criteria

| ID                             | Verification                                                                                         |
| ------------------------------ | ---------------------------------------------------------------------------------------------------- |
| `FE_LOGIN_BOOTSTRAP_VERIFY_01` | Fresh document load starts in `unknown`.                                                             |
| `FE_LOGIN_BOOTSTRAP_VERIFY_02` | Valid refresh session reaches `authenticated` without requiring `/login`.                            |
| `FE_LOGIN_BOOTSTRAP_VERIFY_03` | Invalid/expired refresh session reaches `unauthenticated` and protected routes redirect to `/login`. |
| `FE_LOGIN_BOOTSTRAP_VERIFY_04` | Non-`401` bootstrap failure reaches `authentication-error` with no automatic retry.                  |
| `FE_LOGIN_BOOTSTRAP_VERIFY_05` | Explicit retry starts a new bootstrap attempt.                                                       |
| `FE_LOGIN_BOOTSTRAP_VERIFY_06` | Concurrent bootstrap callers produce one network request.                                            |

## 5.2 Login

```mermaid
flowchart LR
    A["Login page"] --> B["Submit email + password"]
    B --> C["POST /auth/login<br/>credentials: include"]
    C -->|200| D["Browser stores refresh cookie"]
    D --> E["Access token in memory"]
    E --> F["authStatus = authenticated"]
    F --> G["Navigate /admin"]
    C -->|400/401/429| H["Show appropriate error"]
```

| ID                 | Step                                                                 |
| ------------------ | ------------------------------------------------------------------- |
| `FE_LOGIN_FLOW_10` | Submit email/password only to `POST /auth/login`.                   |
| `FE_LOGIN_FLOW_11` | Use `credentials: "include"` for the login request.                |
| `FE_LOGIN_FLOW_12` | Backend issues the refresh cookie through `Set-Cookie`.             |
| `FE_LOGIN_FLOW_13` | Backend returns access-token data only; no refresh credential value. |
| `FE_LOGIN_FLOW_14` | Store the access token in memory only.                               |
| `FE_LOGIN_FLOW_15` | Transition to `authenticated`.                                      |
| `FE_LOGIN_FLOW_16` | Navigate to `/admin`.                                               |

## 5.3 Protected Request Recovery

```mermaid
flowchart TD
    A["Protected request"] --> B{401?}
    B -->|No| C["Return response"]
    B -->|Yes| D{"Retry already used?"}
    D -->|Yes| E["Return 401"]
    D -->|No| F{"Refresh already running in this application context?"}
    F -->|Yes| G["Wait for shared refresh"]
    F -->|No| H["Start one refresh"]

    H --> I{Refresh result}
    G --> I

    I -->|200| J["Replace in-memory access token"]
    J --> K["Retry original request once"]

    I -->|401| L["Clear access token"]
    L --> M["authStatus = unauthenticated"]
    M --> N["Fail affected request"]

    I -->|Other error| O["Clear access token"]
    O --> P["authStatus = authentication-error"]
    P --> Q["Fail affected request"]
```

| ID                 | Rule                                                                                                  |
| ------------------ | ----------------------------------------------------------------------------------------------------- |
| `FE_LOGIN_FLOW_17` | At most one protected-request refresh is active per browser application context.                      |
| `FE_LOGIN_FLOW_18` | Concurrent protected requests in the same application context wait for the same refresh operation.    |
| `FE_LOGIN_FLOW_19` | Retry each original protected request at most once.                                                   |
| `FE_LOGIN_FLOW_20` | The `/auth/refresh` request is excluded from refresh-on-`401` handling.                               |
| `FE_LOGIN_FLOW_21` | If the original request has already been retried, return its `401` without another refresh.           |
| `FE_LOGIN_FLOW_22` | A failed refresh is not automatically retried.                                                        |
| `FE_LOGIN_FLOW_23` | The frontend never retains or manually replays an old refresh credential.                             |
| `FE_LOGIN_FLOW_24` | Refresh coordination is scoped to one browser application context; cross-context coordination is not assumed by this contract. |
| `FE_LOGIN_FLOW_25` | A refresh performed concurrently by another browser application context can consume the same one-time refresh credential before this context uses it. |
| `FE_LOGIN_FLOW_26` | When this context receives `401 INVALID_REFRESH_TOKEN`, it clears its access token and becomes `unauthenticated`; it must not retry the old refresh credential. |

## 5.4 Logout

| ID                 | Rule                                                                                         |
| ------------------ | -------------------------------------------------------------------------------------------- |
| `FE_LOGIN_FLOW_27` | Logout uses the browser-managed refresh credential.                                          |
| `FE_LOGIN_FLOW_28` | Logout does not require an unexpired access token.                                           |
| `FE_LOGIN_FLOW_29` | Successful logout invalidates the server authentication session when a valid session exists. |
| `FE_LOGIN_FLOW_30` | Logout returns `204 No Content` even when the refresh credential is missing or already invalid, and expires the browser refresh cookie. |
| `FE_LOGIN_FLOW_31` | Frontend clears in-memory access-token state and transitions to `unauthenticated` after logout is accepted as successful. |

---

# 6. API Contract

## 6.1 Cookie Policy

| ID | Requirement |
| --- | --- |
| `FE_LOGIN_COOKIE_01` | Cookie name is exactly `__Host-refresh_token`. |
| `FE_LOGIN_COOKIE_02` | Cookie is `HttpOnly`. |
| `FE_LOGIN_COOKIE_03` | Cookie is `Secure`. |
| `FE_LOGIN_COOKIE_04` | Cookie uses `Path=/`. |
| `FE_LOGIN_COOKIE_05` | Cookie has no `Domain` attribute. |
| `FE_LOGIN_COOKIE_06` | Default cookie policy is `SameSite=Strict`. |
| `FE_LOGIN_COOKIE_07` | Cross-site cookie authentication requires explicit `SameSite=None; Secure` configuration plus the defined Origin/CSRF protections. |
| `FE_LOGIN_COOKIE_08` | Cookie lifetime must not exceed the remaining authentication-session lifetime. |
| `FE_LOGIN_COOKIE_09` | Frontend code never reads or stores the raw refresh credential. |
| `FE_LOGIN_COOKIE_10` | Frontend code never constructs a `Cookie` request header. |
| `FE_LOGIN_COOKIE_11` | Frontend code never reads or parses `Set-Cookie`. |
| `FE_LOGIN_COOKIE_12` | Successful refresh replaces the browser-managed refresh cookie. |
| `FE_LOGIN_COOKIE_13` | Logout expires the browser-managed refresh cookie using the same cookie name and scope. |

### Cookie Policy by Deployment

| ID | Deployment | Cookie | Frontend Request |
| --- | --- | --- | --- |
| `FE_LOGIN_COOKIE_DEPLOY_01` | Same-origin | `Secure; HttpOnly; Path=/; no Domain; SameSite=Strict` | `credentials: "include"` |
| `FE_LOGIN_COOKIE_DEPLOY_02` | Cross-origin, same-site | `Secure; HttpOnly; Path=/; no Domain; SameSite=Strict` | `credentials: "include"` + explicit credentialed CORS |
| `FE_LOGIN_COOKIE_DEPLOY_03` | Cross-site | `Secure; HttpOnly; Path=/; no Domain; SameSite=None` | `credentials: "include"` + explicit credentialed CORS + Origin/CSRF protections |

### Login Cookie

```text
Set-Cookie: __Host-refresh_token=<opaque>; Max-Age=<remaining-session-seconds>; Path=/; Secure; HttpOnly; SameSite=<configured-policy>
```

### Refresh Cookie

```text
Set-Cookie: __Host-refresh_token=<replacement-opaque>; Max-Age=<remaining-session-seconds>; Path=/; Secure; HttpOnly; SameSite=<configured-policy>
```

### Logout Cookie

```text
Set-Cookie: __Host-refresh_token=; Max-Age=0; Path=/; Secure; HttpOnly; SameSite=<configured-policy>
```

## 6.2 Login

| Property     | Contract           |
| ------------ | ------------------ |
| Method       | `POST`             |
| Endpoint     | `/auth/login`      |
| Request      | JSON               |
| Credentials  | `include`          |
| Content-Type | `application/json` |

### Request

```json
{
  "email": "admin@example.com",
  "password": "example-secure-password"
}
```

### Success

```http
HTTP/1.1 200 OK
Content-Type: application/json
Cache-Control: no-store
Set-Cookie: __Host-refresh_token=<opaque>; Max-Age=<seconds-until-session-expiry>; Path=/; Secure; HttpOnly; SameSite=<deployment-policy>
```

```json
{
  "access_token": "opaque-access-token",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

| ID                | Requirement                                                                      |
| ----------------- | -------------------------------------------------------------------------------- |
| `FE_LOGIN_API_01` | Response must not contain `refresh_token`.                                       |
| `FE_LOGIN_API_02` | Frontend must not read `Set-Cookie`.                                             |
| `FE_LOGIN_API_03` | Frontend must not construct a `Cookie` header.                                   |
| `FE_LOGIN_API_04` | Refresh credential expiration must not exceed authentication-session expiration. |

## 6.3 Refresh

| Property    | Contract        |
| ----------- | --------------- |
| Method      | `POST`          |
| Endpoint    | `/auth/refresh` |
| Body        | None            |
| Credentials | `include`       |

```http
POST /auth/refresh
```

The browser attaches the refresh cookie when the deployment's cookie policy permits it.

### Success

```http
HTTP/1.1 200 OK
Content-Type: application/json
Cache-Control: no-store
Set-Cookie: __Host-refresh_token=<new-opaque>; Max-Age=<seconds-until-session-expiry>; Path=/; Secure; HttpOnly; SameSite=<deployment-policy>
```

```json
{
  "access_token": "new-opaque-access-token",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

| ID                | Requirement                                                                                    |
| ----------------- | ---------------------------------------------------------------------------------------------- |
| `FE_LOGIN_API_05` | Server rotates the refresh credential.                                                         |
| `FE_LOGIN_API_06` | Replacement refresh credential does not extend the original authentication-session expiration. |
| `FE_LOGIN_API_07` | Response contains access-token data only.                                                      |
| `FE_LOGIN_API_08` | `401 INVALID_REFRESH_TOKEN` transitions frontend to `unauthenticated`.                         |
| `FE_LOGIN_API_09` | Other refresh failures transition frontend to `authentication-error`.                          |
| `FE_LOGIN_API_10` | A failed refresh is not automatically retried.                                                 |

## 6.4 Logout

| Property    | Contract       |
| ----------- | -------------- |
| Method      | `POST`         |
| Endpoint    | `/auth/logout` |
| Body        | None           |
| Credentials | `include`      |

### Success

```http
HTTP/1.1 204 No Content
Cache-Control: no-store
Set-Cookie: __Host-refresh_token=; Max-Age=0; Path=/; Secure; HttpOnly; SameSite=<deployment-policy>
```

| ID                | Requirement                                                                                                      |
| ----------------- | ---------------------------------------------------------------------------------------------------------------- |
| `FE_LOGIN_API_11` | Logout does not require a bearer access token.                                                                   |
| `FE_LOGIN_API_12` | When the refresh credential is valid, logout invalidates the authentication session.                             |
| `FE_LOGIN_API_13` | Logout is idempotent and returns `204 No Content` even when the refresh credential is missing or already invalid.|
| `FE_LOGIN_API_14` | Logout always expires the browser-managed `__Host-refresh_token` cookie.                                         |
| `FE_LOGIN_API_15` | Frontend logout clears in-memory access-token state after the logout operation is accepted as successful.        |

## 6.5 Error Contract

| ID                | Status | Code                          | Frontend Behavior                 |
| ----------------- | ------ | ----------------------------- | --------------------------------- |
| `FE_LOGIN_ERR_01` | `400`  | `INVALID_REQUEST`             | Show request/validation error     |
| `FE_LOGIN_ERR_02` | `401`  | `INVALID_CREDENTIALS`         | Show generic authentication error |
| `FE_LOGIN_ERR_03` | `401`  | `INVALID_REFRESH_TOKEN`       | Become unauthenticated            |
| `FE_LOGIN_ERR_04` | `429`  | `AUTHENTICATION_RATE_LIMITED` | Show generic rate-limit error     |

---

# 7. Authentication State

## 7.1 Authentication State Model

```mermaid
stateDiagram-v2
    [*] --> unknown

    unknown --> authenticated: Bootstrap 200 + valid response
    unknown --> unauthenticated: Bootstrap 401
    unknown --> authenticationError: Other bootstrap failure

    authenticationError --> unknown: Explicit retry
    authenticationError --> authenticated: Successful login

    unauthenticated --> authenticated: Successful login

    authenticated --> authenticated: Access token refreshed
    authenticated --> unauthenticated: Refresh 401
    authenticated --> authenticationError: Refresh non-401 failure
    authenticated --> unauthenticated: Successful logout

    state authenticationError as "authentication-error"
```

| ID                  | State                  | Meaning                                                                                                 |
| ------------------- | ---------------------- | ------------------------------------------------------------------------------------------------------- |
| `FE_LOGIN_STATE_01` | `unknown`              | Authentication has not yet been resolved for the current application load.                              |
| `FE_LOGIN_STATE_02` | `authenticated`        | A usable access token is held in frontend memory.                                                       |
| `FE_LOGIN_STATE_03` | `unauthenticated`      | No usable authenticated session is available.                                                           |
| `FE_LOGIN_STATE_04` | `authentication-error` | Authentication could not be resolved because bootstrap or refresh failed for a reason other than `401`. |

## 7.2 State Rules

| ID                       | Requirement                                                                                                                                 |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `FE_LOGIN_STATE_RULE_01` | Initial state is `unknown`.                                                                                                                 |
| `FE_LOGIN_STATE_RULE_02` | `unknown` must not redirect to `/login`.                                                                                                    |
| `FE_LOGIN_STATE_RULE_03` | `unknown` must not redirect to `/admin`.                                                                                                    |
| `FE_LOGIN_STATE_RULE_04` | `authenticated` permits protected-route rendering.                                                                                          |
| `FE_LOGIN_STATE_RULE_05` | `unauthenticated` redirects protected routes to `/login`.                                                                                   |
| `FE_LOGIN_STATE_RULE_06` | `authentication-error` displays retryable authentication UI.                                                                                |
| `FE_LOGIN_STATE_RULE_07` | `authentication-error` must not automatically redirect to `/login`.                                                                         |
| `FE_LOGIN_STATE_RULE_08` | Failed bootstrap/refresh operations clear stale in-memory access-token state before publishing `unauthenticated` or `authentication-error`. |
| `FE_LOGIN_STATE_RULE_09` | Refresh credentials are never represented in authentication state.                                                                          |
| `FE_LOGIN_STATE_RULE_10` | Route components must not maintain independent authentication state.                                                                        |

## 7.3 Store Snapshot

The authentication store exposes only browser-safe state.

```ts
type AuthStatus =
    | "unknown"
    | "authenticated"
    | "unauthenticated"
    | "authentication-error";

type AuthOperation =
    | "bootstrapIdle"
    | "bootstrapPending"
    | "bootstrapRetryPending"
    | "refreshIdle"
    | "refreshing";

interface AuthenticationSnapshot {
    authStatus: AuthStatus;
    operation: AuthOperation;
}
```

| ID                        | Requirement                                                                                                |
| ------------------------- | ---------------------------------------------------------------------------------------------------------- |
| `FE_LOGIN_STATE_STORE_01` | The snapshot contains `authStatus`.                                                                        |
| `FE_LOGIN_STATE_STORE_02` | The snapshot contains the current authentication operation state.                                          |
| `FE_LOGIN_STATE_STORE_03` | The snapshot contains no refresh token or cookie value.                                                    |
| `FE_LOGIN_STATE_STORE_04` | The access token is not exposed through persistent browser storage.                                        |
| `FE_LOGIN_STATE_STORE_05` | The store provides stable `subscribe()` and `getSnapshot()` behavior for React external-store consumption. |

## 7.4 Route Decision Matrix

| Route                 | `unknown`                    | `authenticated`        | `unauthenticated` | `authentication-error`                   |
| --------------------- | ---------------------------- | ---------------------- | ----------------- | ---------------------------------------- |
| `/login`              | Show bootstrap pending state | Redirect `/admin`      | Render Login page | Render Login page with retry/error UI    |
| `/admin`              | Show bootstrap pending state | Render Admin Shell     | Redirect `/login` | Render retryable authentication-error UI |
| Other protected route | Show bootstrap pending state | Render protected route | Redirect `/login` | Render retryable authentication-error UI |

## 7.5 Coordination Rules

| ID                        | Requirement                                                                        |
| ------------------------- | ---------------------------------------------------------------------------------- |
| `FE_LOGIN_STATE_COORD_01` | Only one bootstrap operation may be active at a time.                              |
| `FE_LOGIN_STATE_COORD_02` | Concurrent bootstrap callers share the same in-flight Promise/result.              |
| `FE_LOGIN_STATE_COORD_03` | React remounts must not create independent bootstrap operations.                   |
| `FE_LOGIN_STATE_COORD_04` | Explicit retry may begin only after the previous bootstrap attempt has settled.    |
| `FE_LOGIN_STATE_COORD_05` | Protected-request refresh is separate from initial bootstrap.                      |
| `FE_LOGIN_STATE_COORD_06` | Initial bootstrap must never be triggered by a protected-request `401` retry path. |
| `FE_LOGIN_STATE_COORD_07` | Only one protected-request refresh may be active per browser application context.  |
| `FE_LOGIN_STATE_COORD_08` | Concurrent protected requests waiting for refresh receive the same refresh result. |

---

# 8. Security Contract

## 8.1 Token Storage

| ID                | Rule                                                                                                                      |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `FE_LOGIN_SEC_01` | Access token: memory only.                                                                                                |
| `FE_LOGIN_SEC_02` | Refresh credential: browser-managed `HttpOnly` cookie only.                                                               |
| `FE_LOGIN_SEC_03` | Never use `localStorage`, `sessionStorage`, IndexedDB, or other JavaScript-accessible storage for the refresh credential. |
| `FE_LOGIN_SEC_04` | Never send authentication tokens in query parameters.                                                                     |
| `FE_LOGIN_SEC_05` | Never log authentication secrets.                                                                                         |
| `FE_LOGIN_SEC_06` | Never display authentication secrets.                                                                                     |

## 8.2 Cookie Policy

| ID | Rule |
| --- | --- |
| `FE_LOGIN_SEC_COOKIE_01` | The refresh credential is stored only in the browser-managed `__Host-refresh_token` cookie. |
| `FE_LOGIN_SEC_COOKIE_02` | The cookie is `HttpOnly`, `Secure`, `Path=/`, and has no `Domain`. |
| `FE_LOGIN_SEC_COOKIE_03` | `SameSite=Strict` is the default policy. |
| `FE_LOGIN_SEC_COOKIE_04` | Cross-site cookie authentication requires explicit `SameSite=None; Secure` configuration plus Origin/CSRF protections. |
| `FE_LOGIN_SEC_COOKIE_05` | The `__Host-` prefix requires `Secure`, `Path=/`, and no `Domain`. |
| `FE_LOGIN_SEC_COOKIE_06` | Authentication requests that rely on the browser-managed refresh cookie use `credentials: "include"`. |
| `FE_LOGIN_SEC_COOKIE_07` | `credentials: "include"` does not override browser cookie restrictions such as `SameSite`. |
| `FE_LOGIN_SEC_COOKIE_08` | Cookie lifetime must not exceed the authentication-session lifetime. |
| `FE_LOGIN_SEC_COOKIE_09` | Refresh rotation replaces the browser-managed cookie; frontend JavaScript never receives the raw refresh credential value. |
| `FE_LOGIN_SEC_COOKIE_10` | Logout expires the refresh cookie using the same cookie name and scope. |
| `FE_LOGIN_SEC_COOKIE_11` | The frontend never stores the refresh credential in `localStorage`, `sessionStorage`, IndexedDB, or other JavaScript-accessible storage. |
| `FE_LOGIN_SEC_COOKIE_12` | The frontend never manually constructs a `Cookie` header or reads `Set-Cookie`. |

## 8.3 CSRF, Origin, and CORS

| ID                | Rule                                                                                        |
| ----------------- | ------------------------------------------------------------------------------------------- |
| `FE_LOGIN_SEC_07` | `/auth/login`, `/auth/refresh`, and `/auth/logout` must follow the backend `Origin` policy. |
| `FE_LOGIN_SEC_08` | Cookie-authenticated state-changing endpoints must use the backend CSRF protections.        |
| `FE_LOGIN_SEC_09` | Credentialed cross-origin requests use an explicit allowed-origin list.                     |
| `FE_LOGIN_SEC_10` | Credentialed CORS must not use `Access-Control-Allow-Origin: *`.                            |
| `FE_LOGIN_SEC_11` | Credentialed cross-origin requests require `Access-Control-Allow-Credentials: true`.        |

## 8.4 Refresh Security

| ID                | Rule                                                                                                     |
| ----------------- | -------------------------------------------------------------------------------------------------------- |
| `FE_LOGIN_SEC_12` | Refresh credentials are rotated according to the backend Authentication domain contract.                 |
| `FE_LOGIN_SEC_13` | Rotation must not extend the original authentication-session expiration.                                 |
| `FE_LOGIN_SEC_14` | The frontend never replays an old refresh credential manually.                                           |
| `FE_LOGIN_SEC_15` | Refresh credentials are one-time credentials; concurrent refresh attempts are subject to backend replay detection. |
| `FE_LOGIN_SEC_16` | Protected-request refresh is serialized per browser application context.                                 |
| `FE_LOGIN_SEC_17` | Refresh-on-`401` is bounded to one refresh attempt and one retry per original protected request.         |
| `FE_LOGIN_SEC_18` | Bootstrap automatically executes once per application load; subsequent bootstrap attempts are user-triggered by the retry UI. |
| `FE_LOGIN_SEC_19` | A failed protected-request refresh is not automatically retried.                                         |

---

# 9. Accessibility

| ID                 | Requirement                                                  |
| ------------------ | ------------------------------------------------------------ |
| `FE_LOGIN_A11Y_01` | Each form field has a visible label.                         |
| `FE_LOGIN_A11Y_02` | Validation errors are associated with their relevant fields. |
| `FE_LOGIN_A11Y_03` | Form submission works from the keyboard.                     |
| `FE_LOGIN_A11Y_04` | Focus states are visible.                                    |
| `FE_LOGIN_A11Y_05` | Semantic form controls are used.                             |
| `FE_LOGIN_A11Y_06` | Loading and error states are announced appropriately.        |
| `FE_LOGIN_A11Y_07` | Sign In remains accessible while the form is usable.         |

---

# 10. Implementation Criteria

## Status

| Status         | Meaning                                                          |
| -------------- | ---------------------------------------------------------------- |
| ⚪ Not Started  | Not implemented or not verified                                  |
| 🟡 In Progress | Partially implemented or verification incomplete                 |
| 🟢 Implemented | Implemented and verified                                         |
| 🔴 Blocked     | Cannot proceed because a required dependency/contract is missing |

## 10.1 Routes

| ID                       | Criteria                                                             | Status         | Current Reason                                       |
| ------------------------ | -------------------------------------------------------------------- | -------------- | ---------------------------------------------------- |
| `FE_LOGIN_IMPL_ROUTE_01` | Bootstrap `unknown` state exists                                     | 🔴 Blocked     | Not implemented in `App.tsx`                         |
| `FE_LOGIN_IMPL_ROUTE_02` | `/login` waits for bootstrap                                         | 🔴 Blocked     | Current guard treats missing auth as unauthenticated |
| `FE_LOGIN_IMPL_ROUTE_03` | `/login` redirects authenticated users to `/admin`                   | 🟡 In Progress | Existing volatile-memory guard                       |
| `FE_LOGIN_IMPL_ROUTE_04` | `/admin` waits for bootstrap                                         | 🔴 Blocked     | Current guard redirects before bootstrap             |
| `FE_LOGIN_IMPL_ROUTE_05` | `/admin` redirects unauthenticated users to `/login` after bootstrap | 🟡 In Progress | Existing protection requires bootstrap integration   |
| `FE_LOGIN_IMPL_ROUTE_06` | `/admin` renders Admin Shell when authenticated                      | 🟢 Implemented | Existing route/layout                                |

## 10.2 Authentication Service

| ID                      | Criteria                                                      | Status         | Current Reason                                                |
| ----------------------- | ------------------------------------------------------------- | -------------- | ------------------------------------------------------------- |
| `FE_LOGIN_IMPL_AUTH_01` | Access token is memory-only                                   | 🟢 Implemented | Current auth service uses module memory                       |
| `FE_LOGIN_IMPL_AUTH_02` | Refresh token is removed from frontend state                  | 🔴 Blocked     | Current `AuthenticationSession` still contains `refreshToken` |
| `FE_LOGIN_IMPL_AUTH_03` | Login uses browser credentials                                | 🔴 Blocked     | `credentials: "include"` not implemented                      |
| `FE_LOGIN_IMPL_AUTH_04` | Bootstrap calls `/auth/refresh`                               | 🔴 Blocked     | Not implemented                                               |
| `FE_LOGIN_IMPL_AUTH_05` | Refresh rotates browser credential                            | 🔴 Blocked     | Cookie refresh not implemented                                |
| `FE_LOGIN_IMPL_AUTH_06` | Refresh `401` becomes unauthenticated                         | 🔴 Blocked     | Refresh flow not implemented                                  |
| `FE_LOGIN_IMPL_AUTH_07` | Non-`401` refresh failures become authentication-error        | 🔴 Blocked     | Error state not implemented                                   |
| `FE_LOGIN_IMPL_AUTH_08` | Protected `401` refresh is serialized                         | 🔴 Blocked     | Refresh coordinator not implemented                           |
| `FE_LOGIN_IMPL_AUTH_09` | Protected request retries at most once                        | 🔴 Blocked     | Retry mechanism not implemented                               |
| `FE_LOGIN_IMPL_AUTH_10` | Refresh request is excluded from recursive refresh handling   | 🔴 Blocked     | Refresh interceptor not implemented                           |
| `FE_LOGIN_IMPL_AUTH_11` | Bootstrap does not automatically retry after failure          | 🔴 Blocked     | Retry policy not implemented                                  |
| `FE_LOGIN_IMPL_AUTH_12` | Failed protected-request refresh is not automatically retried | 🔴 Blocked     | Bounded refresh policy not implemented                        |
| `FE_LOGIN_IMPL_AUTH_13` | Refresh credentials are never logged or exposed                | 🔴 Blocked     | Current login response exposes `refresh_token` to frontend JavaScript and stores it in frontend state |
| `FE_LOGIN_IMPL_AUTH_14` | Logout uses browser-managed refresh credential                | 🔴 Blocked     | Current logout uses bearer access token                       |

## 10.3 Backend Contract Dependencies

| ID                         | Criteria                                                                                         | Status     | Current Reason                                                                                         |
| -------------------------- | ------------------------------------------------------------------------------------------------ | ---------- | ------------------------------------------------------------------------------------------------------ |
| `FE_LOGIN_IMPL_BACKEND_01` | Login sets the server-managed refresh cookie with deployment-appropriate attributes              | 🔴 Blocked | Backend cookie contract not implemented                                                                |
| `FE_LOGIN_IMPL_BACKEND_02` | Refresh accepts the browser-managed refresh cookie and returns access-token data only            | 🔴 Blocked | Current refresh contract uses a request-body refresh token                                              |
| `FE_LOGIN_IMPL_BACKEND_03` | Logout accepts browser-managed refresh authentication and does not require bearer access-token authentication | 🔴 Blocked | Current logout requires bearer access-token authentication |
| `FE_LOGIN_IMPL_BACKEND_04` | Refresh expiry cannot exceed session expiry                                                     | 🔴 Blocked | Backend currently issues refresh credentials for 30 days against a 24-hour authentication session      |
| `FE_LOGIN_IMPL_BACKEND_05` | Origin/CSRF policy is implemented                                                              | 🔴 Blocked | Backend policy requires update                                                                       |
| `FE_LOGIN_IMPL_BACKEND_06` | Concurrent use of the same refresh credential follows one-time rotation/replay semantics        | 🔴 Blocked | Browser-cookie integration is not implemented; backend one-time rotation exists but browser behavior is not yet wired |

## 10.4 Testing

| ID                 | Test                                                                                         |
| ------------------ | -------------------------------------------------------------------------------------------- |
| `FE_LOGIN_TEST_01` | Successful login transitions authentication state to `authenticated`.                        |
| `FE_LOGIN_TEST_02` | Failed login remains on `/login`.                                                            |
| `FE_LOGIN_TEST_03` | Application startup begins with `authStatus = unknown`.                                      |
| `FE_LOGIN_TEST_04` | Startup invokes `POST /auth/refresh` once for one active application context.                |
| `FE_LOGIN_TEST_05` | Bootstrap uses `credentials: "include"`.                                                     |
| `FE_LOGIN_TEST_06` | Bootstrap sends no refresh-token request body.                                               |
| `FE_LOGIN_TEST_07` | Bootstrap uses `cache: "no-store"`.                                                          |
| `FE_LOGIN_TEST_08` | Valid bootstrap response transitions to `authenticated`.                                     |
| `FE_LOGIN_TEST_09` | Invalid bootstrap response payload transitions to `authentication-error`.                    |
| `FE_LOGIN_TEST_10` | Bootstrap `401` transitions to `unauthenticated`.                                            |
| `FE_LOGIN_TEST_11` | Other bootstrap failures transition to `authentication-error`.                               |
| `FE_LOGIN_TEST_12` | `unknown` does not redirect `/admin` to `/login`.                                            |
| `FE_LOGIN_TEST_13` | `unknown` does not redirect `/login` to `/admin`.                                            |
| `FE_LOGIN_TEST_14` | `authenticated` on `/login` redirects to `/admin`.                                           |
| `FE_LOGIN_TEST_15` | `unauthenticated` on `/admin` redirects to `/login`.                                         |
| `FE_LOGIN_TEST_16` | `authentication-error` does not automatically redirect.                                      |
| `FE_LOGIN_TEST_17` | Bootstrap failure does not automatically retry.                                              |
| `FE_LOGIN_TEST_18` | Explicit retry starts a new bootstrap attempt.                                                |
| `FE_LOGIN_TEST_19` | Concurrent bootstrap callers share one in-flight operation.                                  |
| `FE_LOGIN_TEST_20` | React development Strict Mode does not produce duplicate concurrent bootstrap requests.      |
| `FE_LOGIN_TEST_21` | Component unmount/remount does not invalidate bootstrap.                                     |
| `FE_LOGIN_TEST_22` | Full document reload restores an active authenticated session through `/auth/refresh`.       |
| `FE_LOGIN_TEST_23` | Full document reload reaches `/login` when `/auth/refresh` returns `401`.                     |
| `FE_LOGIN_TEST_24` | Refresh credentials never appear in frontend state, rendered UI, logs, or application APIs.  |
| `FE_LOGIN_TEST_25` | Access token exists only in frontend runtime memory.                                         |
| `FE_LOGIN_TEST_26` | Route components use the shared authentication store.                                        |
| `FE_LOGIN_TEST_27` | A protected-request `401` triggers one shared refresh operation.                             |
| `FE_LOGIN_TEST_28` | Concurrent protected-request `401`s share the same refresh operation.                        |
| `FE_LOGIN_TEST_29` | Each original protected request retries at most once.                                         |
| `FE_LOGIN_TEST_30` | Failed protected-request refresh is not automatically retried.                               |
| `FE_LOGIN_TEST_31` | The refresh endpoint cannot recursively trigger another refresh operation.                   |
| `FE_LOGIN_TEST_32` | Successful logout clears server-side authentication and in-memory access state.              |
| `FE_LOGIN_TEST_33` | Refresh credentials never appear in the authentication store snapshot.                       |
| `FE_LOGIN_TEST_34` | When two browser application contexts concurrently use the same one-time refresh credential, at most one refresh succeeds; a later use of the consumed credential returns `401 INVALID_REFRESH_TOKEN` and that context becomes `unauthenticated`. |

### Test Coverage Matrix

| Area                       | Required IDs            |
| -------------------------- | ----------------------- |
| Login                      | `FE_LOGIN_TEST_01`–`02` |
| Bootstrap                  | `FE_LOGIN_TEST_03`–`11` |
| Routing                    | `FE_LOGIN_TEST_12`–`16` |
| Retry/coordination         | `FE_LOGIN_TEST_17`–`21` |
| Reload/session restoration | `FE_LOGIN_TEST_22`–`25` |
| Store integration          | `FE_LOGIN_TEST_26`      |
| Protected-request recovery | `FE_LOGIN_TEST_27`–`31` |
| Logout                     | `FE_LOGIN_TEST_32`      |
| Credential isolation       | `FE_LOGIN_TEST_33`      |
| Cross-context behavior     | `FE_LOGIN_TEST_34`      |

## 10.5 Current Verification Status

| Area                            | Status         |
| ------------------------------- | -------------- |
| Login form UI                   | 🟢 Implemented |
| Basic validation                | 🟢 Implemented |
| In-memory access token          | 🟢 Implemented |
| Reload-safe authentication      | 🔴 Blocked     |
| HttpOnly refresh cookie         | 🔴 Blocked     |
| Bootstrap refresh               | 🔴 Blocked     |
| Protected-request refresh/retry | 🔴 Blocked     |
| Cookie-based logout             | 🔴 Blocked     |
| Origin/CSRF/CORS policy         | 🔴 Blocked     |
| Cross-context refresh policy    | 🔴 Blocked     |
