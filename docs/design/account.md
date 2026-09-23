# Account Domain Design

## Table of Contents

1. Requirements
   - 1.1 Functional Requirements
   - 1.2 Non-Functional Requirements
2. Data Model
   - 2.1 `account`
   - 2.2 `account_credentials`

# 1. Requirements

## 1.1 Functional Requirements

| ID | Requirement |
|---|---|
| `AC_REQ_FC_01` | The system must create an account. |
| `AC_REQ_FC_02` | The system must assign a unique identifier to each account. |
| `AC_REQ_FC_03` | The system must store the account creation timestamp. |
| `AC_REQ_FC_04` | The system must store the timestamp of the latest account update. |
| `AC_REQ_FC_05` | The system must support `active` and `inactive` account statuses. |
| `AC_REQ_FC_06` | The system must support soft deletion of an account. |
| `AC_REQ_FC_07` | The system must record the timestamp of soft deletion. |
| `AC_REQ_FC_08` | The system must record the account responsible for creating, updating, or soft-deleting an account when applicable. |
| `AC_REQ_FC_09` | The system must prevent authentication for inactive accounts. |
| `AC_REQ_FC_10` | The system must prevent authentication for soft-deleted accounts. |
| `AC_REQ_FC_11` | The system must prevent deactivation of the last active administrator account. |
| `AC_REQ_FC_12` | The system must support hard deletion of an account as an explicit operation when permitted. |
| `AC_REQ_FC_13` | Each account must have one authentication credential set. |
| `AC_REQ_FC_14` | The current authentication method must be email and password. |
| `AC_REQ_FC_15` | The system must use email as the authentication identifier. |
| `AC_REQ_FC_16` | The system must enforce email uniqueness for accounts that are retained in the system. |
| `AC_REQ_FC_17` | The system must store a password verifier rather than a plaintext password. |
| `AC_REQ_FC_18` | The system must allow account credentials to be updated. |
| `AC_REQ_FC_19` | Each credential set must belong to exactly one account. |
| `AC_REQ_FC_20` | The system must reject authentication when the supplied credentials are invalid. |
| `AC_REQ_FC_21` | The system must create an authenticated session only after successful authentication and account eligibility checks. |

## 1.2 Non-Functional Requirements

| ID | Requirement |
|---|---|
| `AC_REQ_NON_FC_01` | Passwords must never be stored in plaintext. |
| `AC_REQ_NON_FC_02` | Passwords must be stored using a password hashing algorithm designed for password storage, such as Argon2id. |
| `AC_REQ_NON_FC_03` | The password hashing implementation must use a unique salt for each password. |
| `AC_REQ_NON_FC_04` | A single-factor password must have a minimum length of 15 characters. |
| `AC_REQ_NON_FC_05` | The password policy must not require arbitrary character composition rules such as uppercase, lowercase, number, or symbol combinations. |
| `AC_REQ_NON_FC_06` | The password policy should support passwords of at least 64 characters. |
| `AC_REQ_NON_FC_07` | The system should reject passwords identified as commonly used, expected, or compromised. |
| `AC_REQ_NON_FC_08` | Authentication credentials must not be exposed through normal application responses, logs, or administrative views. |
| `AC_REQ_NON_FC_09` | Account lifecycle operations must preserve defined account invariants. |
| `AC_REQ_NON_FC_10` | `account_credentials.account_id` must reference an existing account. |
| `AC_REQ_NON_FC_11` | Each account must have no more than one credential set. |
| `AC_REQ_NON_FC_12` | Account timestamps must use a consistent time representation. |
| `AC_REQ_NON_FC_13` | Account lifecycle data must remain separate from authentication credential data. |
| `AC_REQ_NON_FC_14` | Authentication must remain separate from authorization. |
| `AC_REQ_NON_FC_15` | Authorization rules must not be stored in the Account domain unless they become a defined domain requirement. |
| `AC_REQ_NON_FC_16` | Account status must represent operational state and must not represent deletion state. |
| `AC_REQ_NON_FC_17` | Soft-deleted accounts must be excluded from normal account operations and authentication. |
| `AC_REQ_NON_FC_18` | Hard deletion must be an explicit operation and must not occur as a side effect of normal account updates. |
| `AC_REQ_NON_FC_19` | Credential storage must be isolated from the protected account entity. |
| `AC_REQ_NON_FC_20` | The current Account domain must not introduce additional authentication methods until they are required. |

# 2. Data Model

## 2.1 `account`

**ID:** `AC_DM_01`

The `account` table represents the protected Account entity.

It stores account identity, lifecycle state, timestamps, and the accounts responsible for account changes.

| Column | Description | Constraint |
|---|---|---|
| `id` | Unique account identifier | Primary key |
| `created_at` | Account creation timestamp | Required |
| `created_by` | Account responsible for account creation | Nullable foreign key to `account.id` |
| `updated_at` | Timestamp of the latest account update | Required |
| `updated_by` | Account responsible for the latest update | Nullable foreign key to `account.id` |
| `status` | Current operational account status | Required |
| `deleted_at` | Account soft-deletion timestamp | Nullable |
| `deleted_by` | Account responsible for soft deletion | Nullable foreign key to `account.id` |

### Account Status

The `status` field represents operational state only.

```
active
inactive
```

Deletion is represented independently through `deleted_at`.

```
Active
status = active
deleted_at = NULL

Inactive
status = inactive
deleted_at = NULL

Soft Deleted
status = inactive
deleted_at != NULL
```

A hard-deleted account has no remaining `account` record.

### Account Lifecycle

```
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

The exact restore operation is an application decision, but a soft-deleted account must not authenticate while it remains deleted.

### Last Active Administrator Rule

The system must always retain at least one active administrator account.

```
active administrator count >= 1
```

Therefore, deactivation of the last active administrator must be rejected.

This invariant must be enforced atomically at the application and persistence boundary so concurrent operations cannot bypass the rule.

## 2.2 `account_credentials`

**ID:** `AC_DM_02`

The `account_credentials` table represents the authentication credentials associated with an Account.

The current version supports only email and password authentication.

| Column | Description | Constraint |
|---|---|---|
| `account_id` | Associated account identifier | Primary key and foreign key |
| `email` | Email address associated with the credential | Required |
| `email_normalized` | Normalized email used for authentication lookup | Required, unique |
| `password_hash` | Password verifier | Required |
| `created_at` | Credential creation timestamp | Required |
| `updated_at` | Timestamp of the latest credential update | Required |

### Relationship

```
account
   1
   │
   │
   1
   ▼
account_credentials
```

`account_credentials.account_id` is both the primary key and foreign key to `account.id`.

This enforces:

```
One Account
    ↓
One Credential Set
```

### Authentication Boundary

The responsibilities are intentionally separated:

```
account
    → protected account entity
    → lifecycle
    → status
    → deletion

account_credentials
    → authentication identity
    → password verification
```

Authentication verifies the credential and account eligibility.

Authorization is a separate concern and determines whether the authenticated principal may perform a protected operation.

### Authentication Flow

```
Email + Password
       │
       ▼
account_credentials
       │
       ├── email lookup
       └── password verification
       │
       ▼
account
       │
       ├── status = active
       └── deleted_at = NULL
       │
       ▼
Authenticated Session
       │
       ▼
Authorization
       │
       ├── allow
       └── deny
```

Session management remains part of the separate Authentication concern rather than the Account domain data model.

### Credential Security

The database must store only the password verifier:

```
Submitted Password
       │
       ▼
Password Hashing
       │
       ▼
password_hash
```

The plaintext password must never be persisted.

For a new implementation, Argon2id is the preferred password hashing approach.

# Domain Boundary

The Account domain contains exactly two tables:

```
Account Domain
│
├── AC_DM_01
│   └── account
│
└── AC_DM_02
    └── account_credentials
```

The architectural responsibilities are:

```
Authentication
    → account_credentials
    → authentication session

Authorization
    → separate policy/enforcement layer

Protected Entity
    → account
```

The current version intentionally does not introduce roles, permissions, multiple credential types, social login, or other authentication mechanisms.
