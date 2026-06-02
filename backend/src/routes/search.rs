use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{cedict::SearchEntry, AppState};

#[derive(Deserialize)]
pub struct SearchParams {
    q: String,
}

pub async fn search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<SearchEntry>>, StatusCode> {
    let q = params.q.trim().to_string();
    if q.is_empty() {
        return Ok(Json(vec![]));
    }

    let is_pinyin = q.chars().all(|c| c.is_ascii_alphabetic() || c == ' ');
    let is_chinese = q.chars().all(is_cjk);

    if !is_pinyin && !is_chinese {
        return Err(StatusCode::BAD_REQUEST);
    }

    let entries = if is_pinyin {
        let base = normalize_pinyin(&q);
        tracing::info!(query = %q, normalized = %base, kind = "pinyin", "search request");
        state.cedict.lookup_by_pinyin(&base)
    } else {
        tracing::info!(query = %q, kind = "character", "search request");
        state.cedict.lookup_by_char(&q)
    };

    tracing::info!(query = %q, results = entries.len(), "search complete");
    Ok(Json(entries))
}

fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}'
        | '\u{3400}'..='\u{4DBF}'
        | '\u{F900}'..='\u{FAFF}'
        | '\u{20000}'..='\u{2A6DF}'
    )
}

pub fn strip_tones(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'ā' | 'á' | 'ǎ' | 'à' => 'a',
            'ē' | 'é' | 'ě' | 'è' => 'e',
            'ī' | 'í' | 'ǐ' | 'ì' => 'i',
            'ō' | 'ó' | 'ǒ' | 'ò' => 'o',
            'ū' | 'ú' | 'ǔ' | 'ù' => 'u',
            'ǖ' | 'ǘ' | 'ǚ' | 'ǜ' | 'ü' => 'u',
            _ => c,
        })
        .collect()
}

pub fn normalize_pinyin(s: &str) -> String {
    strip_tones(s).to_lowercase().replace(' ', "")
}
