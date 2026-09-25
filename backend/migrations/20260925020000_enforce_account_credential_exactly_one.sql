CREATE FUNCTION enforce_account_exactly_one_credential()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    IF TG_TABLE_NAME = 'account' AND TG_OP = 'INSERT' THEN
        IF NOT EXISTS (
            SELECT 1
            FROM account_credentials
            WHERE account_id = NEW.id
        ) THEN
            RAISE EXCEPTION 'account % must have exactly one credential', NEW.id
                USING
                    ERRCODE = '23514',
                    CONSTRAINT = 'account_exactly_one_credential';
        END IF;

        RETURN NULL;
    END IF;

    IF TG_TABLE_NAME = 'account_credentials' AND TG_OP = 'DELETE' THEN
        IF EXISTS (
            SELECT 1
            FROM account
            WHERE id = OLD.account_id
        )
        AND NOT EXISTS (
            SELECT 1
            FROM account_credentials
            WHERE account_id = OLD.account_id
        ) THEN
            RAISE EXCEPTION
                'account % must have exactly one credential',
                OLD.account_id
                USING
                    ERRCODE = '23514',
                    CONSTRAINT = 'account_exactly_one_credential';
        END IF;

        RETURN NULL;
    END IF;

    IF TG_TABLE_NAME = 'account_credentials' AND TG_OP = 'UPDATE' THEN
        IF EXISTS (
            SELECT 1
            FROM account
            WHERE id = OLD.account_id
        )
        AND NOT EXISTS (
            SELECT 1
            FROM account_credentials
            WHERE account_id = OLD.account_id
        ) THEN
            RAISE EXCEPTION
                'account % must have exactly one credential',
                OLD.account_id
                USING
                    ERRCODE = '23514',
                    CONSTRAINT = 'account_exactly_one_credential';
        END IF;

        IF EXISTS (
            SELECT 1
            FROM account
            WHERE id = NEW.account_id
        )
        AND NOT EXISTS (
            SELECT 1
            FROM account_credentials
            WHERE account_id = NEW.account_id
        ) THEN
            RAISE EXCEPTION
                'account % must have exactly one credential',
                NEW.account_id
                USING
                    ERRCODE = '23514',
                    CONSTRAINT = 'account_exactly_one_credential';
        END IF;

        RETURN NULL;
    END IF;

    RETURN NULL;
END;
$$;

CREATE CONSTRAINT TRIGGER account_requires_exactly_one_credential
AFTER INSERT ON account
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW
EXECUTE FUNCTION enforce_account_exactly_one_credential();

CREATE CONSTRAINT TRIGGER account_credentials_delete_requires_account
AFTER DELETE ON account_credentials
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW
EXECUTE FUNCTION enforce_account_exactly_one_credential();

CREATE CONSTRAINT TRIGGER account_credentials_update_requires_account
AFTER UPDATE OF account_id ON account_credentials
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW
EXECUTE FUNCTION enforce_account_exactly_one_credential();