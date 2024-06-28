use anyhow::anyhow;
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!(e).context("Failed to hash password"))
        .map(|hash| hash.to_string())
}

pub fn is_valid(password: &str, hashed_password: &str) -> bool {
    match PasswordHash::new(hashed_password) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_password_hashing() {
        let password = "password".to_string();
        let hashed_password = hash_password(&password).unwrap();

        assert!(is_valid(&password, &hashed_password));
    }

    #[test]
    fn test_invalid_password_hashing() {
        let hashed_password = hash_password("password").unwrap();

        assert!(!is_valid("1234", &hashed_password));
    }
}
