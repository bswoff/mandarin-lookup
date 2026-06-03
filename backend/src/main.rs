mod cedict;
mod db;
mod llm;
mod routes;

use std::sync::Arc;
use axum::{Router, routing::get};
use tower_http::cors::{CorsLayer, Any};
use tracing_subscriber::fmt;
use anyhow::Result;

pub struct AppState {
    pub db: db::Db,
    pub http: reqwest::Client,
    pub api_key: String,
    pub cedict: cedict::CedictIndex,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let file_appender = tracing_appender::rolling::never(".", "mandarin.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    fmt().with_writer(non_blocking).with_ansi(false).init();

    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY must be set in .env");

    let cedict_path = std::env::var("CEDICT_PATH")
        .unwrap_or_else(|_| "data/cedict.txt".to_string());
    tracing::info!("Loading CEDICT from {cedict_path}");
    let cedict = cedict::CedictIndex::load(&cedict_path)
        .expect("Failed to load cedict.txt — run from the backend/ directory");

    tracing::info!("Loading CEDICT finished");

    let db = db::Db::open("mandarin.db")?;
    let state = Arc::new(AppState {
        db,
        http: reqwest::Client::new(),
        api_key,
        cedict,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(routes::health::health))
        .route("/api/search", get(routes::search::search))
        .route(
            "/api/word/{pinyin}/{slug}",
            get(routes::word::get_word).put(routes::word::upsert_definition),
        )
        .route(
            "/api/word/{pinyin}/{slug}/regenerate",
            axum::routing::post(routes::word::regenerate),
        )
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    tracing::info!("Backend listening on http://localhost:3001");
    axum::serve(listener, app).await?;
    Ok(())
}
