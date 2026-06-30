use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::AppError;

/// Hashes a plaintext password with Argon2id.
///
/// # Errors
///
/// Returns an error if hashing fails.
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {e}")))
}

/// Verifies a plaintext password against a stored Argon2 hash.
///
/// # Errors
///
/// Returns an error if the hash format is invalid.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("Invalid password hash: {e}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

/// Generates a cryptographically secure session token.
#[must_use]
pub fn generate_session_token() -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use rand::RngCore;

    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Generates a new API key with `ns_` prefix.
#[must_use]
pub fn generate_api_key() -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use rand::RngCore;

    let mut bytes = [0u8; 24];
    rand::rng().fill_bytes(&mut bytes);
    format!("ns_{}", URL_SAFE_NO_PAD.encode(bytes))
}

/// Returns the SHA-256 hex digest of an API key for storage.
#[must_use]
pub fn hash_api_key(key: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(key.as_bytes());
    hex::encode(digest)
}

/// Converts a display name into a URL-safe slug.
#[must_use]
pub fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_hash_roundtrip() {
        let hash = hash_password("secure-password").expect("hash");
        assert!(verify_password("secure-password", &hash).expect("verify"));
        assert!(!verify_password("wrong-password", &hash).expect("verify"));
    }

    #[test]
    fn slugify_normalizes_name() {
        assert_eq!(slugify("My Cool Project!"), "my-cool-project");
    }

    #[test]
    fn api_key_has_prefix() {
        let key = generate_api_key();
        assert!(key.starts_with("ns_"));
    }
}
