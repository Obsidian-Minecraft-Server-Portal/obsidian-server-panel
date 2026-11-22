use actix_web::{HttpResponse, delete, get, patch, post, web};
use futures_util::future::try_join_all;
use obsidian_auth::{AuthUser as User, Permission};
use obsidian_common::{CommonError, InstanceId};
use obsidian_instances::{CreateInstance, Instance, UpdateInstance};

use super::dto::InstanceView;
use crate::auth::AuthUser;
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

async fn access(state: &AppState, user: &User, id: InstanceId, required: Permission) -> ApiResult<Permission> {
	let granted = state.auth.instance_permission(user, id).await?;
	if granted.is_empty() {
		return Err(CommonError::Forbidden("no access to this instance".into()).into());
	}
	if !granted.contains(required) {
		return Err(CommonError::Forbidden(format!("missing permission {}", required.names().join(", "))).into());
	}
	Ok(granted)
}

async fn view(state: &AppState, instance: Instance, permissions: Permission) -> ApiResult<InstanceView> {
	let status = state.instances.status(instance.id).await?;
	Ok(InstanceView { instance, status, permissions: permissions.names() })
}

async fn view_for(state: &AppState, user: &User, instance: Instance) -> ApiResult<InstanceView> {
	let permissions = state.auth.instance_permission(user, instance.id).await?;
	view(state, instance, permissions).await
}

#[utoipa::path(
	get,
	path = "/api/v1/instances",
	tag = "instances",
	responses((status = 200, description = "Instances the caller can access, with the caller's permissions", body = [InstanceView]))
)]
#[get("/instances")]
pub async fn list_instances(state: web::Data<AppState>, user: AuthUser) -> ApiResult<HttpResponse> {
	let instances = state.instances.list_for(user.id).await?;
	let views = try_join_all(instances.into_iter().map(|instance| view_for(&state, &user, instance))).await?;
	Ok(HttpResponse::Ok().json(views))
}

#[utoipa::path(
	post,
	path = "/api/v1/instances",
	tag = "instances",
	request_body = CreateInstance,
	responses(
		(status = 201, description = "Instance created, installation runs as a task", body = InstanceView),
		(status = 403, description = "Instance limit reached"),
	)
)]
#[post("/instances")]
pub async fn create_instance(
	state: web::Data<AppState>,
	user: AuthUser,
	body: web::Json<CreateInstance>,
) -> ApiResult<HttpResponse> {
	let instance = state.instances.create(body.into_inner(), user.id).await?;
	Ok(HttpResponse::Created().json(view(&state, instance, Permission::all()).await?))
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}",
	tag = "instances",
	responses(
		(status = 200, description = "Instance with its live status and the caller's permissions", body = InstanceView),
		(status = 403, description = "The caller has no access to this instance"),
	)
)]
#[get("/instances/{id}")]
pub async fn get_instance(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	let permissions = access(&state, &user, id, Permission::empty()).await?;
	Ok(HttpResponse::Ok().json(view(&state, state.instances.get(id).await?, permissions).await?))
}

#[utoipa::path(
	patch,
	path = "/api/v1/instances/{id}",
	tag = "instances",
	request_body = UpdateInstance,
	responses((status = 200, description = "Updated instance", body = InstanceView))
)]
#[patch("/instances/{id}")]
pub async fn update_instance(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
	body: web::Json<UpdateInstance>,
) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	let permissions = access(&state, &user, id, Permission::SETTINGS_EDIT).await?;
	let instance = state.instances.update(id, body.into_inner()).await?;
	Ok(HttpResponse::Ok().json(view(&state, instance, permissions).await?))
}

#[utoipa::path(
	delete,
	path = "/api/v1/instances/{id}",
	tag = "instances",
	responses(
		(status = 204, description = "Instance and its files deleted"),
		(status = 403, description = "Only the owner or an administrator may delete"),
		(status = 409, description = "The server is running or installing"),
	)
)]
#[delete("/instances/{id}")]
pub async fn delete_instance(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	let instance = state.instances.get(id).await?;
	if instance.owner_id != user.id && !user.is_admin {
		return Err(ApiError::from(CommonError::Forbidden("only the owner can delete this server".into())));
	}
	state.instances.delete(id).await?;
	Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/install",
	tag = "instances",
	responses(
		(status = 202, description = "Installation restarted as an 'instance.install' task", body = InstanceView),
		(status = 409, description = "The server is running or already installing"),
	)
)]
#[post("/instances/{id}/install")]
pub async fn reinstall_instance(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	let permissions = access(&state, &user, id, Permission::SETTINGS_EDIT).await?;
	state.instances.reinstall(id).await?;
	Ok(HttpResponse::Accepted().json(view(&state, state.instances.get(id).await?, permissions).await?))
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct NextPort {
	pub port: u16,
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/next-port",
	tag = "instances",
	responses((status = 200, description = "The port a new instance gets when none is requested", body = NextPort))
)]
#[get("/instances/next-port")]
pub async fn next_port(_user: AuthUser, state: web::Data<AppState>) -> ApiResult<HttpResponse> {
	let port = state.instances.next_free_port().await?;
	Ok(HttpResponse::Ok().json(NextPort { port }))
}
