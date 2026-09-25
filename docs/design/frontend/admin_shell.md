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
11. [Acceptance Criteria](#acceptance-criteria)

---

# 1. Scope

This document defines the protected Admin Shell for the first Admin Application vertical slice.

The shell contains:

```text
Admin Shell
├── Sidebar
└── Main Content Area
```

The Main Content Area is intentionally empty.

This document does not define:

* Account management
* Role management
* Post management
* Book management
* Dashboard functionality
* Business logic
* Feature-specific navigation

Authentication behavior follows `authentication.md`.

Login behavior follows `login.md`.

---

# 2. Routes

| Route    | Access          | Purpose           |
| -------- | --------------- | ----------------- |
| `/admin` | Authenticated   | Empty Admin Shell |
| `/login` | Unauthenticated | Login page        |

Unauthenticated users attempting to access `/admin` must be redirected to `/login`.

Authenticated users attempting to access `/login` should be redirected to `/admin`.

---

# 3. Requirements

| ID            | Requirement                                                                        |
| ------------- | ---------------------------------------------------------------------------------- |
| `FE_SHELL_01` | Provide a protected `/admin` route.                                                |
| `FE_SHELL_02` | Render the Admin Shell after successful authentication.                            |
| `FE_SHELL_03` | Render a Sidebar within the Admin Shell.                                           |
| `FE_SHELL_04` | Render an empty Main Content Area.                                                 |
| `FE_SHELL_05` | Provide a Logout action in the Sidebar.                                            |
| `FE_SHELL_06` | Prevent unauthenticated access to `/admin`.                                        |
| `FE_SHELL_07` | Redirect unauthenticated users to `/login`.                                        |
| `FE_SHELL_08` | Logout must revoke the current authentication session through `POST /auth/logout`. |
| `FE_SHELL_09` | Logout must clear the client authentication state.                                 |
| `FE_SHELL_10` | After logout, navigate to `/login`.                                                |

---

# 4. Layout

The initial Admin Shell contains only:

```text
┌─────────────────────────────────────────────┐
│                                             │
│ Sidebar              Main Content           │
│                                             │
│ Admin Application                            │
│                                             │
│                                             │
│                                             │
│ Logout                                       │
│                                             │
└─────────────────────────────────────────────┘
```

## Sidebar

The Sidebar must contain:

* Admin Application identity
* Logout action

No feature navigation is required yet.

Future navigation items may be added when their corresponding frontend designs are implemented.

## Main Content

The Main Content Area must be empty.

It must provide the application content container for future pages.

---

# 5. Authentication Behavior

## Authenticated Access

```text
Authenticated
    ↓
/admin
    ↓
Render Admin Shell
```

## Unauthenticated Access

```text
Unauthenticated
    ↓
/admin
    ↓
Redirect to /login
```

## Login Redirect

```text
Authenticated
    ↓
/login
    ↓
Redirect to /admin
```

The frontend must use the authentication state established by the Authentication flow.

The frontend must not determine authentication from user-supplied account IDs, roles, permissions, or other client-provided identity values.

The backend remains the authority for authentication.

---

# 6. Logout

Logout is initiated from the Sidebar.

```text
User selects Logout
        ↓
POST /auth/logout
        ↓
Clear client authentication state
        ↓
Navigate to /login
```

## Request

```http
POST /auth/logout
Authorization: Bearer <access-token>
```

## Success

```text
204 No Content
Cache-Control: no-store
```

## Unauthorized Logout

If `/auth/logout` returns `401 Unauthorized`:

```text
Clear client authentication state
        ↓
Navigate to /login
```

The client must treat the current authentication state as invalid.

## Logout Requirements

* Prevent duplicate logout requests.
* Disable the Logout action while the request is pending.
* Do not expose the access token in the UI.
* Do not log the access token.
* Clear client authentication state after logout.
* Do not remain on `/admin` after logout.

---

# 7. UI States

| State               | Behavior                                                                           |
| ------------------- | ---------------------------------------------------------------------------------- |
| Authenticated       | Render Admin Shell                                                                 |
| Unauthenticated     | Redirect to `/login`                                                               |
| Logout Pending      | Disable Logout action                                                              |
| Logout Success      | Clear auth state and navigate to `/login`                                          |
| Logout Unauthorized | Clear auth state and navigate to `/login`                                          |
| Logout Failure      | Keep authenticated state and allow retry, unless authentication is no longer valid |

The Main Content Area remains empty for this implementation.

---

# 8. Security

The Admin Shell must follow the Authentication and Authorization domain contracts.

* `/admin` must not be accessible without authentication.
* The frontend must not implement its own authentication rules.
* The backend remains authoritative for authentication.
* Bearer credentials must only be sent through the `Authorization` header.
* Tokens must never be placed in URLs.
* Tokens must never be rendered.
* Tokens must never be logged.
* Passwords must never be stored or rendered by the Admin Shell.
* Client-side authentication state must not be treated as proof of authorization.
* Future permission checks must not replace server-side authorization.

Relevant backend contracts include:

```text
AU_SEC_REQ_01
AU_SEC_REQ_02
AU_SEC_REQ_03
AU_SEC_REQ_04
AU_SEC_REQ_05
AU_SEC_REQ_06
AU_SEC_REQ_07
AU_SEC_REQ_09
```

---

# 9. Accessibility

The Admin Shell must:

* Use semantic layout elements.
* Provide an accessible name for the Sidebar.
* Provide an accessible name for the Logout action.
* Support keyboard navigation.
* Provide visible focus states.
* Communicate the Logout pending state appropriately.
* Maintain usable layout behavior on supported viewport sizes.

---

# 10. Testing

## Unit

Test:

* Admin Shell rendering.
* Sidebar rendering.
* Logout button state.
* Authenticated/unauthenticated route guard behavior.

## Integration

Test:

* Authenticated user can access `/admin`.
* Unauthenticated user is redirected to `/login`.
* Authenticated user accessing `/login` is redirected to `/admin`.
* Logout sends `POST /auth/logout`.
* Logout success clears authentication state.
* Logout success navigates to `/login`.
* Logout `401` clears authentication state.
* Duplicate logout submission is prevented.

## E2E

Test:

```text
Login
  ↓
Admin Shell appears
  ↓
Sidebar appears
  ↓
Logout
  ↓
/login appears
```

Also test:

```text
Unauthenticated
  ↓
Open /admin
  ↓
Redirect to /login
```

## Manual

Verify:

* Sidebar layout.
* Empty content area.
* Login → Admin transition.
* Logout behavior.
* Redirect behavior.
* Keyboard interaction.
* Responsive layout.
* Visible focus states.

---

# 11. Acceptance Criteria

The Admin Shell implementation is complete when:

```text
✓ /admin is protected
✓ Authenticated users can access /admin
✓ Unauthenticated users are redirected to /login
✓ Authenticated users are redirected from /login to /admin
✓ Sidebar is rendered
✓ Main Content Area is empty
✓ Logout action is available
✓ POST /auth/logout is called correctly
✓ Logout state prevents duplicate submission
✓ Successful logout clears authentication state
✓ 401 logout clears authentication state
✓ Logout navigates to /login
✓ Tokens are not rendered
✓ Tokens are not logged
✓ Unit tests exist
✓ Integration tests exist
✓ E2E flow exists
✓ Manual verification is complete
```
