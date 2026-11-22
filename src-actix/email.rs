use lettre::message::Mailbox;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use obsidian_common::CommonError;

use crate::config::{SmtpConfig, SmtpTls};
use crate::error::ApiResult;
use crate::state::AppState;

pub const SMTP_SETTING: &str = "smtp";
const APP_NAME: &str = "Obsidian";
const CODE_LIFETIME_MINUTES: u32 = 10;

/// SMTP from the environment or config file wins; otherwise the panel-managed setting applies.
pub async fn effective_smtp(state: &AppState) -> ApiResult<Option<SmtpConfig>> {
	if let Some(smtp) = &state.config.smtp {
		return Ok(Some(smtp.clone()));
	}
	Ok(state.auth.secret_setting(SMTP_SETTING).await?)
}

pub async fn send_login_code(state: &AppState, to: &str, code: &str) -> ApiResult<()> {
	let body = format!(
		"Your {APP_NAME} verification code is {code}.\n\nIt expires in {CODE_LIFETIME_MINUTES} minutes. \
		 If you did not try to sign in, change your password."
	);
	send(state, to, &format!("{APP_NAME} sign-in code"), &body).await
}

pub async fn send_password_reset(state: &AppState, to: &str, link: &str) -> ApiResult<()> {
	let body = format!(
		"Use this link to choose a new {APP_NAME} password. It expires in one hour and works once:\n\n{link}\n\n\
		 If you did not ask for a reset, ignore this email."
	);
	send(state, to, &format!("Reset your {APP_NAME} password"), &body).await
}

pub async fn send_test(state: &AppState, to: &str) -> ApiResult<()> {
	send(
		state,
		to,
		&format!("{APP_NAME} SMTP test"),
		"SMTP is configured correctly.",
	)
	.await
}

async fn send(state: &AppState, to: &str, subject: &str, body: &str) -> ApiResult<()> {
	let smtp = effective_smtp(state)
		.await?
		.ok_or_else(|| CommonError::Invalid("SMTP is not configured".into()))?;
	let message = Message::builder()
		.from(parse_mailbox(&smtp.from)?)
		.to(parse_mailbox(to)?)
		.subject(subject)
		.header(ContentType::TEXT_PLAIN)
		.body(body.to_owned())
		.map_err(|e| CommonError::Other(format!("failed to build email: {e}")))?;
	transport(&smtp)?
		.send(message)
		.await
		.map_err(|e| CommonError::Other(format!("failed to send email: {e}")))?;
	Ok(())
}

fn parse_mailbox(address: &str) -> ApiResult<Mailbox> {
	address
		.parse()
		.map_err(|e| CommonError::Invalid(format!("invalid email address {address}: {e}")).into())
}

fn transport(smtp: &SmtpConfig) -> ApiResult<AsyncSmtpTransport<Tokio1Executor>> {
	let builder = match smtp.tls {
		SmtpTls::Tls => AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp.host),
		SmtpTls::Starttls => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp.host),
		SmtpTls::None => Ok(AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&smtp.host)),
	}
	.map_err(|e| CommonError::Other(format!("invalid SMTP host: {e}")))?;
	let builder = builder.port(smtp.port);
	let builder = match (&smtp.username, &smtp.password) {
		(Some(user), Some(pass)) => builder.credentials(Credentials::new(user.clone(), pass.clone())),
		_ => builder,
	};
	Ok(builder.build())
}
