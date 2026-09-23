CREATE TABLE account (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by UUID NULL REFERENCES account (id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_by UUID NULL REFERENCES account (id) ON DELETE SET NULL,
    status TEXT NOT NULL DEFAULT 'active',
    deleted_at TIMESTAMPTZ NULL,
    deleted_by UUID NULL REFERENCES account (id) ON DELETE SET NULL,
    CONSTRAINT account_status_check CHECK (status IN ('active', 'inactive')),
    CONSTRAINT account_deleted_state_check CHECK (
        (deleted_at IS NULL AND status IN ('active', 'inactive'))
        OR (deleted_at IS NOT NULL AND status = 'inactive')
    )
);

CREATE TABLE account_credentials (
    account_id UUID PRIMARY KEY REFERENCES account (id) ON DELETE CASCADE,
    email TEXT NOT NULL,
    email_normalized TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT account_credentials_email_normalized_key UNIQUE (email_normalized)
);

CREATE INDEX account_active_status_idx
    ON account (status)
    WHERE deleted_at IS NULL;