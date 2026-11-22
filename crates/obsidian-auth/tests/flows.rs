use obsidian_auth::{AuthService, ClientInfo, LoginOutcome, LoginRequest, NewUser, SecretBox};
use obsidian_common::{CommonError, EventBus, Paths};
use obsidian_db::Db;
use totp_rs::{Builder, Secret};

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

fn login(username: &str, password: &str, totp: Option<&str>) -> LoginRequest {
	LoginRequest {
		username: username.to_owned(),
		password: password.to_owned(),
		totp_code: totp.map(str::to_owned),
		email_code: None,
	}
}

fn new_user(username: &str, password: &str) -> NewUser {
	NewUser {
		username: username.to_owned(),
		email: None,
		password: password.to_owned(),
		is_admin: false,
		instance_limit: None,
	}
}

fn current_code(secret: &str) -> String {
	let secret = Secret::try_from_base32(secret).unwrap();
	Builder::new()
		.with_secret(secret)
		.build()
		.unwrap()
		.generate_current()
		.to_string()
}

#[tokio::test]
async fn setup_then_login_from_new_address_is_allowed_without_second_factor() {
	let (_dir, auth) = service().await;
	assert!(auth.bootstrap_admin("root", None, "correct horse").await.unwrap());
	assert!(!auth.bootstrap_admin("other", None, "another pass").await.unwrap());

	let wrong = auth
		.login(&login("root", "nope", None), &client("10.0.0.1"), false)
		.await;
	assert!(matches!(wrong, Err(CommonError::Unauthorized(_))));

	let outcome = auth
		.login(&login("root", "correct horse", None), &client("10.0.0.1"), false)
		.await
		.unwrap();
	let LoginOutcome::Authenticated { session_token, user } = outcome else {
		panic!("expected a session")
	};
	assert!(user.is_admin);
	assert!(auth.user_for_session(&session_token).await.unwrap().is_some());
	assert!(
		auth.security_events(user.id)
			.await
			.unwrap()
			.iter()
			.any(|e| e.kind == "new_ip_login")
	);
}

#[tokio::test]
async fn totp_gates_new_addresses_and_recovery_codes_work_once() {
	let (_dir, auth) = service().await;
	let admin = auth
		.create_first_admin(&new_user("admin", "correct horse"))
		.await
		.unwrap();
	let user = obsidian_auth::AuthUser {
		id: admin.id,
		username: admin.username.clone(),
		is_admin: true,
	};

	let setup = auth.totp_setup(&user).await.unwrap();
	assert!(setup.otpauth_url.starts_with("otpauth://totp/"));
	assert!(setup.qr_svg.contains("<svg"));
	let confirmed_code = current_code(&setup.secret);
	let codes = auth.totp_confirm(&user, &confirmed_code).await.unwrap();
	assert_eq!(codes.len(), 10);

	let pending = auth
		.login(&login("admin", "correct horse", None), &client("10.0.0.2"), false)
		.await
		.unwrap();
	assert!(matches!(pending, LoginOutcome::TotpRequired));
	let wrong = auth
		.login(
			&login("admin", "correct horse", Some("000000")),
			&client("10.0.0.2"),
			false,
		)
		.await;
	assert!(matches!(wrong, Err(CommonError::Unauthorized(_))));
	let replayed = auth
		.login(
			&login("admin", "correct horse", Some(&confirmed_code)),
			&client("10.0.0.2"),
			false,
		)
		.await;
	assert!(matches!(replayed, Err(CommonError::Unauthorized(_))));

	let by_recovery = auth
		.login(
			&login("admin", "correct horse", Some(&codes[0])),
			&client("10.0.0.3"),
			false,
		)
		.await
		.unwrap();
	assert!(matches!(by_recovery, LoginOutcome::Authenticated { .. }));
	let reused = auth
		.login(
			&login("admin", "correct horse", Some(&codes[0])),
			&client("10.0.0.4"),
			false,
		)
		.await;
	assert!(matches!(reused, Err(CommonError::Unauthorized(_))));
}

#[tokio::test]
async fn password_change_keeps_only_the_current_session() {
	let (_dir, auth) = service().await;
	let admin = auth
		.create_first_admin(&new_user("admin", "correct horse"))
		.await
		.unwrap();
	let client = client("10.0.0.5");
	let first = auth.start_session(admin.id, &client).await.unwrap();
	let second = auth.start_session(admin.id, &client).await.unwrap();

	auth.change_password(admin.id, "correct horse", "battery staple", Some(&first), &client)
		.await
		.unwrap();

	assert!(auth.user_for_session(&first).await.unwrap().is_some());
	assert!(auth.user_for_session(&second).await.unwrap().is_none());
}

#[tokio::test]
async fn invites_create_one_account_and_notifications_reach_the_owner() {
	let (_dir, auth) = service().await;
	let admin = auth
		.create_first_admin(&new_user("admin", "correct horse"))
		.await
		.unwrap();
	let (_, token) = auth.create_invite(admin.id, Some(24)).await.unwrap();
	assert!(auth.invite_is_valid(&token).await.unwrap());

	let (session, profile) = auth
		.accept_invite(&token, &new_user("guest", "guest password"), &client("10.0.0.6"))
		.await
		.unwrap();
	assert!(!profile.is_admin);
	assert!(auth.user_for_session(&session).await.unwrap().is_some());
	assert!(!auth.invite_is_valid(&token).await.unwrap());
	let reused = auth
		.accept_invite(&token, &new_user("another", "another pass"), &client("10.0.0.6"))
		.await;
	assert!(matches!(reused, Err(CommonError::Invalid(_))));

	let taken = auth.create_user(admin.id, &new_user("guest", "guest password")).await;
	assert!(matches!(taken, Err(CommonError::Conflict(_))));

	let notice = auth.notify(admin.id, "info", "Hello", "body").await.unwrap();
	assert!(!notice.read);
	auth.mark_notifications_read(admin.id, Some(&[notice.id]))
		.await
		.unwrap();
	let listed = auth.list_notifications(admin.id).await.unwrap();
	assert!(listed.iter().all(|n| n.read));
}

#[tokio::test]
async fn api_tokens_authenticate_and_record_use_until_revoked() {
	let (_dir, auth) = service().await;
	let admin = auth
		.create_first_admin(&new_user("admin", "correct horse"))
		.await
		.unwrap();
	let created = auth.create_api_token(admin.id, "ci", Some(30)).await.unwrap();
	assert!(created.info.last_used_at.is_none());

	let user = auth.user_for_api_token(&created.token).await.unwrap().unwrap();
	assert_eq!(user.id, admin.id);
	let listed = auth.list_api_tokens(admin.id).await.unwrap();
	assert!(listed[0].last_used_at.is_some());

	auth.revoke_api_token(admin.id, created.info.id).await.unwrap();
	assert!(auth.user_for_api_token(&created.token).await.unwrap().is_none());
}

#[tokio::test]
async fn email_codes_verify_new_addresses_and_lock_after_repeated_failures() {
	let (_dir, auth) = service().await;
	let mut admin = new_user("admin", "correct horse");
	admin.email = Some("admin@example.com".to_owned());
	auth.create_first_admin(&admin).await.unwrap();
	let address = client("10.0.0.7");
	let with_code = |code: &str| LoginRequest {
		email_code: Some(code.to_owned()),
		..login("admin", "correct horse", None)
	};

	let LoginOutcome::EmailCodeRequired { to, code } =
		auth.login(&login("admin", "correct horse", None), &address, true).await.unwrap()
	else {
		panic!("expected an emailed code")
	};
	assert_eq!(to, "admin@example.com");
	for _ in 0..5 {
		let wrong = auth.login(&with_code("not-it"), &address, true).await;
		assert!(matches!(wrong, Err(CommonError::Unauthorized(_))));
	}
	let locked = auth.login(&with_code(&code), &address, true).await;
	assert!(matches!(locked, Err(CommonError::Unauthorized(_))));

	let LoginOutcome::EmailCodeRequired { code, .. } =
		auth.login(&login("admin", "correct horse", None), &address, true).await.unwrap()
	else {
		panic!("expected a fresh code")
	};
	let ok = auth.login(&with_code(&code), &address, true).await.unwrap();
	assert!(matches!(ok, LoginOutcome::Authenticated { .. }));
	let trusted = auth.login(&login("admin", "correct horse", None), &address, true).await.unwrap();
	assert!(matches!(trusted, LoginOutcome::Authenticated { .. }));
}
