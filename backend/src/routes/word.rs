use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::Value;
use std::sync::Arc;

use crate::{llm, AppState};

pub async fn get_word(
    State(state): State<Arc<AppState>>,
    Path((pinyin, slug)): Path<(String, String)>,
) -> Result<Json<Value>, StatusCode> {
    let (character, definition_text) = state
        .cedict
        .resolve_entry(&pinyin, &slug)
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(cached) = state.db.get_definition(&character, &slug) {
        let value: Value = serde_json::from_str(&cached).map_err(|e| {
            tracing::error!("Failed to parse cached JSON for ({}, {}): {}", character, slug, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        return Ok(Json(value));
    }

    tracing::info!(character = %character, slug = %slug, "cache miss — calling LLM");
    let detail = llm::generate_word_detail(&state.http, &state.api_key, &character, &definition_text)
        .await
        .map_err(|e| {
            tracing::error!("LLM error for ({}, {}): {}", character, slug, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let json_str = serde_json::to_string(&detail).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.db.set_definition(&character, &slug, &json_str).map_err(|e| {
        tracing::error!("DB write error for ({}, {}): {}", character, slug, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let value: Value = serde_json::from_str(&json_str).unwrap();
    Ok(Json(value))
}

pub async fn regenerate(
    State(state): State<Arc<AppState>>,
    Path((pinyin, slug)): Path<(String, String)>,
) -> Result<Json<Value>, StatusCode> {
    let (character, definition_text) = state
        .cedict
        .resolve_entry(&pinyin, &slug)
        .ok_or(StatusCode::NOT_FOUND)?;

    state.db.delete_definition(&character, &slug).map_err(|e| {
        tracing::error!("DB delete error for ({}, {}): {}", character, slug, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!(character = %character, slug = %slug, "regenerating — calling LLM");
    let detail = llm::generate_word_detail(&state.http, &state.api_key, &character, &definition_text)
        .await
        .map_err(|e| {
            tracing::error!("LLM error for ({}, {}): {}", character, slug, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let json_str = serde_json::to_string(&detail).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.db.set_definition(&character, &slug, &json_str).map_err(|e| {
        tracing::error!("DB write error for ({}, {}): {}", character, slug, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let value: Value = serde_json::from_str(&json_str).unwrap();
    Ok(Json(value))
}
