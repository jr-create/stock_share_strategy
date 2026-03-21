use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use anyhow::Result;

pub fn hash(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
        .to_string();
    Ok(hash)
}

pub fn verify(password: &str, hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let password = "test_password_123";
        let hash = hash(password).unwrap();
        assert!(verify(password, &hash));
        assert!(!verify("wrong_password", &hash));
    }

    #[test]
    fn test_different_passwords_different_hashes() {
        let hash1 = hash("password1").unwrap();
        let hash2 = hash("password2").unwrap();
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_same_password_different_hashes() {
        let hash1 = hash("password").unwrap();
        let hash2 = hash("password").unwrap();
        assert_ne!(hash1, hash2);
        assert!(verify("password", &hash1));
        assert!(verify("password", &hash2));
    }
}
