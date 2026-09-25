CREATE TABLE authorization_role_revocation_audit (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    account_id UUID NOT NULL,
    role_id UUID NOT NULL,
    revoked_by UUID NOT NULL,
    revoked_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO authorization_role_permission (
    role_id,
    permission_id
)
SELECT
    role.id,
    permission.id
FROM authorization_role AS role
INNER JOIN authorization_permission AS permission
    ON permission.name IN (
        'authorization:role_assign',
        'authorization:role_revoke'
    )
WHERE role.name = 'account_admin'
ON CONFLICT (role_id, permission_id) DO NOTHING;