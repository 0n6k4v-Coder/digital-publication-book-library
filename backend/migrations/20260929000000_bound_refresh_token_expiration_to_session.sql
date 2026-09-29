UPDATE authentication_refresh_token AS refresh_token
SET expires_at = session.expires_at
FROM authentication_session AS session
WHERE refresh_token.session_id = session.id
  AND refresh_token.expires_at > session.expires_at;