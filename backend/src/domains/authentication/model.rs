use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuthenticatedPrincipal {
    pub account_id: Uuid,
    pub session_id: Uuid,
    pub authenticated_at: OffsetDateTime,
}
