use obsidian_auth::{Permission, Role, generate_token, hash_password, hash_token, verify_password};

#[test]
fn password_hash_round_trip() {
	let hash = hash_password("correct horse").unwrap();
	assert!(hash.starts_with("$argon2id$"));
	assert!(verify_password("correct horse", &hash));
	assert!(!verify_password("wrong", &hash));
	assert_ne!(hash, hash_password("correct horse").unwrap());
}

#[test]
fn tokens_are_unique_and_hashed() {
	let token = generate_token();
	assert_eq!(token.len(), 64);
	assert_ne!(token, generate_token());
	assert_eq!(hash_token(&token), hash_token(&token));
	assert_ne!(hash_token(&token), token);
}

#[test]
fn roles_expand_to_permissions() {
	assert_eq!(Role::CoOwner.permissions(), Permission::all());
	assert!(Role::Viewer.permissions().contains(Permission::CONSOLE_READ));
	assert!(!Role::Viewer.permissions().contains(Permission::CONSOLE_SEND));
	assert_eq!(Permission::by_name("files.sftp"), Some(Permission::FILES_SFTP));
	assert_eq!(Permission::FILES_SFTP.names(), vec!["files.sftp"]);
}
