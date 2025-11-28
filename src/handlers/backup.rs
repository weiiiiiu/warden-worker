use axum::{extract::State, Json};
use serde_json::json;
use std::sync::Arc;
use worker::Env;

use crate::auth::Claims;
use crate::backup as backup_service;
use crate::error::AppError;

#[worker::send]
pub async fn manual_backup(
    _claims: Claims,
    State(env): State<Arc<Env>>,
) -> Result<Json<serde_json::Value>, AppError> {
    match backup_service::backup_to_jianguoyun(&env).await {
        Ok(msg) => Ok(Json(json!({ "success": true, "message": msg }))),
        Err(e) => Ok(Json(json!({ "success": false, "error": format!("{:?}", e) }))),
    }
}
