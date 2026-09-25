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

This document defines the protected **Admin Shell** for the first Admin Application vertical slice.

| Item                           | Reference                     |
| ------------------------------ | ----------------------------- |
| Frontend Scope                 | Protected Admin Shell         |
| Main Content                   | Intentionally empty           |
| Authentication Source of Truth | Backend Authentication domain |
| Login Behavior                 | `login.md`                    |
| Authentication Behavior        | Backend Authentication domain |

The shell contains:

```text
Admin Shell
├── Sidebar
└── Main Content Area
```

This document does not define:

| Excluded Area               |
| --------------------------- |
| Account management          |
| Role management             |
| Post management             |
| Book management             |
| Dashboard functionality     |
| Business logic              |
| Feature-specific navigation |

---

# 2. Routes

| ID                  | Route    | Access          | Purpose     | Behavior             |
| ------------------- | -------- | --------------- | ----------- | -------------------- |
| `FE_SHELL_ROUTE_01` | `/admin` | Authenticated   | Admin Shell | Display Admin Shell  |
| `FE_SHELL_ROUTE_02` | `/admin` | Unauthenticated | Admin Shell | Redirect to `/login` |
| `FE_SHELL_ROUTE_03` | `/login` | Authenticated   | Login page  | Redirect to `/admin` |

---

# 3. Requirements

| ID            | Requirement                                                                       |
| ------------- | --------------------------------------------------------------------------------- |
| `FE_SHELL_01` | Provide a protected `/admin` route                                                |
| `FE_SHELL_02` | Render the Admin Shell after successful authentication                            |
| `FE_SHELL_03` | Render a Sidebar within the Admin Shell                                           |
| `FE_SHELL_04` | Render an empty Main Content Area                                                 |
| `FE_SHELL_05` | Provide a Logout action in the Sidebar                                            |
| `FE_SHELL_06` | Prevent unauthenticated access to `/admin`                                        |
| `FE_SHELL_07` | Redirect unauthenticated users to `/login`                                        |
| `FE_SHELL_08` | Logout must revoke the current authentication session through `POST /auth/logout` |
| `FE_SHELL_09` | Logout must clear the client authentication state                                 |
| `FE_SHELL_10` | After logout, navigate to `/login`                                                |

---

# 4. Layout

## Admin Shell

| ID               | Component            | Requirement                                       |
| ---------------- | -------------------- | ------------------------------------------------- |
| `FE_SHELL_UI_01` | Admin Shell          | Provide the protected application shell           |
| `FE_SHELL_UI_02` | Sidebar              | Render the Sidebar within the Admin Shell         |
| `FE_SHELL_UI_03` | Main Content Area    | Render an empty content container                 |
| `FE_SHELL_UI_04` | Application Identity | Display Admin Application identity in the Sidebar |
| `FE_SHELL_UI_05` | Logout Action        | Display the Logout action in the Sidebar          |

The initial Admin Shell contains only:

```text
┌─────────────────────────────────────────────┐
│                                             │
│ Sidebar              Main Content           │
│                                             │
│ Admin Application                           │
│                                             │
│                                             │
│                                             │
│ Logout                                      │
│                                             │
└─────────────────────────────────────────────┘
```

## Sidebar

| ID               | Requirement                                                                                    |
| ---------------- | ---------------------------------------------------------------------------------------------- |
| `FE_SHELL_UI_06` | No feature navigation is required for the initial Admin Shell                                  |
| `FE_SHELL_UI_07` | Future navigation items may be added when their corresponding frontend designs are implemented |

## Main Content

| ID               | Requirement                                                                         |
| ---------------- | ----------------------------------------------------------------------------------- |
| `FE_SHELL_UI_08` | Keep the Main Content Area empty for this implementation                            |
| `FE_SHELL_UI_09` | Provide the Main Content Area as the application content container for future pages |

---

# 5. Authentication Behavior

## Authenticated Access

| ID                 | Condition                         | Behavior             |
| ------------------ | --------------------------------- | -------------------- |
| `FE_SHELL_AUTH_01` | Authenticated user opens `/admin` | Render Admin Shell   |
| `FE_SHELL_AUTH_02` | Authenticated user opens `/login` | Redirect to `/admin` |

## Unauthenticated Access

| ID                 | Condition                                      | Behavior                      |
| ------------------ | ---------------------------------------------- | ----------------------------- |
| `FE_SHELL_AUTH_03` | Unauthenticated user opens `/admin`            | Redirect to `/login`          |
| `FE_SHELL_AUTH_04` | Authentication state is unavailable or invalid | Treat user as unauthenticated |

## Authentication Authority

| ID                 | Requirement                                                                |
| ------------------ | -------------------------------------------------------------------------- |
| `FE_SHELL_AUTH_05` | Use the authentication state established by the Authentication flow        |
| `FE_SHELL_AUTH_06` | Do not determine authentication from user-supplied account IDs             |
| `FE_SHELL_AUTH_07` | Do not determine authentication from user-supplied roles                   |
| `FE_SHELL_AUTH_08` | Do not determine authentication from user-supplied permissions             |
| `FE_SHELL_AUTH_09` | Do not determine authentication from other client-provided identity values |
| `FE_SHELL_AUTH_10` | Treat the backend as the authority for authentication                      |

---

# 6. Logout

Logout is initiated from the Sidebar.

## Logout Flow

| Step | Action                                      |
| ---- | ------------------------------------------- |
| 1    | User selects Logout                         |
| 2    | Frontend sends `POST /auth/logout`          |
| 3    | Frontend clears client authentication state |
| 4    | Frontend navigates to `/login`              |

## Request

| ID                       | Property      | Value                   |
| ------------------------ | ------------- | ----------------------- |
| `FE_SHELL_LOGOUT_API_01` | Method        | `POST`                  |
| `FE_SHELL_LOGOUT_API_02` | Endpoint      | `/auth/logout`          |
| `FE_SHELL_LOGOUT_API_03` | Authorization | `Bearer <access-token>` |

## Success

| ID                       | Property      | Value            |
| ------------------------ | ------------- | ---------------- |
| `FE_SHELL_LOGOUT_API_04` | Status        | `204 No Content` |
| `FE_SHELL_LOGOUT_API_05` | Cache-Control | `no-store`       |

## Unauthorized Logout

| ID                       | Condition                                      | Behavior                                      |
| ------------------------ | ---------------------------------------------- | --------------------------------------------- |
| `FE_SHELL_LOGOUT_API_06` | `POST /auth/logout` returns `401 Unauthorized` | Clear client authentication state             |
| `FE_SHELL_LOGOUT_API_07` | `POST /auth/logout` returns `401 Unauthorized` | Navigate to `/login`                          |
| `FE_SHELL_LOGOUT_API_08` | `401 Unauthorized` is received                 | Treat current authentication state as invalid |

## Logout Requirements

| ID                   | Requirement                                            |
| -------------------- | ------------------------------------------------------ |
| `FE_SHELL_LOGOUT_01` | Prevent duplicate logout requests                      |
| `FE_SHELL_LOGOUT_02` | Disable the Logout action while the request is pending |
| `FE_SHELL_LOGOUT_03` | Do not expose the access token in the UI               |
| `FE_SHELL_LOGOUT_04` | Do not log the access token                            |
| `FE_SHELL_LOGOUT_05` | Clear client authentication state after logout         |
| `FE_SHELL_LOGOUT_06` | Do not remain on `/admin` after logout                 |

---

# 7. UI States

| ID                  | State               | Behavior                                                                           |
| ------------------- | ------------------- | ---------------------------------------------------------------------------------- |
| `FE_SHELL_STATE_01` | Authenticated       | Render Admin Shell                                                                 |
| `FE_SHELL_STATE_02` | Unauthenticated     | Redirect to `/login`                                                               |
| `FE_SHELL_STATE_03` | Logout Pending      | Disable Logout action                                                              |
| `FE_SHELL_STATE_04` | Logout Success      | Clear auth state and navigate to `/login`                                          |
| `FE_SHELL_STATE_05` | Logout Unauthorized | Clear auth state and navigate to `/login`                                          |
| `FE_SHELL_STATE_06` | Logout Failure      | Keep authenticated state and allow retry, unless authentication is no longer valid |
| `FE_SHELL_STATE_07` | Main Content        | Remain empty for this implementation                                               |

---

# 8. Security

The Admin Shell must follow the Authentication and Authorization domain contracts.

| ID                | Security Rule            | Requirement                                                             |
| ----------------- | ------------------------ | ----------------------------------------------------------------------- |
| `FE_SHELL_SEC_01` | Protected Route          | `/admin` must not be accessible without authentication                  |
| `FE_SHELL_SEC_02` | Authentication Rules     | The frontend must not implement its own authentication rules            |
| `FE_SHELL_SEC_03` | Authentication Authority | Treat the backend as the authority for authentication                   |
| `FE_SHELL_SEC_04` | Bearer Credentials       | Send bearer credentials only through the `Authorization` header         |
| `FE_SHELL_SEC_05` | Token URLs               | Never place tokens in URLs                                              |
| `FE_SHELL_SEC_06` | Token Display            | Never render tokens                                                     |
| `FE_SHELL_SEC_07` | Token Logging            | Never log tokens                                                        |
| `FE_SHELL_SEC_08` | Password Handling        | Never store or render passwords in the Admin Shell                      |
| `FE_SHELL_SEC_09` | Authorization            | Do not treat client-side authentication state as proof of authorization |
| `FE_SHELL_SEC_10` | Server Authorization     | Future permission checks must not replace server-side authorization     |

### Backend Security References

| Frontend ID       | Backend Reference             |
| ----------------- | ----------------------------- |
| `FE_SHELL_SEC_01` | `AU_SEC_REQ_01`               |
| `FE_SHELL_SEC_02` | `AU_SEC_REQ_02`               |
| `FE_SHELL_SEC_03` | `AU_SEC_REQ_03`               |
| `FE_SHELL_SEC_04` | `AU_SEC_REQ_04`               |
| `FE_SHELL_SEC_05` | `AU_SEC_REQ_05`               |
| `FE_SHELL_SEC_06` | `AU_SEC_REQ_06`               |
| `FE_SHELL_SEC_07` | `AU_SEC_REQ_07`               |
| `FE_SHELL_SEC_08` | `AU_SEC_REQ_09`               |
| `FE_SHELL_SEC_09` | `AU_SEC_REQ_09`               |
| `FE_SHELL_SEC_10` | Authorization domain contract |

---

# 9. Accessibility

| ID                 | Accessibility Requirement                                   |
| ------------------ | ----------------------------------------------------------- |
| `FE_SHELL_A11Y_01` | Use semantic layout elements                                |
| `FE_SHELL_A11Y_02` | Provide an accessible name for the Sidebar                  |
| `FE_SHELL_A11Y_03` | Provide an accessible name for the Logout action            |
| `FE_SHELL_A11Y_04` | Support keyboard navigation                                 |
| `FE_SHELL_A11Y_05` | Provide visible focus states                                |
| `FE_SHELL_A11Y_06` | Communicate the Logout pending state appropriately          |
| `FE_SHELL_A11Y_07` | Maintain usable layout behavior on supported viewport sizes |

---

# 10. Testing

## Unit

| ID                      | Test                                                 |
| ----------------------- | ---------------------------------------------------- |
| `FE_SHELL_TEST_UNIT_01` | Admin Shell renders correctly                        |
| `FE_SHELL_TEST_UNIT_02` | Sidebar renders correctly                            |
| `FE_SHELL_TEST_UNIT_03` | Logout button state is handled correctly             |
| `FE_SHELL_TEST_UNIT_04` | Authenticated route guard behavior works correctly   |
| `FE_SHELL_TEST_UNIT_05` | Unauthenticated route guard behavior works correctly |

## Integration

| ID                     | Test                                                            |
| ---------------------- | --------------------------------------------------------------- |
| `FE_SHELL_TEST_INT_01` | Authenticated user can access `/admin`                          |
| `FE_SHELL_TEST_INT_02` | Unauthenticated user is redirected to `/login`                  |
| `FE_SHELL_TEST_INT_03` | Authenticated user accessing `/login` is redirected to `/admin` |
| `FE_SHELL_TEST_INT_04` | Logout sends `POST /auth/logout`                                |
| `FE_SHELL_TEST_INT_05` | Logout success clears authentication state                      |
| `FE_SHELL_TEST_INT_06` | Logout success navigates to `/login`                            |
| `FE_SHELL_TEST_INT_07` | Logout `401` clears authentication state                        |
| `FE_SHELL_TEST_INT_08` | Duplicate logout submission is prevented                        |

## E2E

| ID                     | Test                                                            |
| ---------------------- | --------------------------------------------------------------- |
| `FE_SHELL_TEST_E2E_01` | User logs in and the Admin Shell appears                        |
| `FE_SHELL_TEST_E2E_02` | Sidebar appears in the Admin Shell                              |
| `FE_SHELL_TEST_E2E_03` | User logs out and `/login` appears                              |
| `FE_SHELL_TEST_E2E_04` | Unauthenticated user opening `/admin` is redirected to `/login` |

## Manual

| ID                        | Test                                 |
| ------------------------- | ------------------------------------ |
| `FE_SHELL_TEST_MANUAL_01` | Sidebar layout is verified           |
| `FE_SHELL_TEST_MANUAL_02` | Main Content Area is empty           |
| `FE_SHELL_TEST_MANUAL_03` | Login → Admin transition is verified |
| `FE_SHELL_TEST_MANUAL_04` | Logout behavior is verified          |
| `FE_SHELL_TEST_MANUAL_05` | Redirect behavior is verified        |
| `FE_SHELL_TEST_MANUAL_06` | Keyboard interaction is verified     |
| `FE_SHELL_TEST_MANUAL_07` | Responsive layout is verified        |
| `FE_SHELL_TEST_MANUAL_08` | Visible focus states are verified    |

---

# 11. Implementation Criteria

### Status Values

| Status         | Meaning                                            |
| -------------- | -------------------------------------------------- |
| ⚪ Not Started  | Criteria has not been implemented or verified     |
| 🟡 In Progress | Implementation is in progress                      |
| 🟢 Implemented | Implementation is complete and verified            |
| 🔴 Blocked     | Implementation cannot proceed because of a blocker |

## Routes

| ID                  | Criteria                                                            | Status        | Reason |
| ------------------- | ------------------------------------------------------------------- | ------------- | ------ |
| `FE_SHELL_ROUTE_01` | Authenticated users can access `/admin` and see the Admin Shell     | ⚪ Not Started |        |
| `FE_SHELL_ROUTE_02` | Unauthenticated users accessing `/admin` are redirected to `/login` | ⚪ Not Started |        |
| `FE_SHELL_ROUTE_03` | Authenticated users accessing `/login` are redirected to `/admin`   | ⚪ Not Started |        |

## Requirements

| ID            | Criteria                                                              | Status        | Reason |
| ------------- | --------------------------------------------------------------------- | ------------- | ------ |
| `FE_SHELL_01` | Provide a protected `/admin` route                                    | ⚪ Not Started |        |
| `FE_SHELL_02` | Render the Admin Shell after successful authentication                | ⚪ Not Started |        |
| `FE_SHELL_03` | Render a Sidebar within the Admin Shell                               | ⚪ Not Started |        |
| `FE_SHELL_04` | Render an empty Main Content Area                                     | ⚪ Not Started |        |
| `FE_SHELL_05` | Provide a Logout action in the Sidebar                                | ⚪ Not Started |        |
| `FE_SHELL_06` | Prevent unauthenticated access to `/admin`                            | ⚪ Not Started |        |
| `FE_SHELL_07` | Redirect unauthenticated users to `/login`                            | ⚪ Not Started |        |
| `FE_SHELL_08` | Revoke the current authentication session through `POST /auth/logout` | ⚪ Not Started |        |
| `FE_SHELL_09` | Clear the client authentication state after logout                    | ⚪ Not Started |        |
| `FE_SHELL_10` | Navigate to `/login` after logout                                     | ⚪ Not Started |        |

## Layout

| ID               | Criteria                                                            | Status        | Reason |
| ---------------- | ------------------------------------------------------------------- | ------------- | ------ |
| `FE_SHELL_UI_01` | Admin Shell is implemented                                          | ⚪ Not Started |        |
| `FE_SHELL_UI_02` | Sidebar is rendered within the Admin Shell                          | ⚪ Not Started |        |
| `FE_SHELL_UI_03` | Empty Main Content Area is rendered                                 | ⚪ Not Started |        |
| `FE_SHELL_UI_04` | Admin Application identity is displayed                             | ⚪ Not Started |        |
| `FE_SHELL_UI_05` | Logout action is displayed in the Sidebar                           | ⚪ Not Started |        |
| `FE_SHELL_UI_06` | No feature navigation is included in the initial Admin Shell        | ⚪ Not Started |        |
| `FE_SHELL_UI_07` | Future navigation is only added with corresponding frontend designs | ⚪ Not Started |        |
| `FE_SHELL_UI_08` | Main Content Area remains empty                                     | ⚪ Not Started |        |
| `FE_SHELL_UI_09` | Main Content Area provides the future page content container        | ⚪ Not Started |        |

## Authentication Behavior

| ID                 | Criteria                                                                       | Status        | Reason |
| ------------------ | ------------------------------------------------------------------------------ | ------------- | ------ |
| `FE_SHELL_AUTH_01` | Authenticated user opening `/admin` sees the Admin Shell                       | ⚪ Not Started |        |
| `FE_SHELL_AUTH_02` | Authenticated user opening `/login` is redirected to `/admin`                  | ⚪ Not Started |        |
| `FE_SHELL_AUTH_03` | Unauthenticated user opening `/admin` is redirected to `/login`                | ⚪ Not Started |        |
| `FE_SHELL_AUTH_04` | Unavailable or invalid authentication state is treated as unauthenticated      | ⚪ Not Started |        |
| `FE_SHELL_AUTH_05` | Authentication state comes from the Authentication flow                        | ⚪ Not Started |        |
| `FE_SHELL_AUTH_06` | Client-supplied account IDs are not used to determine authentication           | ⚪ Not Started |        |
| `FE_SHELL_AUTH_07` | Client-supplied roles are not used to determine authentication                 | ⚪ Not Started |        |
| `FE_SHELL_AUTH_08` | Client-supplied permissions are not used to determine authentication           | ⚪ Not Started |        |
| `FE_SHELL_AUTH_09` | Other client-provided identity values are not used to determine authentication | ⚪ Not Started |        |
| `FE_SHELL_AUTH_10` | Backend remains the authentication authority                                   | ⚪ Not Started |        |

## Logout API

| ID                       | Criteria                                                               | Status        | Reason |
| ------------------------ | ---------------------------------------------------------------------- | ------------- | ------ |
| `FE_SHELL_LOGOUT_API_01` | Logout uses `POST`                                                     | ⚪ Not Started |        |
| `FE_SHELL_LOGOUT_API_02` | Logout uses `/auth/logout`                                             | ⚪ Not Started |        |
| `FE_SHELL_LOGOUT_API_03` | Logout sends the bearer credential through the `Authorization` header  | ⚪ Not Started |        |
| `FE_SHELL_LOGOUT_API_04` | Successful logout returns `204 No Content`                             | ⚪ Not Started |        |
| `FE_SHELL_LOGOUT_API_05` | Successful logout uses `Cache-Control: no-store`                       | ⚪ Not Started |        |
| `FE_SHELL_LOGOUT_API_06` | `401 Unauthorized` clears client authentication state                  | ⚪ Not Started |        |
| `FE_SHELL_LOGOUT_API_07` | `401 Unauthorized` navigates to `/login`                               | ⚪ Not Started |        |
| `FE_SHELL_LOGOUT_API_08` | `401 Unauthorized` invalidates the current client authentication state | ⚪ Not Started |        |

## Logout

| ID                   | Criteria                                               | Status        | Reason |
| -------------------- | ------------------------------------------------------ | ------------- | ------ |
| `FE_SHELL_LOGOUT_01` | Duplicate logout requests are prevented                | ⚪ Not Started |        |
| `FE_SHELL_LOGOUT_02` | Logout action is disabled while the request is pending | ⚪ Not Started |        |
| `FE_SHELL_LOGOUT_03` | Access token is never exposed in the UI                | ⚪ Not Started |        |
| `FE_SHELL_LOGOUT_04` | Access token is never logged                           | ⚪ Not Started |        |
| `FE_SHELL_LOGOUT_05` | Client authentication state is cleared after logout    | ⚪ Not Started |        |
| `FE_SHELL_LOGOUT_06` | User does not remain on `/admin` after logout          | ⚪ Not Started |        |

## UI States

| ID                  | Criteria                                                                                           | Status        | Reason |
| ------------------- | -------------------------------------------------------------------------------------------------- | ------------- | ------ |
| `FE_SHELL_STATE_01` | Authenticated state renders the Admin Shell                                                        | ⚪ Not Started |        |
| `FE_SHELL_STATE_02` | Unauthenticated state redirects to `/login`                                                        | ⚪ Not Started |        |
| `FE_SHELL_STATE_03` | Logout Pending disables the Logout action                                                          | ⚪ Not Started |        |
| `FE_SHELL_STATE_04` | Logout Success clears auth state and navigates to `/login`                                         | ⚪ Not Started |        |
| `FE_SHELL_STATE_05` | Logout Unauthorized clears auth state and navigates to `/login`                                    | ⚪ Not Started |        |
| `FE_SHELL_STATE_06` | Logout Failure keeps authenticated state and allows retry unless authentication is no longer valid | ⚪ Not Started |        |
| `FE_SHELL_STATE_07` | Main Content Area remains empty                                                                    | ⚪ Not Started |        |

## Security

| ID                | Criteria                                                                  | Status        | Reason |
| ----------------- | ------------------------------------------------------------------------- | ------------- | ------ |
| `FE_SHELL_SEC_01` | `/admin` is inaccessible without authentication                           | ⚪ Not Started |        |
| `FE_SHELL_SEC_02` | Frontend does not implement independent authentication rules              | ⚪ Not Started |        |
| `FE_SHELL_SEC_03` | Backend is treated as the authentication authority                        | ⚪ Not Started |        |
| `FE_SHELL_SEC_04` | Bearer credentials are sent only through the `Authorization` header       | ⚪ Not Started |        |
| `FE_SHELL_SEC_05` | Tokens are never placed in URLs                                           | ⚪ Not Started |        |
| `FE_SHELL_SEC_06` | Tokens are never rendered                                                 | ⚪ Not Started |        |
| `FE_SHELL_SEC_07` | Tokens are never logged                                                   | ⚪ Not Started |        |
| `FE_SHELL_SEC_08` | Passwords are never stored or rendered by the Admin Shell                 | ⚪ Not Started |        |
| `FE_SHELL_SEC_09` | Client-side authentication state is not treated as proof of authorization | ⚪ Not Started |        |
| `FE_SHELL_SEC_10` | Future permission checks do not replace server-side authorization         | ⚪ Not Started |        |

## Accessibility

| ID                 | Criteria                                           | Status        | Reason |
| ------------------ | -------------------------------------------------- | ------------- | ------ |
| `FE_SHELL_A11Y_01` | Semantic layout elements are used                  | ⚪ Not Started |        |
| `FE_SHELL_A11Y_02` | Sidebar has an accessible name                     | ⚪ Not Started |        |
| `FE_SHELL_A11Y_03` | Logout action has an accessible name               | ⚪ Not Started |        |
| `FE_SHELL_A11Y_04` | Keyboard navigation is supported                   | ⚪ Not Started |        |
| `FE_SHELL_A11Y_05` | Visible focus states are provided                  | ⚪ Not Started |        |
| `FE_SHELL_A11Y_06` | Logout pending state is communicated appropriately | ⚪ Not Started |        |
| `FE_SHELL_A11Y_07` | Layout remains usable on supported viewport sizes  | ⚪ Not Started |        |

## Testing — Unit

| ID                      | Criteria                                       | Status        | Reason |
| ----------------------- | ---------------------------------------------- | ------------- | ------ |
| `FE_SHELL_TEST_UNIT_01` | Admin Shell rendering is tested                | ⚪ Not Started |        |
| `FE_SHELL_TEST_UNIT_02` | Sidebar rendering is tested                    | ⚪ Not Started |        |
| `FE_SHELL_TEST_UNIT_03` | Logout button state is tested                  | ⚪ Not Started |        |
| `FE_SHELL_TEST_UNIT_04` | Authenticated route guard behavior is tested   | ⚪ Not Started |        |
| `FE_SHELL_TEST_UNIT_05` | Unauthenticated route guard behavior is tested | ⚪ Not Started |        |

## Testing — Integration

| ID                     | Criteria                                                        | Status        | Reason |
| ---------------------- | --------------------------------------------------------------- | ------------- | ------ |
| `FE_SHELL_TEST_INT_01` | Authenticated user can access `/admin`                          | ⚪ Not Started |        |
| `FE_SHELL_TEST_INT_02` | Unauthenticated user is redirected to `/login`                  | ⚪ Not Started |        |
| `FE_SHELL_TEST_INT_03` | Authenticated user accessing `/login` is redirected to `/admin` | ⚪ Not Started |        |
| `FE_SHELL_TEST_INT_04` | Logout sends `POST /auth/logout`                                | ⚪ Not Started |        |
| `FE_SHELL_TEST_INT_05` | Logout success clears authentication state                      | ⚪ Not Started |        |
| `FE_SHELL_TEST_INT_06` | Logout success navigates to `/login`                            | ⚪ Not Started |        |
| `FE_SHELL_TEST_INT_07` | Logout `401` clears authentication state                        | ⚪ Not Started |        |
| `FE_SHELL_TEST_INT_08` | Duplicate logout submission is prevented                        | ⚪ Not Started |        |

## Testing — E2E

| ID                     | Criteria                                                 | Status        | Reason |
| ---------------------- | -------------------------------------------------------- | ------------- | ------ |
| `FE_SHELL_TEST_E2E_01` | Login leads to the Admin Shell                           | ⚪ Not Started |        |
| `FE_SHELL_TEST_E2E_02` | Sidebar appears in the Admin Shell                       | ⚪ Not Started |        |
| `FE_SHELL_TEST_E2E_03` | Logout leads to `/login`                                 | ⚪ Not Started |        |
| `FE_SHELL_TEST_E2E_04` | Unauthenticated access to `/admin` redirects to `/login` | ⚪ Not Started |        |

## Testing — Manual

| ID                        | Criteria                             | Status        | Reason |
| ------------------------- | ------------------------------------ | ------------- | ------ |
| `FE_SHELL_TEST_MANUAL_01` | Sidebar layout is verified           | ⚪ Not Started |        |
| `FE_SHELL_TEST_MANUAL_02` | Main Content Area is empty           | ⚪ Not Started |        |
| `FE_SHELL_TEST_MANUAL_03` | Login → Admin transition is verified | ⚪ Not Started |        |
| `FE_SHELL_TEST_MANUAL_04` | Logout behavior is verified          | ⚪ Not Started |        |
| `FE_SHELL_TEST_MANUAL_05` | Redirect behavior is verified        | ⚪ Not Started |        |
| `FE_SHELL_TEST_MANUAL_06` | Keyboard interaction is verified     | ⚪ Not Started |        |
| `FE_SHELL_TEST_MANUAL_07` | Responsive layout is verified        | ⚪ Not Started |        |
| `FE_SHELL_TEST_MANUAL_08` | Visible focus states are verified    | ⚪ Not Started |        |
