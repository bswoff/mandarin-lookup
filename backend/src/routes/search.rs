use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::AppState;

pub async fn search(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Value>, StatusCode> {
    // Stub — implemented in Phase 2
    Ok(Json(json!([])))
}
