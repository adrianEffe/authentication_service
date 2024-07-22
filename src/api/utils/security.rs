use anyhow::anyhow;
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;

/// Hashes a plain text password using the Argon2 algorithm.
///
/// This function generates a random salt and hashes the provided password
/// using the Argon2 hashing algorithm. The resulting hash is returned as a
/// string, which includes the salt and other parameters used for hashing.
///
/// # Arguments
///
/// * `password` - A reference to the plain text password to be hashed.
///
/// # Returns
///
/// A `Result` containing the hashed password as a `String` on success, or an `anyhow::Error` on failure.
///
/// # Errors
///
/// This function returns an error if the password hashing process fails.
///
/// # Examples
///
/// ```rust
/// use authentication_service::api::utils::security::hash_password;
/// use anyhow::Result;
///
/// fn main() -> Result<()> {
///     let password = "my_secure_password";
///     let hashed_password = hash_password(password)?;
///     println!("Hashed password: {}", hashed_password);
///     Ok(())
/// }
/// ```
pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!(e).context("Failed to hash password"))
        .map(|hash| hash.to_string())
}

/// Verifies if a plain text password matches a hashed password.
///
/// This function compares a plain text password with a hashed password
/// to determine if they match. It uses the Argon2 algorithm to verify
/// the password against the provided hash.
///
/// # Arguments
///
/// * `password` - A reference to the plain text password to be verified.
/// * `hashed_password` - A reference to the hashed password to verify against.
///
/// # Returns
///
/// * `true` if the password matches the hashed password.
/// * `false` if the password does not match or if the hashed password is invalid.
///
/// # Examples
///
/// ```rust
/// use authentication_service::api::utils::security::{hash_password, is_valid};
///
/// let password = "my_secure_password";
/// let hashed_password = hash_password(password).unwrap();
///
/// assert!(is_valid(password, &hashed_password));
/// assert!(!is_valid("wrong_password", &hashed_password));
/// ```
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
