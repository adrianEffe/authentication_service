use serde::Serialize;

use super::token_uuid::TokenUuid;

#[derive(Debug, Serialize)]
pub struct LogoutResponse(String);

impl LogoutResponse {
    pub fn new(message: &str) -> LogoutResponse {
        LogoutResponse(message.to_string())
    }
}

#[derive(Debug)]
pub struct LogoutRequest {
    token_uuid: TokenUuid,
}

impl LogoutRequest {
    pub fn new(token_uuid: uuid::Uuid) -> LogoutRequest {
        LogoutRequest {
            token_uuid: TokenUuid::new(token_uuid),
        }
    }

    pub fn get_uuid(&self) -> &TokenUuid {
        &self.token_uuid
    }
}
