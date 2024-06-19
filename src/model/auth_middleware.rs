use crate::model::user::User;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthMiddleware {
    pub user: User,
    pub access_token_uuid: uuid::Uuid,
}
