mod db;
mod routes;

use std::sync::Arc;
use axum::{Router, routing::get};
use tower_http::cors::{CorsLayer, Any};
use anyhow::Result;

pub struct AppState {
    pub db: db::Db,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let db = db::Db::open("mandarin.db")?;
    let state = Arc::new(AppState { db });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(routes::health::health))
        .route("/api/search", get(routes::search::search))
        .route("/api/word/{character}/{slug}", get(routes::word::get_word))
        .route(
            "/api/word/{character}/{slug}/regenerate",
            axum::routing::post(routes::word::regenerate),
        )
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    println!("Backend listening on http://localhost:3001");
    axum::serve(listener, app).await?;
    Ok(())
}
