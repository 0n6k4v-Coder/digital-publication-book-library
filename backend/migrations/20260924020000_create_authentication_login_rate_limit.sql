CREATE TABLE authentication_login_attempt (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    email_key TEXT NOT NULL,
    source_ip_key TEXT NOT NULL,
    failed BOOLEAN NOT NULL,
    email_counted BOOLEAN NOT NULL,
    source_ip_counted BOOLEAN NOT NULL,
    attempted_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX authentication_login_attempt_email_idx
    ON authentication_login_attempt (email_key, attempted_at);

CREATE INDEX authentication_login_attempt_source_ip_idx
    ON authentication_login_attempt (source_ip_key, attempted_at);