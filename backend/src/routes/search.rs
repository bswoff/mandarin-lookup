use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    llm::{self, SenseCandidate},
    AppState,
};

#[derive(Deserialize)]
pub struct SearchParams {
    q: String,
}

pub async fn search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<SenseCandidate>>, StatusCode> {
    let q = params.q.trim().to_string();
    if q.is_empty() {
        return Ok(Json(vec![]));
    }

    if !q.chars().all(|c| c.is_ascii_alphabetic() || c == ' ') {
        return Err(StatusCode::BAD_REQUEST);
    }

    let pinyin_base = normalize_pinyin(&q);

    let cached = fetch_cached(&state.db, &pinyin_base).map_err(|e| {
        tracing::error!("DB read error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if !cached.is_empty() {
        tracing::info!(query = %q, "Tier 1 cache hit");
        return Ok(Json(cached));
    }

    tracing::info!("Search query: {:?} — cache miss, calling LLM", q);

    let raw_candidates = llm::fetch_tier1_senses(&state.http, &state.api_key, &q)
        .await
        .map_err(|e| {
            tracing::error!("LLM error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Drop any candidates whose pinyin doesn't match the query base syllable.
    // The LLM occasionally returns off-syllable results despite prompt instructions.
    let candidates: Vec<SenseCandidate> = raw_candidates
        .into_iter()
        .filter(|c| normalize_pinyin(&c.pinyin) == pinyin_base)
        .collect();

    let results = store_and_fetch(&state.db, &candidates, &pinyin_base).map_err(|e| {
        tracing::error!("DB write error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(results))
}

fn fetch_cached(db: &crate::db::Db, pinyin_base: &str) -> anyhow::Result<Vec<SenseCandidate>> {
    let conn = db.0.lock().unwrap();
    let mut stmt =
        conn.prepare("SELECT character, pinyin, sense_label, slug FROM senses")?;
    let rows = stmt
        .query_map([], |row| {
            Ok(SenseCandidate {
                character: row.get(0)?,
                pinyin: row.get(1)?,
                sense_label: row.get(2)?,
                slug: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .filter(|c| normalize_pinyin(&c.pinyin) == pinyin_base)
        .collect();
    Ok(rows)
}

fn store_and_fetch(
    db: &crate::db::Db,
    candidates: &[SenseCandidate],
    pinyin_base: &str,
) -> anyhow::Result<Vec<SenseCandidate>> {
    let conn = db.0.lock().unwrap();

    for c in candidates {
        let slug = unique_slug_for_pinyin(&conn, c)?;
        conn.execute(
            "INSERT OR IGNORE INTO senses (character, pinyin, sense_label, slug) \
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![c.character, c.pinyin, c.sense_label, slug],
        )?;
    }

    // Return what is actually stored so the caller always sees slugs consistent
    // with the DB primary keys, not whatever the LLM returned.
    let mut stmt =
        conn.prepare("SELECT character, pinyin, sense_label, slug FROM senses")?;
    let results = stmt
        .query_map([], |row| {
            Ok(SenseCandidate {
                character: row.get(0)?,
                pinyin: row.get(1)?,
                sense_label: row.get(2)?,
                slug: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .filter(|c| normalize_pinyin(&c.pinyin) == pinyin_base)
        .collect();

    Ok(results)
}

// Returns a slug that is unique among rows where normalize_pinyin(pinyin) == the
// candidate's pinyin base, so that (pinyin_base, slug) resolves to exactly one character.
fn unique_slug_for_pinyin(
    conn: &Connection,
    candidate: &SenseCandidate,
) -> anyhow::Result<String> {
    let pinyin_base = normalize_pinyin(&candidate.pinyin);
    let base_slug = candidate.slug.clone();

    if !slug_taken_at_pinyin_base(conn, &candidate.character, &pinyin_base, &base_slug)? {
        return Ok(base_slug);
    }

    let mut i = 2u32;
    loop {
        let s = format!("{}-{}", base_slug, i);
        if !slug_taken_at_pinyin_base(conn, &candidate.character, &pinyin_base, &s)? {
            return Ok(s);
        }
        i += 1;
    }
}

fn slug_taken_at_pinyin_base(
    conn: &Connection,
    character: &str,
    pinyin_base: &str,
    slug: &str,
) -> anyhow::Result<bool> {
    let mut stmt =
        conn.prepare("SELECT pinyin FROM senses WHERE character != ?1 AND slug = ?2")?;
    let taken = stmt
        .query_map(rusqlite::params![character, slug], |row| {
            row.get::<_, String>(0)
        })?
        .filter_map(|r| r.ok())
        .any(|pinyin| normalize_pinyin(&pinyin) == pinyin_base);
    Ok(taken)
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
    strip_tones(s).replace(' ', "")
}
