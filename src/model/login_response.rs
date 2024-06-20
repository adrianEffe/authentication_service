use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct LoginResponse {
    pub access_token: String,
}
