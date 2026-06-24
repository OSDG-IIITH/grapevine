use argon2::Argon2;
use password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};

/// Hash a password with argon2id (default params) and a random per-password salt.
/// Returns the PHC string suitable for storing in `users.password_hash`.
/// Never logs the password.
pub fn hash(password: &str) -> Result<String, password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Constant-time verify of `password` against a stored PHC hash string.
/// Returns false on any mismatch or malformed hash; never logs secrets.
pub fn verify(password: &str, stored_hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(stored_hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_then_verify_roundtrips() {
        let h = hash("correct horse battery").unwrap();
        assert!(verify("correct horse battery", &h));
        assert!(!verify("wrong password", &h));
    }

    #[test]
    fn malformed_hash_does_not_panic() {
        assert!(!verify("anything", "not-a-phc-string"));
    }
}
