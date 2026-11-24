use actix_web::{post, web};
use obsidian_auth::Permission;
use obsidian_content::MatchSummary;

use crate::auth::{AuthUser, require_instance_perm};
use crate::error::ApiResult;
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(find_content_matches);
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/content/matches",
	tag = "content",
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 200, description = "Unlinked files were matched against Modrinth, then CurseForge", body = MatchSummary),
		(status = 403, description = "Missing content.install permission"),
	)
)]
#[post("/instances/{id}/content/matches")]
pub async fn find_content_matches(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<i64>,
) -> ApiResult<web::Json<MatchSummary>> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONTENT_INSTALL).await?;
	Ok(web::Json(state.content.find_matches(id, Some(user.id)).await?))
}
