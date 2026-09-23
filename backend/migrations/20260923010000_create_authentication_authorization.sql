CREATE TABLE authentication_session (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    account_id UUID NOT NULL REFERENCES account (id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    last_authenticated_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ NULL,
    revocation_reason TEXT NULL
);

CREATE INDEX authentication_session_account_idx
    ON authentication_session (account_id);

CREATE TABLE authentication_access_token (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    session_id UUID NOT NULL REFERENCES authentication_session (id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT authentication_access_token_expiry_check CHECK (expires_at > created_at)
);

CREATE TABLE authorization_role (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    disabled_at TIMESTAMPTZ NULL
);

CREATE TABLE authorization_permission (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE authorization_role_permission (
    role_id UUID NOT NULL REFERENCES authorization_role (id),
    permission_id UUID NOT NULL REFERENCES authorization_permission (id),
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE authorization_account_role (
    account_id UUID NOT NULL REFERENCES account (id),
    role_id UUID NOT NULL REFERENCES authorization_role (id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by UUID NULL REFERENCES account (id),
    PRIMARY KEY (account_id, role_id)
);

INSERT INTO authorization_role (name, description)
VALUES
    ('account_admin', 'Manage administrator accounts.'),
    ('account_viewer', 'View administrator accounts.');

INSERT INTO authorization_permission (name, description)
VALUES
    ('account:view', 'View non-deleted administrator accounts.'),
    ('account:view_deleted', 'View soft-deleted administrator accounts.'),
    ('account:create', 'Create administrator accounts.'),
    ('account:update', 'Update administrator account fields.'),
    ('account:deactivate', 'Deactivate administrator accounts.'),
    ('account:activate', 'Activate administrator accounts.'),
    ('account:delete', 'Soft-delete administrator accounts.'),
    ('account:restore', 'Restore soft-deleted administrator accounts.'),
    ('account:purge', 'Hard-delete administrator accounts.'),
    ('account:change_email', 'Change administrator email credentials.'),
    ('account:change_password', 'Change administrator password credentials.'),
    ('authorization:role_assign', 'Assign a role to an account.'),
    ('authorization:role_revoke', 'Revoke a role from an account.');

INSERT INTO authorization_role_permission (role_id, permission_id)
SELECT role.id, permission.id
FROM authorization_role AS role
INNER JOIN authorization_permission AS permission
    ON permission.name IN (
        'account:view',
        'account:view_deleted',
        'account:create',
        'account:update',
        'account:deactivate',
        'account:activate',
        'account:delete',
        'account:restore',
        'account:purge',
        'account:change_email',
        'account:change_password'
    )
WHERE role.name = 'account_admin';

INSERT INTO authorization_role_permission (role_id, permission_id)
SELECT role.id, permission.id
FROM authorization_role AS role
INNER JOIN authorization_permission AS permission
    ON permission.name IN ('account:view', 'account:view_deleted')
WHERE role.name = 'account_viewer';