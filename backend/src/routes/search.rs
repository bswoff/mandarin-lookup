// Input classification strategy
//
// Both English and pinyin are plain ASCII, so distinguishing them is non-trivial.
// Three approaches were considered:
//
// 1. Syllable whitelist — parse input against the ~400 valid Mandarin syllables; if it
//    fully decomposes, treat as pinyin, otherwise English.
//    Pro: deterministic, single backend call.
//    Con: requires maintaining the syllable list; short words that are coincidentally
//    valid syllables ("he", "an", "man") always route to pinyin.
//
// 2. Try pinyin first, fall back to English (chosen) — attempt the pinyin lookup; if it
//    returns no results, retry as an English definition search.
//    Pro: simplest to implement; handles ~95% of real queries correctly.
//    Con: same edge-case overlap as #1 — "man" has valid pinyin hits so the English
//    fallback never fires for it. Acceptable because those words are poor dictionary
//    search terms in practice.
//
// 3. Dual search — run both lookups in parallel and merge results, pinyin first.
//    Pro: most correct; no classification needed; handles every ambiguous input.
//    Con: every query pays two lookup costs; result display needs visual separation
//    between pinyin and English hits to avoid noise.

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
        let pinyin_results = state.cedict.lookup_by_pinyin(&base);
        if pinyin_results.is_empty() {
            tracing::info!(query = %q, kind = "english-fallback", "search request");
            state.cedict.lookup_by_english(&q)
        } else {
            pinyin_results
        }
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
