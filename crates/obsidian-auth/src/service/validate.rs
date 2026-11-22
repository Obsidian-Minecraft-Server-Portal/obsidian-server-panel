use obsidian_common::{CommonError, Result};

const USERNAME_RANGE: std::ops::RangeInclusive<usize> = 3..=32;
const PASSWORD_MIN_CHARS: usize = 8;
const PASSWORD_MAX_BYTES: usize = 1024;
const EMAIL_MAX_BYTES: usize = 254;

pub fn username(value: &str) -> Result<()> {
	let valid = USERNAME_RANGE.contains(&value.len())
		&& value
			.chars()
			.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'));
	if valid {
		Ok(())
	} else {
		Err(CommonError::Invalid(
			"username must be 3-32 letters, digits, '_', '-' or '.'".into(),
		))
	}
}

pub fn password(value: &str) -> Result<()> {
	if value.chars().count() < PASSWORD_MIN_CHARS {
		return Err(CommonError::Invalid(format!(
			"password must be at least {PASSWORD_MIN_CHARS} characters"
		)));
	}
	if value.len() > PASSWORD_MAX_BYTES {
		return Err(CommonError::Invalid("password is too long".into()));
	}
	Ok(())
}

pub fn email(value: &str) -> Result<()> {
	let valid = value.len() <= EMAIL_MAX_BYTES
		&& !value.chars().any(char::is_whitespace)
		&& value
			.split_once('@')
			.is_some_and(|(local, domain)| !local.is_empty() && domain.contains('.'));
	if valid {
		Ok(())
	} else {
		Err(CommonError::Invalid("email address is not valid".into()))
	}
}
