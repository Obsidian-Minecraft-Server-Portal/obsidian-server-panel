use actix_web::{HttpResponse, delete, get, patch, post, web};
use obsidian_auth::{Permission, Role};
use serde::Deserialize;
use obsidian_common::{CommonError, InstanceId, UserId};

use super::access::{Access, load};
use super::model::{AddMemberRequest, MemberView, UpdateMemberRequest};
use super::store::{self, MemberRow};
use crate::auth::AuthUser;
use crate::error::ApiResult;
use crate::state::AppState;

const MIN_CANDIDATE_PREFIX: usize = 2;
const CANDIDATE_LIMIT: u32 = 10;

#[derive(Debug, Deserialize)]
pub struct CandidateQuery {
	#[serde(default)]
	pub q: String,
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/member-candidates",
	tag = "members",
	params(
		("id" = i64, Path, description = "Instance id"),
		("q" = Option<String>, Query, description = "Username prefix, at least two characters"),
	),
	responses(
		(status = 200, description = "Up to ten panel usernames that can be invited", body = Vec<String>),
		(status = 403, description = "Missing members.manage"),
		(status = 404, description = "Instance not found"),
	)
)]
#[get("/instances/{id}/member-candidates")]
pub async fn member_candidates(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
	web::Query(query): web::Query<CandidateQuery>,
) -> ApiResult<web::Json<Vec<String>>> {
	let instance = path.into_inner();
	let access = load(&state, &user, instance).await?;
	access.require(Permission::MEMBERS_MANAGE)?;
	let prefix = query.q.trim();
	if prefix.chars().count() < MIN_CANDIDATE_PREFIX {
		return Ok(web::Json(Vec::new()));
	}
	Ok(web::Json(store::candidates(state.db.pool(), instance, prefix, CANDIDATE_LIMIT).await?))
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/members",
	tag = "members",
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 200, description = "Owner followed by members", body = Vec<MemberView>),
		(status = 403, description = "No access to this instance"),
		(status = 404, description = "Instance not found"),
	)
)]
#[get("/instances/{id}/members")]
pub async fn list_members(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
) -> ApiResult<web::Json<Vec<MemberView>>> {
	let instance = path.into_inner();
	let access = load(&state, &user, instance).await?;
	access.ensure_member()?;
	let mut views = vec![MemberView::owner(access.owner)];
	views.extend(store::members(state.db.pool(), instance).await?.into_iter().map(MemberView::member));
	Ok(web::Json(views))
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/members",
	tag = "members",
	params(("id" = i64, Path, description = "Instance id")),
	request_body = AddMemberRequest,
	responses(
		(status = 201, description = "Member added", body = MemberView),
		(status = 400, description = "Unknown role"),
		(status = 403, description = "Missing members.manage or granting permissions not held"),
		(status = 404, description = "Instance or user not found"),
		(status = 409, description = "Already a member or the owner"),
	)
)]
#[post("/instances/{id}/members")]
pub async fn add_member(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
	body: web::Json<AddMemberRequest>,
) -> ApiResult<HttpResponse> {
	let instance = path.into_inner();
	let pool = state.db.pool();
	let access = load(&state, &user, instance).await?;
	access.require(Permission::MEMBERS_MANAGE)?;
	let role = parse_role(&body.role)?;
	let granted = role.permissions();
	access.can_grant(granted)?;
	let target = store::user_by_name(pool, &body.username)
		.await?
		.ok_or_else(|| CommonError::NotFound(format!("user {}", body.username)))?;
	if target.id == access.owner.id {
		return Err(CommonError::Conflict("user already owns this instance".into()).into());
	}
	let bits = granted.bits() as i64;
	if !store::insert_member(pool, instance, target.id, role.as_str(), bits).await? {
		return Err(CommonError::Conflict("user is already a member of this instance".into()).into());
	}
	let row = MemberRow {
		user_id: target.id,
		username: target.username,
		role: role.as_str().to_owned(),
		permissions: bits,
	};
	Ok(HttpResponse::Created().json(MemberView::member(row)))
}

#[utoipa::path(
	patch,
	path = "/api/v1/instances/{id}/members/{user_id}",
	tag = "members",
	params(
		("id" = i64, Path, description = "Instance id"),
		("user_id" = i64, Path, description = "Member user id"),
	),
	request_body = UpdateMemberRequest,
	responses(
		(status = 200, description = "Member updated", body = MemberView),
		(status = 400, description = "Unknown role or permission"),
		(status = 403, description = "Missing members.manage or changing permissions not held"),
		(status = 404, description = "Instance or member not found"),
	)
)]
#[patch("/instances/{id}/members/{user_id}")]
pub async fn update_member(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(InstanceId, UserId)>,
	body: web::Json<UpdateMemberRequest>,
) -> ApiResult<web::Json<MemberView>> {
	let (instance, target) = path.into_inner();
	let pool = state.db.pool();
	let access = load(&state, &user, instance).await?;
	access.require(Permission::MEMBERS_MANAGE)?;
	reject_owner(&access, target)?;
	let current = store::member(pool, instance, target)
		.await?
		.ok_or_else(|| CommonError::NotFound(format!("member {target}")))?;
	access.can_grant(stored_permissions(&current))?;

	let role = body.role.as_deref().map(parse_role).transpose()?;
	let permissions = match (&body.permissions, role) {
		(Some(names), _) => parse_permissions(names)?,
		(None, Some(role)) => role.permissions(),
		(None, None) => stored_permissions(&current),
	};
	access.can_grant(permissions)?;
	let role_name = role.map_or_else(|| current.role.clone(), |role| role.as_str().to_owned());
	let bits = permissions.bits() as i64;
	if !store::update_member(pool, instance, target, &role_name, bits).await? {
		return Err(CommonError::NotFound(format!("member {target}")).into());
	}
	let row = MemberRow {
		user_id: target,
		username: current.username,
		role: role_name,
		permissions: bits,
	};
	Ok(web::Json(MemberView::member(row)))
}

#[utoipa::path(
	delete,
	path = "/api/v1/instances/{id}/members/{user_id}",
	tag = "members",
	params(
		("id" = i64, Path, description = "Instance id"),
		("user_id" = i64, Path, description = "Member user id"),
	),
	responses(
		(status = 204, description = "Member removed"),
		(status = 403, description = "Missing members.manage or removing a member with permissions not held"),
		(status = 404, description = "Instance or member not found"),
	)
)]
#[delete("/instances/{id}/members/{user_id}")]
pub async fn remove_member(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(InstanceId, UserId)>,
) -> ApiResult<HttpResponse> {
	let (instance, target) = path.into_inner();
	let pool = state.db.pool();
	let access = load(&state, &user, instance).await?;
	reject_owner(&access, target)?;
	let leaving = target == user.id;
	if !leaving {
		access.require(Permission::MEMBERS_MANAGE)?;
	}
	let current = store::member(pool, instance, target)
		.await?
		.ok_or_else(|| CommonError::NotFound(format!("member {target}")))?;
	if !leaving {
		access.can_grant(stored_permissions(&current))?;
	}
	if !store::delete_member(pool, instance, target).await? {
		return Err(CommonError::NotFound(format!("member {target}")).into());
	}
	Ok(HttpResponse::NoContent().finish())
}

fn reject_owner(access: &Access, target: UserId) -> ApiResult<()> {
	if target == access.owner.id {
		return Err(CommonError::Forbidden("the instance owner's access cannot be changed".into()).into());
	}
	Ok(())
}

fn stored_permissions(row: &MemberRow) -> Permission {
	Permission::from_bits_truncate(row.permissions as u32)
}

fn parse_role(name: &str) -> ApiResult<Role> {
	Role::parse(name).ok_or_else(|| CommonError::Invalid(format!("unknown role {name}")).into())
}

fn parse_permissions(names: &[String]) -> ApiResult<Permission> {
	names.iter().try_fold(Permission::empty(), |acc, name| {
		Permission::by_name(name)
			.map(|permission| acc | permission)
			.ok_or_else(|| CommonError::Invalid(format!("unknown permission {name}")).into())
	})
}
