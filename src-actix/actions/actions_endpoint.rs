use crate::actions::actions_data::{ActionData, CreateActionRequest, UpdateActionRequest};
use crate::authentication::auth_data::UserData;
use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
use serde_json::json;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/actions")
            .service(get_actions)
            .service(get_active_actions)
            .service(create_action)
            .service(update_action)
            .service(delete_completed_actions)
            .service(delete_action),
    );
}

#[get("")]
pub async fn get_actions(user: web::ReqData<UserData>) -> impl Responder {
    let user_id = match user.id {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Invalid user ID"
            }));
        }
    };

    match ActionData::get_by_user_id(user_id as i64).await {
        Ok(actions) => HttpResponse::Ok().json(actions),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to fetch actions: {}", e)
        })),
    }
}

#[get("/active")]
pub async fn get_active_actions(user: web::ReqData<UserData>) -> impl Responder {
    let user_id = match user.id {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Invalid user ID"
            }));
        }
    };

    match ActionData::get_active_by_user_id(user_id as i64).await {
        Ok(actions) => HttpResponse::Ok().json(actions),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to fetch active actions: {}", e)
        })),
    }
}

#[post("")]
pub async fn create_action(user: web::ReqData<UserData>, req: web::Json<CreateActionRequest>) -> impl Responder {
    let user_id = match user.id {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Invalid user ID"
            }));
        }
    };

    let details = req.details.as_ref().map(|details| details.to_string());

    match ActionData::create(user_id as i64, req.tracker_id.clone(), req.action_type.clone(), details).await {
        Ok(action) => HttpResponse::Created().json(action),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to create action: {}", e)
        })),
    }
}

#[put("/{tracker_id}")]
pub async fn update_action(user: web::ReqData<UserData>, path: web::Path<String>, req: web::Json<UpdateActionRequest>) -> impl Responder {
    let tracker_id = path.into_inner();

    let action = match ActionData::get_by_tracker_id(&tracker_id).await {
        Ok(Some(action)) => action,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "error": "Action not found"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to fetch action: {}", e)
            }));
        }
    };

    // Verify user owns this action
    let user_id = match user.id {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Invalid user ID"
            }));
        }
    };

    if action.user_id != user_id as i64 {
        return HttpResponse::Forbidden().json(json!({
            "error": "Access denied"
        }));
    }

    // Update progress if provided
    if let Some(progress) = req.progress
        && let Err(e) = action.update_progress(progress).await
    {
        return HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to update progress: {}", e)
        }));
    }

    // Update status if provided
    if let Some(status) = &req.status {
        let details = req.details.as_ref().map(|details| details.to_string());

        if let Err(e) = action.update_status(status.clone(), details).await {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to update status: {}", e)
            }));
        }
    }

    // Fetch updated action
    match ActionData::get_by_tracker_id(&tracker_id).await {
        Ok(Some(updated_action)) => HttpResponse::Ok().json(updated_action),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "error": "Action not found after update"
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to fetch updated action: {}", e)
        })),
    }
}

#[delete("/completed")]
pub async fn delete_completed_actions(user: web::ReqData<UserData>) -> impl Responder {
    let user_id = match user.id {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Invalid user ID"
            }));
        }
    };

    match ActionData::delete_completed_by_user_id(user_id as i64).await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "message": "Completed actions deleted successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to delete completed actions: {}", e)
        })),
    }
}

#[delete("/{tracker_id}")]
pub async fn delete_action(user: web::ReqData<UserData>, path: web::Path<String>) -> impl Responder {
    let tracker_id = path.into_inner();

    // Verify action exists and user owns it
    let action = match ActionData::get_by_tracker_id(&tracker_id).await {
        Ok(Some(action)) => action,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "error": "Action not found"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to fetch action: {}", e)
            }));
        }
    };

    let user_id = match user.id {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Invalid user ID"
            }));
        }
    };

    if action.user_id != user_id as i64 {
        return HttpResponse::Forbidden().json(json!({
            "error": "Access denied"
        }));
    }

    match ActionData::delete_by_tracker_id(&tracker_id).await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "message": "Action deleted successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to delete action: {}", e)
        })),
    }
}
