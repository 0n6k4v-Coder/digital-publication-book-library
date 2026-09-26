# Frontend Admin Shell

## Table of Contents

1. [Scope](#scope)
2. [Routes](#routes)
3. [Requirements](#requirements)
4. [Layout](#layout)
5. [Authentication Behavior](#authentication-behavior)
6. [Logout](#logout)
7. [UI States](#ui-states)
8. [Security](#security)
9. [Accessibility](#accessibility)
10. [Testing](#testing)
11. [Implementation Criteria](#implementation-criteria)

---

# 1. Scope

This document defines the protected **Admin Shell** and its integration with routed Admin feature pages.

| Item                           | Reference                     |
| ------------------------------ | ----------------------------- |
| Frontend Scope                 | Protected Admin Shell         |
| Main Content                   | Routed Admin feature content  |
| Authentication Source of Truth | Backend Authentication domain |
| Authorization Source of Truth  | Backend Authorization domain  |
| Login Behavior                 | `login.md`                    |
| Authentication Behavior        | Backend Authentication domain |

The shell contains:

```text
Admin Shell
├── Sidebar
│   ├── Application Identity
│   ├── Accounts Navigation
│   └── Logout
└── Main Content Area
    └── Routed Admin Feature Page
```

The first routed Admin feature is:

```text
Account Management
└── Account List
```

The shell is responsible for:

* Protected Admin application layout
* Sidebar navigation
* Active navigation state
* Main content routing container
* Authentication-aware access
* Logout behavior

Feature-specific business logic remains outside the shell.

The following areas are outside this document:

| Excluded Area                 |
| ----------------------------- |
| Account List details          |
| Account CRUD logic            |
| Role management               |
| Post management               |
| Book management               |
| Dashboard logic               |
| Feature-specific API behavior |

---

# 2. Routes

| ID                  | Route             | Access          | Purpose      | Behavior                                         |
| ------------------- | ----------------- | --------------- | ------------ | ------------------------------------------------ |
| `FE_SHELL_ROUTE_01` | `/admin`          | Authenticated   | Admin Shell  | Display Admin Shell                              |
| `FE_SHELL_ROUTE_02` | `/admin`          | Bootstrap       | Admin Shell  | Resolve authentication before route decision     |
| `FE_SHELL_ROUTE_03` | `/admin`          | Unauthenticated | Admin Shell  | Redirect to `/login`                             |
| `FE_SHELL_ROUTE_04` | `/login`          | Authenticated   | Login page   | Redirect to `/admin`                             |
| `FE_SHELL_ROUTE_05` | `/login`          | Bootstrap       | Login page   | Keep route pending until authentication resolves |
| `FE_SHELL_ROUTE_06` | `/admin/accounts` | Authenticated   | Account List | Display Account List inside Admin Shell          |
| `FE_SHELL_ROUTE_07` | `/admin/accounts` | Bootstrap       | Account List | Resolve authentication before route decision     |
| `FE_SHELL_ROUTE_08` | `/admin/accounts` | Unauthenticated | Account List | Redirect to `/login`                             |

---

# 3. Requirements

| ID            | Requirement                                                                            |
| ------------- | -------------------------------------------------------------------------------------- |
| `FE_SHELL_01` | Provide a protected `/admin` route                                                     |
| `FE_SHELL_02` | Render the Admin Shell after authentication state resolves to `authenticated`          |
| `FE_SHELL_03` | Render a Sidebar within the Admin Shell                                                |
| `FE_SHELL_04` | Provide the Main Content Area as the routed feature container                          |
| `FE_SHELL_05` | Provide a Logout action in the Sidebar                                                 |
| `FE_SHELL_06` | Prevent unauthenticated access to `/admin` after authentication bootstrap completes    |
| `FE_SHELL_07` | Redirect unauthenticated users to `/login` after authentication bootstrap completes    |
| `FE_SHELL_08` | Logout must revoke the current authentication session through `POST /auth/logout`      |
| `FE_SHELL_09` | Logout must clear the client authentication state after successful logout              |
| `FE_SHELL_10` | After successful logout, navigate to `/login`                                          |
| `FE_SHELL_11` | Provide navigation to `/admin/accounts`                                                |
| `FE_SHELL_12` | Render the Account List inside the Admin Shell Main Content Area                       |
| `FE_SHELL_13` | Indicate the Accounts navigation item when `/admin/accounts` is active                 |
| `FE_SHELL_14` | Preserve the Admin Shell while navigating between Admin feature routes                 |
| `FE_SHELL_15` | Treat backend authorization as authoritative for Admin feature access                  |
| `FE_SHELL_16` | Represent authentication with `unknown`, `authenticated`, and `unauthenticated` states |
| `FE_SHELL_17` | Keep protected routes pending while authentication state is `unknown`                  |
| `FE_SHELL_18` | Restore browser authentication through the Authentication bootstrap flow               |
| `FE_SHELL_19` | Do not require an unexpired bearer access token for logout                             |
| `FE_SHELL_20` | Use the browser-managed refresh credential for logout                                  |

---

# 4. Layout

## Admin Shell

| ID               | Component            | Requirement                               |
| ---------------- | -------------------- | ----------------------------------------- |
| `FE_SHELL_UI_01` | Admin Shell          | Provide the protected application shell   |
| `FE_SHELL_UI_02` | Sidebar              | Render the Sidebar within the Admin Shell |
| `FE_SHELL_UI_03` | Main Content Area    | Render routed Admin feature content       |
| `FE_SHELL_UI_04` | Application Identity | Display Admin Application identity        |
| `FE_SHELL_UI_05` | Accounts Navigation  | Provide navigation to `/admin/accounts`   |
| `FE_SHELL_UI_06` | Logout Action        | Display the Logout action                 |
| `FE_SHELL_UI_07` | Active Navigation    | Indicate the active Admin feature         |

```text
┌─────────────────────────────────────────────────────────────┐
│ Sidebar                     Main Content                    │
│                                                             │
│ Admin Application           Routed Admin Feature            │
│ Accounts                                                    │
│                                                             │
│ Logout                                                      │
└─────────────────────────────────────────────────────────────┘
```

## Sidebar

| ID               | Requirement                                      |
| ---------------- | ------------------------------------------------ |
| `FE_SHELL_UI_08` | Display the Accounts navigation item             |
| `FE_SHELL_UI_09` | Navigate to `/admin/accounts`                    |
| `FE_SHELL_UI_10` | Indicate Accounts as active on `/admin/accounts` |
| `FE_SHELL_UI_11` | Keep Logout separate from feature navigation     |
| `FE_SHELL_UI_12` | Support keyboard navigation and visible focus    |

## Main Content

| ID               | Requirement                                               |
| ---------------- | --------------------------------------------------------- |
| `FE_SHELL_UI_13` | Provide a stable container for routed Admin feature pages |
| `FE_SHELL_UI_14` | Render Account List content for `/admin/accounts`         |
| `FE_SHELL_UI_15` | Preserve the Admin Shell while the feature page changes   |
| `FE_SHELL_UI_16` | Keep feature-specific business logic outside the shell    |

## Responsive Layout

| Viewport       | Sidebar                              | Main Content         |
| -------------- | ------------------------------------ | -------------------- |
| `≥ 1280px`     | Persistent fixed-width sidebar       | Remaining width      |
| `768px–1279px` | Persistent reduced-width sidebar     | Remaining width      |
| `< 768px`      | Collapsible sidebar via menu control | Full available width |

Responsive requirements:

| ID                 | Requirement                                                              |
| ------------------ | ------------------------------------------------------------------------ |
| `FE_SHELL_RESP_01` | Prevent unintended horizontal page scrolling                             |
| `FE_SHELL_RESP_02` | Preserve active navigation state at every viewport size                  |
| `FE_SHELL_RESP_03` | Keep Sidebar navigation keyboard accessible                              |
| `FE_SHELL_RESP_04` | Allow mobile navigation to be dismissed without leaving the current page |
| `FE_SHELL_RESP_05` | Allow feature pages to manage their own internal responsive behavior     |
| `FE_SHELL_RESP_06` | Preserve authentication and authorization behavior across viewport sizes |

---

# 5. Authentication Behavior

## Authentication States

The Admin Shell consumes the frontend Authentication service state:

```text
unknown
authenticated
unauthenticated
```

`unknown` means browser authentication bootstrap is still being evaluated.

## Authenticated Access

| ID                 | Condition                               | Behavior             |
| ------------------ | --------------------------------------- | -------------------- |
| `FE_SHELL_AUTH_01` | Authentication state is `authenticated` | Render Admin Shell   |
| `FE_SHELL_AUTH_02` | Authenticated user opens `/login`       | Redirect to `/admin` |

## Bootstrap Access

| ID                 | Condition                         | Behavior                                     |
| ------------------ | --------------------------------- | -------------------------------------------- |
| `FE_SHELL_AUTH_03` | Authentication state is `unknown` | Keep route pending; do not redirect to login |
| `FE_SHELL_AUTH_04` | Bootstrap refresh succeeds        | Transition to `authenticated`                |
| `FE_SHELL_AUTH_05` | Bootstrap refresh returns `401`   | Transition to `unauthenticated`              |

## Unauthenticated Access

| ID                 | Condition                                                         | Behavior                      |
| ------------------ | ----------------------------------------------------------------- | ----------------------------- |
| `FE_SHELL_AUTH_06` | User opens `/admin` after bootstrap resolves to `unauthenticated` | Redirect to `/login`          |
| `FE_SHELL_AUTH_07` | Authentication state becomes invalid after bootstrap              | Treat user as unauthenticated |

## Authentication Authority

| ID                 | Requirement                                                                 |
| ------------------ | --------------------------------------------------------------------------- |
| `FE_SHELL_AUTH_08` | Use the authentication state established by the Authentication flow.        |
| `FE_SHELL_AUTH_09` | Do not determine authentication from user-supplied account IDs.             |
| `FE_SHELL_AUTH_10` | Do not determine authentication from user-supplied roles.                   |
| `FE_SHELL_AUTH_11` | Do not determine authentication from user-supplied permissions.             |
| `FE_SHELL_AUTH_12` | Do not determine authentication from other client-provided identity values. |
| `FE_SHELL_AUTH_13` | Treat the backend as the authority for authentication.                      |

---

# 6. Logout

Logout is initiated from the Sidebar.

## Logout Flow

| Step | Action                                                            |
| ---- | ----------------------------------------------------------------- |
| 1    | User selects Logout.                                              |
| 2    | Frontend sends `POST /auth/logout` with `credentials: "include"`. |
| 3    | Browser supplies the `__Host-refresh_token` cookie.               |
| 4    | Backend revokes the authentication session.                       |
| 5    | Backend expires the refresh cookie.                               |
| 6    | Frontend clears the in-memory access token.                       |
| 7    | Frontend navigates to `/login`.                                   |

## Request

| ID                       | Property     | Value                                 |
| ------------------------ | ------------ | ------------------------------------- |
| `FE_SHELL_LOGOUT_API_01` | Method       | `POST`                                |
| `FE_SHELL_LOGOUT_API_02` | Endpoint     | `/auth/logout`                        |
| `FE_SHELL_LOGOUT_API_03` | Credentials  | Browser `__Host-refresh_token` cookie |
| `FE_SHELL_LOGOUT_API_04` | Request Body | None                                  |

The logout request must not depend on an unexpired bearer access token.

## Success

| ID                       | Property      | Value                         |
| ------------------------ | ------------- | ----------------------------- |
| `FE_SHELL_LOGOUT_API_05` | Status        | `204 No Content`              |
| `FE_SHELL_LOGOUT_API_06` | Cache-Control | `no-store`                    |
| `FE_SHELL_LOGOUT_API_07` | Cookie Action | Expire `__Host-refresh_token` |

## Invalid or Missing Session Credential

Logout is idempotent.

A missing, expired, revoked, or already-used refresh credential must not expose authentication state to the UI.

The server must still expire the browser refresh cookie and return successful logout semantics.

## Logout Failure

| ID                   | Requirement                                                     |
| -------------------- | --------------------------------------------------------------- |
| `FE_SHELL_LOGOUT_01` | Prevent duplicate logout requests.                              |
| `FE_SHELL_LOGOUT_02` | Disable the Logout action while the request is pending.         |
| `FE_SHELL_LOGOUT_03` | Do not expose the access token in the UI.                       |
| `FE_SHELL_LOGOUT_04` | Do not log the access token.                                    |
| `FE_SHELL_LOGOUT_05` | Clear in-memory authentication only after successful logout.    |
| `FE_SHELL_LOGOUT_06` | Do not remain on `/admin` after successful logout.              |
| `FE_SHELL_LOGOUT_07` | On network failure, retain authenticated state and allow retry. |

---

# 7. UI States

| ID                  | State               | Behavior                                                                 |
| ------------------- | ------------------- | ------------------------------------------------------------------------ |
| `FE_SHELL_STATE_01` | Bootstrap Pending   | Keep authentication state `unknown` while `/auth/refresh` is evaluated.  |
| `FE_SHELL_STATE_02` | Authenticated       | Render Admin Shell                                                       |
| `FE_SHELL_STATE_03` | Unauthenticated     | Redirect protected routes to `/login`                                    |
| `FE_SHELL_STATE_04` | Logout Pending      | Disable Logout action                                                    |
| `FE_SHELL_STATE_05` | Logout Success      | Clear auth state and navigate to `/login`                                |
| `FE_SHELL_STATE_06` | Logout Failure      | Keep authenticated state and allow retry                                 |
| `FE_SHELL_STATE_07` | Account Route       | Render Account List inside Main Content                                  |
| `FE_SHELL_STATE_08` | Active Accounts Nav | Accounts navigation item is visibly active                               |
| `FE_SHELL_STATE_09` | Feature Loading     | Keep the Admin Shell rendered while feature content loads                |
| `FE_SHELL_STATE_10` | Feature Error       | Keep the Admin Shell rendered while feature content handles its error    |
| `FE_SHELL_STATE_11` | Feature Empty       | Keep the Admin Shell rendered while feature content handles its empty UI |

---

# 8. Security

The Admin Shell must follow the Authentication and Authorization domain contracts.

| ID                | Security Rule            | Requirement                                                                                   |
| ----------------- | ------------------------ | --------------------------------------------------------------------------------------------- |
| `FE_SHELL_SEC_01` | Protected Route          | `/admin` must not be accessible after authentication bootstrap resolves to `unauthenticated`. |
| `FE_SHELL_SEC_02` | Bootstrap State          | `unknown` authentication state must not be treated as unauthenticated.                        |
| `FE_SHELL_SEC_03` | Authentication Rules     | The frontend must not implement its own authentication rules.                                 |
| `FE_SHELL_SEC_04` | Authentication Authority | Treat the backend as the authority for authentication.                                        |
| `FE_SHELL_SEC_05` | Access Token             | Send bearer credentials only through the `Authorization` header for protected Admin APIs.     |
| `FE_SHELL_SEC_06` | Refresh Credential       | Never read or persist the refresh credential in JavaScript storage.                           |
| `FE_SHELL_SEC_07` | Token URLs               | Never place tokens in URLs.                                                                   |
| `FE_SHELL_SEC_08` | Token Display            | Never render tokens.                                                                          |
| `FE_SHELL_SEC_09` | Token Logging            | Never log tokens.                                                                             |
| `FE_SHELL_SEC_10` | Cookie Requests          | Use `credentials: "include"` for refresh/logout requests.                                     |
| `FE_SHELL_SEC_11` | CSRF                     | Backend authentication endpoints must enforce configured `Origin` and `SameSite` protections. |
| `FE_SHELL_SEC_12` | Authorization            | Do not treat client-side authentication state as proof of authorization.                      |

---

# 9. Accessibility

| ID                 | Accessibility Requirement                                               |
| ------------------ | ----------------------------------------------------------------------- |
| `FE_SHELL_A11Y_01` | Use semantic layout elements                                            |
| `FE_SHELL_A11Y_02` | Provide an accessible name for the Sidebar                              |
| `FE_SHELL_A11Y_03` | Provide an accessible name for the Logout action                        |
| `FE_SHELL_A11Y_04` | Support keyboard navigation                                             |
| `FE_SHELL_A11Y_05` | Provide visible focus states                                            |
| `FE_SHELL_A11Y_06` | Communicate the Logout pending state appropriately                      |
| `FE_SHELL_A11Y_07` | Layout remains usable across desktop, tablet, and mobile viewport sizes |

---

# 10. Testing

## Unit

| ID                      | Test                                                          |
| ----------------------- | ------------------------------------------------------------- |
| `FE_SHELL_TEST_UNIT_01` | Admin Shell renders correctly                                 |
| `FE_SHELL_TEST_UNIT_02` | Sidebar renders correctly                                     |
| `FE_SHELL_TEST_UNIT_03` | Accounts navigation item renders correctly                    |
| `FE_SHELL_TEST_UNIT_04` | Accounts navigation active state is correct                   |
| `FE_SHELL_TEST_UNIT_05` | Logout button state is handled correctly                      |
| `FE_SHELL_TEST_UNIT_06` | `unknown` authentication state keeps protected routes pending |
| `FE_SHELL_TEST_UNIT_07` | Authenticated route guard works correctly                     |
| `FE_SHELL_TEST_UNIT_08` | Unauthenticated route guard works correctly                   |
| `FE_SHELL_TEST_UNIT_09` | Logout failure preserves authenticated state                  |
| `FE_SHELL_TEST_UNIT_10` | Duplicate logout submission is prevented                      |

## Integration

| ID                     | Test                                                              |
| ---------------------- | ----------------------------------------------------------------- |
| `FE_SHELL_TEST_INT_01` | Authentication bootstrap is triggered when the application starts |
| `FE_SHELL_TEST_INT_02` | Successful bootstrap establishes authenticated state              |
| `FE_SHELL_TEST_INT_03` | Failed bootstrap establishes unauthenticated state                |
| `FE_SHELL_TEST_INT_04` | `/admin` remains pending while authentication state is `unknown`  |
| `FE_SHELL_TEST_INT_05` | Authenticated user can access `/admin`                            |
| `FE_SHELL_TEST_INT_06` | Unauthenticated user is redirected to `/login` after bootstrap    |
| `FE_SHELL_TEST_INT_07` | Authenticated user accessing `/login` is redirected to `/admin`   |
| `FE_SHELL_TEST_INT_08` | Authenticated user can access `/admin/accounts`                   |
| `FE_SHELL_TEST_INT_09` | Account List renders inside the Admin Shell                       |
| `FE_SHELL_TEST_INT_10` | Accounts navigation becomes active                                |
| `FE_SHELL_TEST_INT_11` | Logout sends `POST /auth/logout` with `credentials: "include"`    |
| `FE_SHELL_TEST_INT_12` | Logout does not send a bearer access token                        |
| `FE_SHELL_TEST_INT_13` | Successful logout clears authentication state                     |
| `FE_SHELL_TEST_INT_14` | Successful logout navigates to `/login`                           |
| `FE_SHELL_TEST_INT_15` | Logout network failure preserves authenticated state              |
| `FE_SHELL_TEST_INT_16` | Duplicate logout submission is prevented                          |
| `FE_SHELL_TEST_INT_17` | Access-token `401` can trigger one serialized refresh attempt     |
| `FE_SHELL_TEST_INT_18` | Successful refresh replaces the in-memory access token            |
| `FE_SHELL_TEST_INT_19` | Refresh failure clears authentication state                       |

## E2E

| ID                     | Test                                                                            |
| ---------------------- | ------------------------------------------------------------------------------- |
| `FE_SHELL_TEST_E2E_01` | User logs in and the Admin Shell appears                                        |
| `FE_SHELL_TEST_E2E_02` | Full page reload restores the authenticated Admin session                       |
| `FE_SHELL_TEST_E2E_03` | Sidebar appears in the Admin Shell                                              |
| `FE_SHELL_TEST_E2E_04` | Accounts navigation is visible                                                  |
| `FE_SHELL_TEST_E2E_05` | User navigates to `/admin/accounts`                                             |
| `FE_SHELL_TEST_E2E_06` | Account List appears inside the Admin Shell                                     |
| `FE_SHELL_TEST_E2E_07` | Accounts navigation appears active                                              |
| `FE_SHELL_TEST_E2E_08` | User logs out and `/login` appears                                              |
| `FE_SHELL_TEST_E2E_09` | Logout expires the browser refresh credential                                   |
| `FE_SHELL_TEST_E2E_10` | Unauthenticated access to `/admin` redirects after bootstrap                    |
| `FE_SHELL_TEST_E2E_11` | Unauthenticated access to `/admin/accounts` redirects after bootstrap           |
| `FE_SHELL_TEST_E2E_12` | Access-token expiration recovers through refresh without an authentication loop |
| `FE_SHELL_TEST_E2E_13` | Logout does not depend on an unexpired bearer access token                      |

## Manual

| ID                        | Test                                          |
| ------------------------- | --------------------------------------------- |
| `FE_SHELL_TEST_MANUAL_01` | Sidebar layout is verified                    |
| `FE_SHELL_TEST_MANUAL_02` | Accounts navigation is verified               |
| `FE_SHELL_TEST_MANUAL_03` | Active Accounts state is verified             |
| `FE_SHELL_TEST_MANUAL_04` | Account List is rendered inside the shell     |
| `FE_SHELL_TEST_MANUAL_05` | Login → Admin transition is verified          |
| `FE_SHELL_TEST_MANUAL_06` | Full page reload preserves authentication     |
| `FE_SHELL_TEST_MANUAL_07` | Logout behavior is verified                   |
| `FE_SHELL_TEST_MANUAL_08` | Redirect behavior is verified after bootstrap |
| `FE_SHELL_TEST_MANUAL_09` | Keyboard interaction is verified              |
| `FE_SHELL_TEST_MANUAL_10` | Responsive layout is verified                 |
| `FE_SHELL_TEST_MANUAL_11` | Visible focus states are verified             |

---

# 11. Implementation Criteria

### Status Values

| Status         | Meaning                                             |
| -------------- | --------------------------------------------------- |
| ⚪ Not Started  | Criteria has not been implemented or verified       |
| 🟡 In Progress | Implementation or verification is still in progress |
| 🟢 Implemented | Implementation is complete and verified             |
| 🔴 Blocked     | Implementation cannot proceed because of a blocker  |

## Authentication Behavior

| ID                 | Criteria                                                                       | Status         | Reason                                           |
| ------------------ | ------------------------------------------------------------------------------ | -------------- | ------------------------------------------------ |
| `FE_SHELL_AUTH_01` | Authenticated state renders the Admin Shell                                    | 🟢 Implemented | Existing shell implementation                    |
| `FE_SHELL_AUTH_02` | Authenticated users accessing `/login` are redirected to `/admin`              | 🟢 Implemented | Existing route behavior                          |
| `FE_SHELL_AUTH_03` | `unknown` authentication state does not redirect protected routes to `/login`  | 🟡 In Progress | Requires authentication bootstrap implementation |
| `FE_SHELL_AUTH_04` | Successful `/auth/refresh` transitions authentication state to `authenticated` | 🟡 In Progress | Requires refresh bootstrap implementation        |
| `FE_SHELL_AUTH_05` | Failed `/auth/refresh` transitions authentication state to `unauthenticated`   | 🟡 In Progress | Requires refresh bootstrap implementation        |
| `FE_SHELL_AUTH_06` | Unauthenticated `/admin` access redirects after bootstrap resolves             | 🟡 In Progress | Requires three-state authentication model        |
| `FE_SHELL_AUTH_07` | Invalid authentication state is treated as unauthenticated                     | 🟡 In Progress | Depends on bootstrap state model                 |
| `FE_SHELL_AUTH_08` | Authentication state comes from the Authentication flow                        | 🟢 Implemented | Existing authentication service boundary         |
| `FE_SHELL_AUTH_09` | Client-supplied account IDs are not used to determine authentication           | 🟢 Implemented | Existing architecture                            |
| `FE_SHELL_AUTH_10` | Client-supplied roles are not used to determine authentication                 | 🟢 Implemented | Existing architecture                            |
| `FE_SHELL_AUTH_11` | Client-supplied permissions are not used to determine authentication           | 🟢 Implemented | Existing architecture                            |
| `FE_SHELL_AUTH_12` | Other client-provided identity values are not used to determine authentication | 🟢 Implemented | Existing architecture                            |
| `FE_SHELL_AUTH_13` | Backend remains the authentication authority                                   | 🟢 Implemented | Existing architecture                            |

## Logout API

| ID                       | Criteria                                                        | Status         | Reason                                 |
| ------------------------ | --------------------------------------------------------------- | -------------- | -------------------------------------- |
| `FE_SHELL_LOGOUT_API_01` | Logout uses `POST`                                              | 🟢 Implemented | Existing endpoint                      |
| `FE_SHELL_LOGOUT_API_02` | Logout uses `/auth/logout`                                      | 🟢 Implemented | Existing endpoint                      |
| `FE_SHELL_LOGOUT_API_03` | Logout uses `credentials: "include"`                            | 🟡 In Progress | Requires browser-cookie implementation |
| `FE_SHELL_LOGOUT_API_04` | Logout does not depend on an unexpired bearer access token      | 🟡 In Progress | Backend/frontend contract changed      |
| `FE_SHELL_LOGOUT_API_05` | Successful logout returns `204 No Content`                      | 🟢 Implemented | Existing backend contract              |
| `FE_SHELL_LOGOUT_API_06` | Successful logout uses `Cache-Control: no-store`                | 🟢 Implemented | Existing backend implementation        |
| `FE_SHELL_LOGOUT_API_07` | Successful logout expires `__Host-refresh_token`                | 🟡 In Progress | Requires browser-cookie implementation |
| `FE_SHELL_LOGOUT_API_08` | Logout is idempotent for missing or invalid refresh credentials | 🟡 In Progress | Requires backend implementation        |

## Logout

| ID                   | Criteria                                                        | Status         | Reason                              |
| -------------------- | --------------------------------------------------------------- | -------------- | ----------------------------------- |
| `FE_SHELL_LOGOUT_01` | Duplicate logout requests are prevented                         | 🟢 Implemented | Existing Admin Shell implementation |
| `FE_SHELL_LOGOUT_02` | Logout action is disabled while the request is pending          | 🟢 Implemented | Existing Admin Shell implementation |
| `FE_SHELL_LOGOUT_03` | Access token is never exposed in the UI                         | 🟢 Implemented | Existing implementation             |
| `FE_SHELL_LOGOUT_04` | Access token is never logged                                    | 🟢 Implemented | Existing implementation             |
| `FE_SHELL_LOGOUT_05` | Client authentication state is cleared after successful logout  | 🟢 Implemented | Existing implementation             |
| `FE_SHELL_LOGOUT_06` | User does not remain on `/admin` after successful logout        | 🟢 Implemented | Existing route behavior             |
| `FE_SHELL_LOGOUT_07` | Network failure preserves authenticated state and permits retry | 🟢 Implemented | Existing Admin Shell behavior       |

## Security

| ID                | Criteria                                                               | Status         | Reason                                  |
| ----------------- | ---------------------------------------------------------------------- | -------------- | --------------------------------------- |
| `FE_SHELL_SEC_01` | `/admin` is inaccessible after bootstrap resolves to unauthenticated   | 🟡 In Progress | Requires bootstrap state                |
| `FE_SHELL_SEC_02` | `unknown` authentication state is not treated as unauthenticated       | 🟡 In Progress | Requires bootstrap state                |
| `FE_SHELL_SEC_03` | Frontend does not implement independent authentication rules           | 🟢 Implemented | Existing architecture                   |
| `FE_SHELL_SEC_04` | Backend is treated as the authentication authority                     | 🟢 Implemented | Existing architecture                   |
| `FE_SHELL_SEC_05` | Bearer credentials are used only in the Authorization header           | 🟢 Implemented | Existing protected API design           |
| `FE_SHELL_SEC_06` | Refresh credentials are never read or persisted in JavaScript storage  | 🟡 In Progress | Requires HttpOnly cookie implementation |
| `FE_SHELL_SEC_07` | Tokens are never placed in URLs                                        | 🟢 Implemented | Existing implementation                 |
| `FE_SHELL_SEC_08` | Tokens are never rendered                                              | 🟢 Implemented | Existing implementation                 |
| `FE_SHELL_SEC_09` | Tokens are never logged                                                | 🟢 Implemented | Existing implementation                 |
| `FE_SHELL_SEC_10` | Authentication requests use browser credentials when required          | 🟡 In Progress | Requires `credentials: "include"`       |
| `FE_SHELL_SEC_11` | Authentication endpoints rely on backend Origin/SameSite protections   | 🟡 In Progress | Requires backend implementation         |
| `FE_SHELL_SEC_12` | Client-side authentication state is not treated as authorization proof | 🟢 Implemented | Existing architecture                   |

## Testing — Unit

| ID                      | Criteria                                                      | Status         | Reason                               |
| ----------------------- | ------------------------------------------------------------- | -------------- | ------------------------------------ |
| `FE_SHELL_TEST_UNIT_01` | Admin Shell rendering is tested                               | 🟢 Implemented | Existing unit test                   |
| `FE_SHELL_TEST_UNIT_02` | Sidebar rendering is tested                                   | 🟢 Implemented | Existing unit test                   |
| `FE_SHELL_TEST_UNIT_03` | Logout button state is tested                                 | 🟢 Implemented | Existing unit test                   |
| `FE_SHELL_TEST_UNIT_04` | Authenticated route guard behavior is tested                  | 🟡 In Progress | Requires bootstrap-aware route tests |
| `FE_SHELL_TEST_UNIT_05` | Unauthenticated route guard behavior is tested                | 🟡 In Progress | Requires bootstrap-aware route tests |
| `FE_SHELL_TEST_UNIT_06` | `unknown` authentication state keeps protected routes pending | 🟡 In Progress | Requires bootstrap implementation    |
| `FE_SHELL_TEST_UNIT_07` | Accounts navigation and active state are tested               | 🟢 Implemented | Existing unit/integration tests      |
| `FE_SHELL_TEST_UNIT_08` | Logout failure preserves authenticated state                  | 🟢 Implemented | Existing Admin Shell unit test       |
| `FE_SHELL_TEST_UNIT_09` | Duplicate logout submission is prevented                      | 🟢 Implemented | Existing Admin Shell unit test       |

## Testing — Integration

| ID                     | Criteria                                                                      | Status         | Reason                              |
| ---------------------- | ----------------------------------------------------------------------------- | -------------- | ----------------------------------- |
| `FE_SHELL_TEST_INT_01` | Auth bootstrap successfully restores authentication after application startup | 🟡 In Progress | Requires bootstrap implementation   |
| `FE_SHELL_TEST_INT_02` | Authenticated user can access `/admin` after bootstrap                        | 🟡 In Progress | Requires bootstrap implementation   |
| `FE_SHELL_TEST_INT_03` | Unauthenticated user is redirected to `/login` after bootstrap                | 🟡 In Progress | Requires bootstrap implementation   |
| `FE_SHELL_TEST_INT_04` | Authenticated user accessing `/login` is redirected to `/admin`               | 🟢 Implemented | Existing integration test           |
| `FE_SHELL_TEST_INT_05` | Logout sends `POST /auth/logout` with browser credentials                     | 🟡 In Progress | Requires cookie-based logout        |
| `FE_SHELL_TEST_INT_06` | Logout does not send a bearer access token                                    | 🟡 In Progress | Existing implementation must change |
| `FE_SHELL_TEST_INT_07` | Logout success clears authentication state                                    | 🟢 Implemented | Existing integration behavior       |
| `FE_SHELL_TEST_INT_08` | Logout success navigates to `/login`                                          | 🟢 Implemented | Existing integration behavior       |
| `FE_SHELL_TEST_INT_09` | Duplicate logout submission is prevented                                      | 🟢 Implemented | Existing integration behavior       |
| `FE_SHELL_TEST_INT_10` | Access-token `401` can trigger one serialized refresh attempt                 | 🟡 In Progress | Requires refresh implementation     |
| `FE_SHELL_TEST_INT_11` | Successful refresh replaces the in-memory access token                        | 🟡 In Progress | Requires refresh implementation     |
| `FE_SHELL_TEST_INT_12` | Refresh failure clears authentication state                                   | 🟡 In Progress | Requires refresh implementation     |

## Testing — E2E

| ID                     | Criteria                                                                        | Status         | Reason                                    |
| ---------------------- | ------------------------------------------------------------------------------- | -------------- | ----------------------------------------- |
| `FE_SHELL_TEST_E2E_01` | Login leads to the Admin Shell                                                  | 🟢 Implemented | Existing E2E coverage                     |
| `FE_SHELL_TEST_E2E_02` | Full page reload preserves the authenticated Admin session                      | 🟡 In Progress | Requires browser refresh-cookie bootstrap |
| `FE_SHELL_TEST_E2E_03` | Sidebar appears in the Admin Shell                                              | 🟢 Implemented | Existing E2E coverage                     |
| `FE_SHELL_TEST_E2E_04` | Accounts navigation is visible                                                  | 🟢 Implemented | Existing E2E coverage                     |
| `FE_SHELL_TEST_E2E_05` | User navigates to `/admin/accounts`                                             | 🟢 Implemented | Existing E2E coverage                     |
| `FE_SHELL_TEST_E2E_06` | Account List renders inside the Admin Shell                                     | 🟢 Implemented | Existing E2E coverage                     |
| `FE_SHELL_TEST_E2E_07` | Accounts navigation appears active                                              | 🟢 Implemented | Existing E2E coverage                     |
| `FE_SHELL_TEST_E2E_08` | User logs out and `/login` appears                                              | 🟢 Implemented | Existing E2E coverage                     |
| `FE_SHELL_TEST_E2E_09` | Logout expires the browser refresh credential                                   | 🟡 In Progress | Requires cookie-based logout              |
| `FE_SHELL_TEST_E2E_10` | Unauthenticated access to `/admin` redirects after bootstrap                    | 🟡 In Progress | Requires bootstrap implementation         |
| `FE_SHELL_TEST_E2E_11` | Unauthenticated access to `/admin/accounts` redirects after bootstrap           | 🟡 In Progress | Requires bootstrap implementation         |
| `FE_SHELL_TEST_E2E_12` | Access-token expiration recovers through refresh without an authentication loop | 🟡 In Progress | Requires refresh implementation           |
| `FE_SHELL_TEST_E2E_13` | Logout does not depend on an unexpired bearer access token                      | 🟡 In Progress | Requires cookie-based logout              |

## Testing — Manual

| ID                        | Criteria                                  | Status         | Reason                  |
| ------------------------- | ----------------------------------------- | -------------- | ----------------------- |
| `FE_SHELL_TEST_MANUAL_01` | Sidebar layout is verified                | 🟡 In Progress | Not yet verified        |
| `FE_SHELL_TEST_MANUAL_02` | Main Content Area is verified             | 🟡 In Progress | Not yet verified        |
| `FE_SHELL_TEST_MANUAL_03` | Login → Admin transition is verified      | 🟡 In Progress | Not yet verified        |
| `FE_SHELL_TEST_MANUAL_04` | Full page reload preserves authentication | 🟡 In Progress | Requires implementation |
| `FE_SHELL_TEST_MANUAL_05` | Logout behavior is verified               | 🟡 In Progress | Requires implementation |
| `FE_SHELL_TEST_MANUAL_06` | Redirect behavior is verified             | 🟡 In Progress | Requires implementation |
| `FE_SHELL_TEST_MANUAL_07` | Keyboard interaction is verified          | 🟡 In Progress | Not yet verified        |
| `FE_SHELL_TEST_MANUAL_08` | Responsive layout is verified             | 🟡 In Progress | Not yet verified        |
| `FE_SHELL_TEST_MANUAL_09` | Visible focus states are verified         | 🟡 In Progress | Not yet verified        |
