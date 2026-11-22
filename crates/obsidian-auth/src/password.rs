use argon2::password_hash::phc::PasswordHash;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use obsidian_common::{CommonError, Result};

pub fn hash_password(password: &str) -> Result<String> {
	Argon2::default()
		.hash_password(password.as_bytes())
		.map(|hash| hash.to_string())
		.map_err(|e| CommonError::Other(format!("failed to hash password: {e}")))
}

pub fn verify_password(password: &str, hash: &str) -> bool {
	PasswordHash::new(hash)
		.map(|parsed| Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok())
		.unwrap_or(false)
}
