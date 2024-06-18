use anyhow::Result;
use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub token_uuid: String,
    pub exp: i64,
    pub iat: i64,
    pub nbf: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenDetails {
    pub token: Option<String>,
    pub token_uuid: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub expires_in: Option<i64>,
}

pub fn generate_jwt(user_id: uuid::Uuid, ttl: i64, private_key: String) -> Result<TokenDetails> {
    let bytes_private_key = general_purpose::STANDARD.decode(private_key)?;
    let decoded_private_key = String::from_utf8(bytes_private_key)?;

    let now = chrono::Utc::now();
    let exp = (now + chrono::Duration::minutes(ttl)).timestamp();

    let mut token_details = TokenDetails {
        user_id,
        token_uuid: uuid::Uuid::new_v4(),
        expires_in: Some(exp),
        token: None,
    };

    let claims = TokenClaims {
        sub: token_details.user_id.to_string(),
        token_uuid: token_details.token_uuid.to_string(),
        exp,
        iat: now.timestamp(),
        nbf: now.timestamp(),
    };

    let token = encode_jwt(&claims, decoded_private_key)?;
    token_details.token = Some(token);

    Ok(token_details)
}

fn encode_jwt(claims: &TokenClaims, private_key: String) -> Result<String> {
    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    let token = jsonwebtoken::encode(
        &header,
        &claims,
        &jsonwebtoken::EncodingKey::from_rsa_pem(private_key.as_bytes())?,
    )?;

    Ok(token)
}
