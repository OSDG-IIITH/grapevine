//! Shared credential + email validators for the auth flows.
//! Reused by the register/login/verify handlers (wired by a later agent).

pub const PASSWORD_MIN_LEN: usize = 8;
pub const PASSWORD_MAX_LEN: usize = 128;
pub const USERNAME_MIN_LEN: usize = 3;
pub const USERNAME_MAX_LEN: usize = 32;

/// Validate a plaintext password. Min 8 chars, max 128 (bound argon2 cost),
/// no composition rules. Rejects empty/whitespace-only.
/// Returns Ok(()) or a user-facing error message.
pub fn validate_password(password: &str) -> Result<(), String> {
    if password.trim().is_empty() {
        return Err("password must not be empty".into());
    }
    let len = password.chars().count();
    if len < PASSWORD_MIN_LEN {
        return Err(format!("password must be at least {PASSWORD_MIN_LEN} characters"));
    }
    if len > PASSWORD_MAX_LEN {
        return Err(format!("password must be at most {PASSWORD_MAX_LEN} characters"));
    }
    Ok(())
}

/// Validate and normalize a username. 3–32 chars from `[a-z0-9_.-]`.
/// Compared/stored lowercased (case-insensitive uniqueness).
/// Returns the normalized (lowercased) username on success, or an error message.
pub fn normalize_username(username: &str) -> Result<String, String> {
    let normalized = username.trim().to_lowercase();
    let len = normalized.chars().count();
    if len < USERNAME_MIN_LEN || len > USERNAME_MAX_LEN {
        return Err(format!(
            "username must be {USERNAME_MIN_LEN}–{USERNAME_MAX_LEN} characters"
        ));
    }
    if !normalized
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '.' | '-'))
    {
        return Err("username may only contain a-z, 0-9, '_', '.', '-'".into());
    }
    Ok(normalized)
}

/// Normalize a security question answer: lowercase, alphanumeric only.
pub fn normalize_answer(answer: &str) -> String {
    answer.trim().to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect()
}

/// Whether `email`'s domain is in `allowed_domains` (case-insensitive).
pub fn is_allowed_email_domain(email: &str, allowed_domains: &[String]) -> bool {
    let email = email.trim().to_lowercase();
    let Some(domain) = email.rsplit('@').next().filter(|d| *d != email) else {
        return false;
    };
    allowed_domains.iter().any(|d| d == domain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_rules() {
        assert!(validate_password("12345678").is_ok());
        assert!(validate_password("short").is_err());
        assert!(validate_password("   ").is_err());
        assert!(validate_password(&"a".repeat(129)).is_err());
    }

    #[test]
    fn username_rules() {
        assert_eq!(normalize_username("Alice_99").unwrap(), "alice_99");
        assert!(normalize_username("ab").is_err());
        assert!(normalize_username("has space").is_err());
        assert!(normalize_username("bad!char").is_err());
    }

    #[test]
    fn email_domain_check() {
        let allowed = vec![
            "students.iiit.ac.in".to_string(),
            "research.iiit.ac.in".to_string(),
        ];
        assert!(is_allowed_email_domain("a@students.iiit.ac.in", &allowed));
        assert!(is_allowed_email_domain("A@Research.IIIT.ac.in", &allowed));
        assert!(!is_allowed_email_domain("a@gmail.com", &allowed));
        assert!(!is_allowed_email_domain("noatsign", &allowed));
    }
}
