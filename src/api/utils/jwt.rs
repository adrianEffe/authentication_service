use crate::domain::model::token::{TokenClaims, TokenDetails};
use anyhow::Result;
use base64::{engine::general_purpose, Engine};

/// Verifies a JSON Web Token (JWT) using the provided public key.
///
/// This function decodes and verifies a JWT using a public RSA key. The JWT is decoded to extract its claims,
/// which are then parsed to obtain the user ID and token UUID. If the token is valid and the claims can be parsed
/// successfully, a `TokenDetails` struct is returned, containing the user ID and token UUID.
///
/// The process includes:
/// 1. **Decoding the Public Key:** Converts the base64-encoded public key string into bytes and then into a UTF-8
///    string representation.
/// 2. **JWT Validation:** Uses the RSA public key to validate the token's signature and decode its claims.
/// 3. **Parsing Claims:** Extracts the user ID and token UUID from the token claims.
///
/// # Arguments
///
/// * `public_key` - A base64-encoded string representation of the RSA public key used for token verification.
/// * `token` - The JWT to be verified.
///
/// # Returns
///
/// A `Result` containing a `TokenDetails` struct with the extracted user ID and token UUID if the token is valid,
/// or an `anyhow::Error` if the verification or decoding fails.
///
/// # Errors
///
/// This function returns an error if the public key decoding, JWT decoding, or claims parsing fails.
pub fn verify_jwt(public_key: &str, token: &str) -> Result<TokenDetails> {
    let bytes_public_key = general_purpose::STANDARD.decode(public_key)?;
    let decoded_public_key = String::from_utf8(bytes_public_key)?;

    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);

    let decoded = jsonwebtoken::decode::<TokenClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_rsa_pem(decoded_public_key.as_bytes())?,
        &validation,
    )?;

    let user_id = uuid::Uuid::parse_str(decoded.claims.sub.as_str())?;
    let token_uuid = uuid::Uuid::parse_str(decoded.claims.token_uuid.as_str())?;

    Ok(TokenDetails {
        token: None,
        token_uuid,
        user_id,
        expires_in: None,
    })
}

/// Generates a JSON Web Token (JWT) for a user with the given time-to-live (TTL) and private key.
///
/// This function creates a JWT for a user, including a unique token UUID and an expiration timestamp. The JWT is
/// signed using a private RSA key. The generated token is returned along with other token details.
///
/// The process includes:
/// 1. **Decoding the Private Key:** Converts the base64-encoded private key string into bytes and then into a UTF-8
///    string representation.
/// 2. **Creating Claims:** Constructs the claims for the token, including the user ID, token UUID, and expiration
///    time.
/// 3. **Encoding JWT:** Uses the RSA private key to sign and encode the token with the specified claims.
///
/// # Arguments
///
/// * `user_id` - The UUID of the user for whom the token is being generated.
/// * `ttl` - The time-to-live (TTL) in minutes for the token, determining how long the token is valid.
/// * `private_key` - A base64-encoded string representation of the RSA private key used for signing the token.
///
/// # Returns
///
/// A `Result` containing a `TokenDetails` struct with the generated token and its details if successful, or an
/// `anyhow::Error` if the token generation or encoding fails.
///
/// # Errors
///
/// This function returns an error if the private key decoding, JWT encoding, or token details creation fails.
pub fn generate_jwt(user_id: uuid::Uuid, ttl: i64, private_key: &str) -> Result<TokenDetails> {
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

    let token = encode_jwt(&claims, &decoded_private_key)?;
    token_details.token = Some(token);

    Ok(token_details)
}

/// Encodes claims into a JSON Web Token (JWT) using the provided private key.
///
/// This helper function creates a JWT by encoding the given claims with an RSA private key. The resulting token
/// is a string representation that can be used for authentication and authorization.
///
/// The process includes:
/// 1. **Creating Header:** Sets up the JWT header with the RS256 algorithm.
/// 2. **Encoding:** Uses the private key to sign the token and attach the claims.
///
/// # Arguments
///
/// * `claims` - The claims to be included in the JWT, such as user ID and token UUID.
/// * `private_key` - A UTF-8 string representation of the RSA private key used for signing the token.
///
/// # Returns
///
/// A `Result` containing the encoded JWT as a `String` if successful, or an `anyhow::Error` if the encoding fails.
///
/// # Errors
///
/// This function returns an error if the JWT encoding process fails.
fn encode_jwt(claims: &TokenClaims, private_key: &str) -> Result<String> {
    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    let token = jsonwebtoken::encode(
        &header,
        &claims,
        &jsonwebtoken::EncodingKey::from_rsa_pem(private_key.as_bytes())?,
    )?;

    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helper::config::Config;
    use dotenv::dotenv;

    #[test]
    fn test_encoding_jwt() {
        dotenv().ok();
        let config = Config::init();
        let user_id = uuid::Uuid::new_v4();

        let token_details = generate_jwt(
            user_id,
            config.access_token_max_age,
            &config.access_token_private_key,
        );

        assert_eq!(token_details.unwrap().user_id, user_id);
    }

    #[test]
    fn test_decoding_jwt() {
        dotenv().ok();
        let config = Config::init();
        let user_id = uuid::Uuid::new_v4();

        let token_details = generate_jwt(
            user_id,
            config.access_token_max_age,
            &config.access_token_private_key,
        )
        .unwrap();

        let verified_details = verify_jwt(
            &config.access_token_public_key,
            &token_details.token.unwrap(),
        );

        assert_eq!(verified_details.unwrap().user_id, user_id);
    }
}
