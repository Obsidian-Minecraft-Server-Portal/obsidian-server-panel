use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;

use actix_web::http::header::AUTHORIZATION;
use actix_web::{FromRequest, HttpRequest, web, dev::Payload};
use obsidian_common::CommonError;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

const SESSION_COOKIE: &str = "obsidian_session";
const BEARER_PREFIX: &str = "Bearer ";

type Authenticating<T> = Pin<Box<dyn Future<Output = ApiResult<T>>>>;

pub struct AuthUser(pub obsidian_auth::AuthUser);

pub struct AdminUser(pub obsidian_auth::AuthUser);

impl Deref for AuthUser {
	type Target = obsidian_auth::AuthUser;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Deref for AdminUser {
	type Target = obsidian_auth::AuthUser;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl FromRequest for AuthUser {
	type Error = ApiError;
	type Future = Authenticating<Self>;

	fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
		let user = authenticate(req);
		Box::pin(async move { user.await.map(Self) })
	}
}

impl FromRequest for AdminUser {
	type Error = ApiError;
	type Future = Authenticating<Self>;

	fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
		let user = authenticate(req);
		Box::pin(async move {
			let user = user.await?;
			if user.is_admin {
				Ok(Self(user))
			} else {
				Err(CommonError::Forbidden("administrator access required".into()).into())
			}
		})
	}
}

#[derive(Clone)]
pub enum Credential {
	ApiToken(String),
	Session(String),
}

pub fn credential(req: &HttpRequest) -> Option<Credential> {
	let bearer = req
		.headers()
		.get(AUTHORIZATION)
		.and_then(|value| value.to_str().ok())
		.and_then(|value| value.strip_prefix(BEARER_PREFIX));
	match bearer {
		Some(token) => Some(Credential::ApiToken(token.trim().to_owned())),
		None => req.cookie(SESSION_COOKIE).map(|cookie| Credential::Session(cookie.value().to_owned())),
	}
}

fn authenticate(req: &HttpRequest) -> Authenticating<obsidian_auth::AuthUser> {
	let state = req.app_data::<web::Data<AppState>>().cloned();
	let credential = credential(req);
	Box::pin(async move {
		let state = state.ok_or_else(|| CommonError::Other("application state is not registered".into()))?;
		let user = credential.ok_or(ApiError::Unauthorized)?.resolve(&state).await?;
		user.ok_or(ApiError::Unauthorized)
	})
}

impl Credential {
	pub async fn resolve(&self, state: &AppState) -> ApiResult<Option<obsidian_auth::AuthUser>> {
		let user = match self {
			Self::ApiToken(token) => state.auth.user_for_api_token(token).await?,
			Self::Session(token) => state.auth.user_for_session(token).await?,
		};
		Ok(user)
	}
}
