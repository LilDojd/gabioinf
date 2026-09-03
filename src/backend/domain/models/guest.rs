use crate::shared::models::{Guest, GuestId};
use axum_login::AuthUser;
impl AuthUser for Guest {
    type Id = GuestId;
    fn id(&self) -> Self::Id {
        self.id
    }
    fn session_auth_hash(&self) -> &[u8] {
        self.username.as_bytes()
    }
}
