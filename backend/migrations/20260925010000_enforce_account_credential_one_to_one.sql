DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM account AS a
        LEFT JOIN account_credentials AS ac
            ON ac.account_id = a.id
        GROUP BY a.id
        HAVING COUNT(ac.account_id) <> 1
    ) THEN
        RAISE EXCEPTION
            'account credential invariant failed: every account must have exactly one credential set';
    END IF;
END;
$$;

CREATE OR REPLACE FUNCTION enforce_account_exactly_one_credential_set()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
    target_account_id UUID;
    credential_count BIGINT;
BEGIN
    IF TG_OP = 'DELETE' THEN
        target_account_id := OLD.account_id;
    ELSE
        target_account_id := NEW.account_id;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM account
        WHERE id = target_account_id
    ) THEN
        SELECT COUNT(*)
        INTO credential_count
        FROM account_credentials
        WHERE account_id = target_account_id;

        IF credential_count <> 1 THEN
            RAISE EXCEPTION
                'account credential invariant failed: every account must have exactly one credential set';
        END IF;
    END IF;

    IF TG_OP = 'UPDATE'
        AND NEW.account_id IS DISTINCT FROM OLD.account_id
        AND EXISTS (
            SELECT 1
            FROM account
            WHERE id = OLD.account_id
        )
    THEN
        SELECT COUNT(*)
        INTO credential_count
        FROM account_credentials
        WHERE account_id = OLD.account_id;

        IF credential_count <> 1 THEN
            RAISE EXCEPTION
                'account credential invariant failed: every account must have exactly one credential set';
        END IF;
    END IF;

    RETURN NULL;
END;
$$;

CREATE CONSTRAINT TRIGGER account_exactly_one_credential_after_insert
AFTER INSERT ON account
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW
EXECUTE FUNCTION enforce_account_exactly_one_credential_set();

CREATE CONSTRAINT TRIGGER account_credentials_exactly_one_after_change
AFTER INSERT OR DELETE OR UPDATE OF account_id ON account_credentials
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW
EXECUTE FUNCTION enforce_account_exactly_one_credential_set();