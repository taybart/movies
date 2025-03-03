mod db;
mod macros;
mod routes;

use anyhow::Result;
use axum::http::Method;
use sqlx::sqlite::SqlitePoolOptions;
use std::{env, net::SocketAddr, sync::Arc};
use tera::Tera;
use tokio;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::{error, info};

pub struct AppState {
    db: Arc<sqlx::SqlitePool>,
    tera: Arc<Tera>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        // .with_max_level(tracing::Level::DEBUG)
        .init();

    if env::var("INGEST_MOVIES").is_ok() {
        let ingest_client = db::ingest::IngestClient::new("sqlite:movies.db").await?;
        ingest_client.start().await?;
        info!("All data imports completed");
        return Ok(());
    }

    let tera = match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            error!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    let tera = Arc::new(tera);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:movies.db")
        .await?;
    db::init_tables(&pool).await?;
    let db = Arc::new(pool);

    let cors = CorsLayer::new().allow_methods([Method::GET, Method::POST]);
    // .allow_headers([header::CONTENT_TYPE, header::ACCEPT, header::AUTHORIZATION]);

    // build our application with a route
    let app = routes::register()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .layer(cors)
        .with_state(Arc::new(AppState { db, tera }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();

    Ok(())
}
