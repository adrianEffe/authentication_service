use crate::domain::model::user::User;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthMiddleware {
    pub user: User,
    pub access_token_uuid: uuid::Uuid,
}

impl AuthMiddleware {
    pub fn new(user: User, access_token_uuid: uuid::Uuid) -> AuthMiddleware {
        AuthMiddleware {
            user,
            access_token_uuid,
        }
    }
}
