CREATE TABLE authentication_refresh_token (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    session_id UUID NOT NULL
        REFERENCES authentication_session (id)
        ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ NULL,
    revoked_at TIMESTAMPTZ NULL
);

ALTER TABLE authentication_session
DROP CONSTRAINT authentication_session_account_id_fkey;

ALTER TABLE authentication_session
ADD CONSTRAINT authentication_session_account_id_fkey
FOREIGN KEY (account_id)
REFERENCES account (id)
ON DELETE CASCADE;

ALTER TABLE authorization_account_role
DROP CONSTRAINT authorization_account_role_account_id_fkey;

ALTER TABLE authorization_account_role
ADD CONSTRAINT authorization_account_role_account_id_fkey
FOREIGN KEY (account_id)
REFERENCES account (id)
ON DELETE CASCADE;

ALTER TABLE authorization_account_role
DROP CONSTRAINT authorization_account_role_created_by_fkey;

ALTER TABLE authorization_account_role
ADD CONSTRAINT authorization_account_role_created_by_fkey
FOREIGN KEY (created_by)
REFERENCES account (id)
ON DELETE SET NULL;

CREATE TABLE account_administrator_invariant_lock (
    id SMALLINT PRIMARY KEY,
    CONSTRAINT account_administrator_invariant_lock_singleton
        CHECK (id = 1)
);

INSERT INTO account_administrator_invariant_lock (id)
VALUES (1);