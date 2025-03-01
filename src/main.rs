mod db;
mod routes;

use anyhow::Result;
use axum::{
    http::{header, Method},
    routing::{get, post},
    Router,
};
use sqlx::sqlite::SqlitePoolOptions;
use std::{env, sync::Arc};
use tokio;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

pub struct AppState {
    db: Arc<sqlx::SqlitePool>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        // .with_max_level(tracing::Level::DEBUG)
        .init();

    if env::var("INGEST_MOVIES").is_ok() {
        let ingest_client = db::ingest::IngestClient::new("sqlite:movies.db").await?;
        ingest_client.start().await?;
        println!("All data imports completed");
        return Ok(());
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:movies.db")
        .await?;
    db::init_tables(&pool).await?;
    let db = Arc::new(pool);

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE, header::ACCEPT, header::AUTHORIZATION]);

    let state = Arc::new(AppState { db });

    // build our application with a route
    let app = Router::new()
        .route("/api", post(routes::root))
        .route("/api/item/{id}", get(routes::item))
        .nest_service("/assets", ServeDir::new("assets"))
        .fallback_service(
            ServeDir::new("pages").not_found_service(ServeFile::new("pages/404.html")),
        )
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
