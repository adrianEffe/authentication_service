use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct LoginResponse {
    pub access_token: String,
    pub access_token_max_age: i64,
    pub refresh_token: String,
    pub refresh_token_max_age: i64,
}
