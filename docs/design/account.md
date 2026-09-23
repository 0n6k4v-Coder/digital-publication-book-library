# Account Domain Ready For Implement Design

## Table of Contents

1. [Requirements](#1-requirements)

   * [1.1 Account](#11-account)
   * [1.2 Account Credentials](#12-account-credentials)
   * [1.3 Account Lifecycle](#13-account-lifecycle)
   * [1.4 Non-Functional Requirements](#14-non-functional-requirements)
2. [Security](#2-security)

   * [2.1 Security Requirements](#21-security-requirements)

     * [2.1.1 Functional Requirements](#211-functional-requirements)
     * [2.1.2 Non-Functional Requirements](#212-non-functional-requirements)
   * [2.2 Security Decisions](#22-security-decisions)

     * [2.2.1 Email](#221-email)
     * [2.2.2 Password](#222-password)
3. [Design Decisions](#3-design-decisions)

   * [3.1 Account](#31-account)
   * [3.2 Account Credentials](#32-account-credentials)
   * [3.3 Account Lifecycle](#33-account-lifecycle)
4. [Data Model](#4-data-model)

   * [4.1 `account`](#41-account)
   * [4.2 `account_credentials`](#42-account_credentials)

---

# 1. Requirements

## 1.1 Account

| ID             | Requirement                                                                                         |
| -------------- | --------------------------------------------------------------------------------------------------- |
| `AC_REQ_FC_01` | The system must create an account.                                                                  |
| `AC_REQ_FC_02` | The system must assign a unique identifier to each account.                                         |
| `AC_REQ_FC_03` | The system must record the account creation timestamp.                                              |
| `AC_REQ_FC_04` | The system must record the latest Account entity update timestamp.                                  |
| `AC_REQ_FC_05` | The system must support `active` and `inactive` statuses.                                           |
| `AC_REQ_FC_06` | The system must allow Account fields to be updated.                                                 |
| `AC_REQ_FC_07` | The system must record the account responsible for creating or updating an account when applicable. |

## 1.2 Account Credentials

| ID             | Requirement                                                         |
| -------------- | ------------------------------------------------------------------- |
| `AC_REQ_FC_08` | Each account must have exactly one credential set.                  |
| `AC_REQ_FC_09` | The current authentication method must be email and password.       |
| `AC_REQ_FC_10` | The system must use email as the authentication identifier.         |
| `AC_REQ_FC_11` | The system must enforce unique email identity.                      |
| `AC_REQ_FC_12` | The system must allow email and password credentials to be updated. |

## 1.3 Account Lifecycle

| ID             | Requirement                                                                       |
| -------------- | --------------------------------------------------------------------------------- |
| `AC_REQ_FC_13` | The system must support soft deletion of an account.                              |
| `AC_REQ_FC_14` | The system must record the soft-deletion timestamp.                               |
| `AC_REQ_FC_15` | The system must record the account responsible for soft deletion when applicable. |
| `AC_REQ_FC_16` | The system must prevent authentication for inactive accounts.                     |
| `AC_REQ_FC_17` | The system must prevent authentication for soft-deleted accounts.                 |
| `AC_REQ_FC_18` | The system must prevent deactivation of the last active administrator account.    |
| `AC_REQ_FC_19` | The system must support explicit hard deletion when permitted.                    |
| `AC_REQ_FC_20` | The system must support restoration of a soft-deleted account.                    |

## 1.4 Non-Functional Requirements

| ID                 | Requirement                                                            |
| ------------------ | ---------------------------------------------------------------------- |
| `AC_REQ_NON_FC_01` | Account lifecycle operations must preserve defined Account invariants. |
| `AC_REQ_NON_FC_02` | Account timestamps must use `TIMESTAMPTZ`.                             |
| `AC_REQ_NON_FC_03` | Account lifecycle data must remain separate from credential data.      |
| `AC_REQ_NON_FC_04` | Soft-deleted accounts must be excluded from normal Account operations. |
| `AC_REQ_NON_FC_05` | Hard deletion must be an explicit operation.                           |

# 2. Security

## 2.1 Security Requirements

### 2.1.1 Functional Requirements

| ID                 | Requirement                                                                    |
| ------------------ | ------------------------------------------------------------------------------ |
| `AC_SEC_REQ_FC_01` | The system must authenticate using email and password.                         |
| `AC_SEC_REQ_FC_02` | The system must verify the supplied password against the stored password hash. |
| `AC_SEC_REQ_FC_03` | The system must allow an email address to be changed.                          |
| `AC_SEC_REQ_FC_04` | The system must allow a password to be changed.                                |

### 2.1.2 Non-Functional Requirements

| ID                     | Requirement                                                                                             |
| ---------------------- | ------------------------------------------------------------------------------------------------------- |
| `AC_SEC_REQ_NON_FC_01` | Passwords must never be stored in plaintext.                                                            |
| `AC_SEC_REQ_NON_FC_02` | Passwords must be hashed using Argon2id.                                                                |
| `AC_SEC_REQ_NON_FC_03` | Each password must use a unique salt.                                                                   |
| `AC_SEC_REQ_NON_FC_04` | Passwords must have a minimum length of 15 characters.                                                  |
| `AC_SEC_REQ_NON_FC_05` | The system must support passwords of at least 64 characters.                                            |
| `AC_SEC_REQ_NON_FC_06` | The password policy must not require arbitrary character composition rules.                             |
| `AC_SEC_REQ_NON_FC_07` | The system must reject commonly used or compromised passwords.                                          |
| `AC_SEC_REQ_NON_FC_08` | Authentication credentials must not be exposed through normal responses, logs, or administrative views. |

## 2.2 Security Decisions

### 2.2.1 Email

| ID                    | Decision                        | Definition                                                          |
| --------------------- | ------------------------------- | ------------------------------------------------------------------- |
| `AC_SEC_DEC_EMAIL_01` | Email identity                  | Email identity is case-insensitive.                                 |
| `AC_SEC_DEC_EMAIL_02` | Email storage                   | `email` stores the canonical application email value.               |
| `AC_SEC_DEC_EMAIL_03` | Email normalization             | Trim surrounding whitespace and lowercase the domain.               |
| `AC_SEC_DEC_EMAIL_04` | Local-part handling             | Preserve local-part case.                                           |
| `AC_SEC_DEC_EMAIL_05` | Provider-specific normalization | Do not remove `+` tags or modify provider-specific dot conventions. |
| `AC_SEC_DEC_EMAIL_06` | Email uniqueness                | Enforce case-insensitive uniqueness in PostgreSQL.                  |

### 2.2.2 Password

| ID                       | Decision         | Definition                                                           |
| ------------------------ | ---------------- | -------------------------------------------------------------------- |
| `AC_SEC_DEC_PASSWORD_01` | Password storage | Store only the password hash.                                        |
| `AC_SEC_DEC_PASSWORD_02` | Hashing          | Use Argon2id.                                                        |
| `AC_SEC_DEC_PASSWORD_03` | Salt             | Use a unique salt for every password.                                |
| `AC_SEC_DEC_PASSWORD_04` | Minimum length   | Passwords must contain at least 15 characters.                       |
| `AC_SEC_DEC_PASSWORD_05` | Maximum length   | Support passwords of at least 64 characters.                         |
| `AC_SEC_DEC_PASSWORD_06` | Composition      | Do not require uppercase, lowercase, number, or symbol combinations. |
| `AC_SEC_DEC_PASSWORD_07` | Blocklist        | Reject commonly used or compromised passwords.                       |
| `AC_SEC_DEC_PASSWORD_08` | Plaintext        | Never persist plaintext passwords.                                   |

# 3. Design Decisions

## 3.1 Account

| ID                  | Decision           | Definition                                         |
| ------------------- | ------------------ | -------------------------------------------------- |
| `AC_DEC_ACCOUNT_01` | Identifier         | `account.id` uses PostgreSQL `uuid`.               |
| `AC_DEC_ACCOUNT_02` | ID generation      | Generate IDs with PostgreSQL native `uuidv7()`.    |
| `AC_DEC_ACCOUNT_03` | Creation timestamp | `created_at TIMESTAMPTZ NOT NULL`.                 |
| `AC_DEC_ACCOUNT_04` | Update timestamp   | `updated_at TIMESTAMPTZ NOT NULL`.                 |
| `AC_DEC_ACCOUNT_05` | Deletion timestamp | `deleted_at TIMESTAMPTZ NULL`.                     |
| `AC_DEC_ACCOUNT_06` | Time zone          | Store timestamps in UTC.                           |
| `AC_DEC_ACCOUNT_07` | Status             | Only `active` and `inactive`.                      |
| `AC_DEC_ACCOUNT_08` | `created_by`       | Nullable FK to `account.id`, `ON DELETE SET NULL`. |
| `AC_DEC_ACCOUNT_09` | `updated_by`       | Nullable FK to `account.id`, `ON DELETE SET NULL`. |
| `AC_DEC_ACCOUNT_10` | `deleted_by`       | Nullable FK to `account.id`, `ON DELETE SET NULL`. |

## 3.2 Account Credentials

| ID                     | Decision              | Definition                                                         |
| ---------------------- | --------------------- | ------------------------------------------------------------------ |
| `AC_DEC_CREDENTIAL_01` | Relationship          | One account has exactly one credential set.                        |
| `AC_DEC_CREDENTIAL_02` | Authentication        | Email and password only.                                           |
| `AC_DEC_CREDENTIAL_03` | Ownership             | `account_credentials.account_id` references `account.id`.          |
| `AC_DEC_CREDENTIAL_04` | Deletion              | `account_credentials.account_id` uses `ON DELETE CASCADE`.         |
| `AC_DEC_CREDENTIAL_05` | Credential timestamps | Email or password changes update `account_credentials.updated_at`. |
| `AC_DEC_CREDENTIAL_06` | Account timestamp     | Credential changes do not update `account.updated_at`.             |

## 3.3 Account Lifecycle

### Account States

| State        | `status`   | `deleted_at` |
| ------------ | ---------- | ------------ |
| Active       | `active`   | `NULL`       |
| Inactive     | `inactive` | `NULL`       |
| Soft deleted | `inactive` | Not `NULL`   |

### Lifecycle

```text
ACTIVE
  │
  ├── deactivate ──► INACTIVE
  │
  └── soft delete ─► SOFT DELETED

INACTIVE
  │
  ├── activate ────► ACTIVE
  │
  └── soft delete ─► SOFT DELETED

SOFT DELETED
  │
  ├── restore ─────► INACTIVE
  │
  └── purge ───────► PHYSICALLY DELETED
```

### Lifecycle Rules

| ID                    | Rule                                                                                                       |
| --------------------- | ---------------------------------------------------------------------------------------------------------- |
| `AC_DEC_LIFECYCLE_01` | A soft-deleted account must have `status = inactive`.                                                      |
| `AC_DEC_LIFECYCLE_02` | A soft-deleted account must not authenticate.                                                              |
| `AC_DEC_LIFECYCLE_03` | Restoring a soft-deleted account changes its status to `inactive`.                                         |
| `AC_DEC_LIFECYCLE_04` | Hard deletion physically removes the account record.                                                       |
| `AC_DEC_LIFECYCLE_05` | Hard deletion is an explicit operation.                                                                    |
| `AC_DEC_LIFECYCLE_06` | Hard deletion never occurs as a side effect of normal updates.                                             |
| `AC_DEC_LIFECYCLE_07` | At least one active, non-deleted administrator must always remain.                                         |
| `AC_DEC_LIFECYCLE_08` | Last-administrator protection must be enforced transactionally.                                            |
| `AC_DEC_LIFECYCLE_09` | For the first administrator, `created_by`, `updated_by`, and `deleted_by` are `NULL` when no actor exists. |

### Account Update Rules

| Operation             | `account.updated_at` | `account_credentials.updated_at` |
| --------------------- | -------------------: | -------------------------------: |
| Update Account fields |                  Yes |                               No |
| Activate              |                  Yes |                               No |
| Deactivate            |                  Yes |                               No |
| Soft delete           |                  Yes |                               No |
| Restore               |                  Yes |                               No |
| Change email          |                   No |                              Yes |
| Change password       |                   No |                              Yes |

# 4. Data Model

## 4.1 `account`

**ID:** `AC_DM_01`

| Column       | Type          | Null | Constraint                              |
| ------------ | ------------- | ---: | --------------------------------------- |
| `id`         | `uuid`        |   No | PK, default `uuidv7()`                  |
| `created_at` | `timestamptz` |   No |                                         |
| `created_by` | `uuid`        |  Yes | FK → `account.id`, `ON DELETE SET NULL` |
| `updated_at` | `timestamptz` |   No |                                         |
| `updated_by` | `uuid`        |  Yes | FK → `account.id`, `ON DELETE SET NULL` |
| `status`     | `text`        |   No | `active` or `inactive`                  |
| `deleted_at` | `timestamptz` |  Yes |                                         |
| `deleted_by` | `uuid`        |  Yes | FK → `account.id`, `ON DELETE SET NULL` |

## 4.2 `account_credentials`

**ID:** `AC_DM_02`

| Column             | Type          | Null | Constraint                                  |
| ------------------ | ------------- | ---: | ------------------------------------------- |
| `account_id`       | `uuid`        |   No | PK + FK → `account.id`, `ON DELETE CASCADE` |
| `email`            | `text`        |   No |                                             |
| `email_normalized` | `text`        |   No | Unique                                      |
| `password_hash`    | `text`        |   No | Argon2id hash                               |
| `created_at`       | `timestamptz` |   No |                                             |
| `updated_at`       | `timestamptz` |   No |                                             |

### Relationship

```text
account
   1
   │
   │
   1
   ▼
account_credentials
```

`account_credentials.account_id` is both the primary key and foreign key.

**One Account → Exactly One Credential Set**
