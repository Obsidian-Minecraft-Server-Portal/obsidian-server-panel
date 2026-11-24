use std::time::{SystemTime, UNIX_EPOCH};

use obsidian_auth::{AuthService, AuthUser, ClientInfo, LoginOutcome, LoginRequest, NewUser, SecretBox, generate_token};
use obsidian_common::{CommonError, EventBus, Paths};
use obsidian_db::Db;
use totp_rs::{Builder, Secret};

const STEP: u64 = 30;

async fn service() -> (tempfile::TempDir, AuthService) {
	let dir = tempfile::tempdir().unwrap();
	let paths = Paths::new(dir.path()).unwrap();
	let db = Db::open(&paths).await.unwrap();
	let secrets = SecretBox::load_or_create(paths.data_dir()).unwrap();
	(dir, AuthService::new(db, EventBus::new(), secrets))
}

fn client(ip: &str) -> ClientInfo {
	ClientInfo {
		ip: ip.to_owned(),
		user_agent: Some("test".to_owned()),
	}
}

fn login(password: &str, totp: &str) -> LoginRequest {
	LoginRequest {
		username: "admin".to_owned(),
		password: password.to_owned(),
		totp_code: Some(totp.to_owned()),
		email_code: None,
	}
}

fn code_at(secret: &str, time: u64) -> String {
	let secret = Secret::try_from_base32(secret).unwrap();
	Builder::new()
		.with_secret(secret)
		.build()
		.unwrap()
		.generate(time)
		.to_string()
}

fn now() -> u64 {
	SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

#[tokio::test]
async fn totp_code_is_accepted_once_per_time_step() {
	let (_dir, auth) = service().await;
	let password = generate_token();
	let admin = auth
		.create_first_admin(&NewUser {
			username: "admin".to_owned(),
			email: None,
			password: password.clone(),
			is_admin: true,
			instance_limit: None,
		})
		.await
		.unwrap();
	let user = AuthUser {
		id: admin.id,
		username: admin.username.clone(),
		is_admin: true,
	};
	let setup = auth.totp_setup(&user).await.unwrap();
	let current = now();
	auth.totp_confirm(&user, &code_at(&setup.secret, current))
		.await
		.unwrap();

	let replayed = auth
		.login(&login(&password, &code_at(&setup.secret, current)), &client("10.0.1.1"), false)
		.await;
	assert!(matches!(replayed, Err(CommonError::Unauthorized(_))));

	let next_step = code_at(&setup.secret, current + STEP);
	let accepted = auth
		.login(&login(&password, &next_step), &client("10.0.1.2"), false)
		.await
		.unwrap();
	assert!(matches!(accepted, LoginOutcome::Authenticated { .. }));

	let reused = auth.login(&login(&password, &next_step), &client("10.0.1.3"), false).await;
	assert!(matches!(reused, Err(CommonError::Unauthorized(_))));
}
