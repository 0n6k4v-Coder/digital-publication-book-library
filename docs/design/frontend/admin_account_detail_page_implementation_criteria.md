# Admin Account Detail Page — Implementation Criteria

This document is the implementation-review checklist for **Section 15 — Implementation Criteria** of `admin_account_detail_page.md`.

**Every item in every Section 15 subsection is represented in its own table.** `Status` and `Reason` are intentionally blank.

## 15.1 Route

| ID | Criteria | Status | Reason |
|---|---|---|---|
| `FE-ACCOUNT-DETAIL-IMPL-001` | `/admin/accounts/:id/edit` renders inside the Admin Shell. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-002` | Unauthenticated access is handled by Admin Shell authentication behavior. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-003` | Accounts navigation remains active while viewing the detail page. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-004` | Back navigation returns to `/admin/accounts`. |  |  |

## 15.2 Data

| ID | Criteria | Status | Reason |
|---|---|---|---|
| `FE-ACCOUNT-DETAIL-IMPL-005` | Detail data comes from `GET /admin/accounts/{id}`. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-006` | Server Account response is authoritative. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-007` | Password credentials are never expected or rendered. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-008` | Account data is not stored in browser persistence. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-009` | Account response cache policy remains `no-store`. |  |  |

## 15.3 Editing

| ID | Criteria | Status | Reason |
|---|---|---|---|
| `FE-ACCOUNT-DETAIL-IMPL-010` | `display_name` is the only field submitted through the primary Account update form. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-011` | Update uses `PATCH /admin/accounts/{id}`. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-012` | Update uses `application/merge-patch+json`. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-013` | Duplicate submissions are prevented. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-014` | Successful updates replace or refresh authoritative Account state. |  |  |

## 15.4 Lifecycle

| ID | Criteria | Status | Reason |
|---|---|---|---|
| `FE-ACCOUNT-DETAIL-IMPL-015` | Deactivate is available for active, non-deleted Accounts. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-016` | Activate is available for inactive, non-deleted Accounts. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-017` | Restore is available for soft-deleted Accounts returned by the API. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-018` | Hard delete is not exposed. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-019` | Lifecycle actions are confirmed by the server before the UI commits the new state. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-020` | `LAST_ACTIVE_ADMINISTRATOR` is handled as an explicit conflict. |  |  |

## 15.5 Security

| ID | Criteria | Status | Reason |
|---|---|---|---|
| `FE-ACCOUNT-DETAIL-IMPL-021` | Backend authentication remains authoritative. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-022` | Backend authorization remains authoritative. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-023` | Tokens are only sent through the `Authorization` header for protected Account APIs. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-024` | Tokens never appear in URLs. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-025` | Tokens never render in the UI. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-026` | Tokens never appear in logs. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-027` | Passwords and password hashes never render. |  |  |

## 15.6 Accessibility

| ID | Criteria | Status | Reason |
|---|---|---|---|
| `FE-ACCOUNT-DETAIL-IMPL-028` | Semantic page structure is used. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-029` | Primary page heading is present. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-030` | Form controls have visible labels. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-031` | Navigation and actions are keyboard accessible. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-032` | Focus is visible and not obscured. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-033` | Async status is accessible. |  |  |
| `FE-ACCOUNT-DETAIL-IMPL-034` | Account lifecycle status does not rely only on color. |  |  |
