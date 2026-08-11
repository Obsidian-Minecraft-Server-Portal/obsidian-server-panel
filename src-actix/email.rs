use anyhow::{Result, anyhow};
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use log::{info, warn};
use std::sync::OnceLock;

static TRANSPORT: OnceLock<Option<(AsyncSmtpTransport<Tokio1Executor>, Mailbox)>> = OnceLock::new();

fn env_var(name: &str) -> Option<String> {
    std::env::var(name).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

fn build() -> Result<Option<(AsyncSmtpTransport<Tokio1Executor>, Mailbox)>> {
    let Some(host) = env_var("SMTP_HOST") else {
        return Ok(None);
    };
    let port: u16 = env_var("SMTP_PORT").map(|p| p.parse()).transpose()?.unwrap_or(587);
    let username = env_var("SMTP_USERNAME");
    let password = env_var("SMTP_PASSWORD");
    let from: Mailbox = env_var("SMTP_FROM")
        .or_else(|| username.clone())
        .ok_or_else(|| anyhow!("SMTP_FROM (or SMTP_USERNAME) is required when SMTP_HOST is set"))?
        .parse()
        .map_err(|e| anyhow!("Invalid SMTP_FROM address: {}", e))?;
    let tls = env_var("SMTP_TLS").unwrap_or_else(|| if port == 465 { "implicit".into() } else { "starttls".into() });

    let mut builder = match tls.as_str() {
        "none" => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&host),
        "implicit" => AsyncSmtpTransport::<Tokio1Executor>::relay(&host)?,
        "starttls" => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)?,
        other => return Err(anyhow!("Invalid SMTP_TLS value '{}': expected 'none', 'starttls' or 'implicit'", other)),
    }
    .port(port);
    if let (Some(username), Some(password)) = (username, password) {
        builder = builder.credentials(Credentials::new(username, password));
    }
    Ok(Some((builder.build(), from)))
}

pub fn init() {
    let transport = match build() {
        Ok(Some(t)) => {
            info!("SMTP configured; email verification and 2FA are enabled");
            Some(t)
        }
        Ok(None) => {
            info!("SMTP_HOST not set; email verification and 2FA are disabled");
            None
        }
        Err(e) => {
            warn!("Invalid SMTP configuration, email disabled: {}", e);
            None
        }
    };
    TRANSPORT.set(transport).ok();
}

pub fn is_enabled() -> bool {
    TRANSPORT.get().is_some_and(|t| t.is_some())
}

pub async fn send(to: &str, subject: &str, body: String) -> Result<()> {
    let (transport, from) = TRANSPORT.get().and_then(|t| t.as_ref()).ok_or_else(|| anyhow!("SMTP is not configured"))?;
    let message = Message::builder()
        .from(from.clone())
        .to(to.parse().map_err(|e| anyhow!("Invalid recipient address: {}", e))?)
        .subject(subject)
        .body(body)?;
    transport.send(message).await?;
    Ok(())
}

pub async fn send_code(to: &str, action: &str, code: &str) -> Result<()> {
    send(
        to,
        &format!("Obsidian Server Panel - {} code", action),
        format!(
            "Your Obsidian Server Panel {} code is: {}\n\nThis code expires in 10 minutes and can only be used once.\nIf you did not request this code, you can safely ignore this email.",
            action, code
        ),
    )
    .await
}
