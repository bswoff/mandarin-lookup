use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::Value;
use std::sync::Arc;

use crate::AppState;

pub async fn get_word(
    State(_state): State<Arc<AppState>>,
    Path((_character, _slug)): Path<(String, String)>,
) -> Result<Json<Value>, StatusCode> {
    // Stub — implemented in Phase 3
    Err(StatusCode::NOT_FOUND)
}

pub async fn regenerate(
    State(_state): State<Arc<AppState>>,
    Path((_character, _slug)): Path<(String, String)>,
) -> Result<Json<Value>, StatusCode> {
    // Stub — implemented in Phase 3
    Err(StatusCode::NOT_FOUND)
}
