mod api;
mod health_check;
mod pages;

use crate::macros::router;
use axum::{
    routing::{get, post},
    Router,
};
use serde::Serialize;
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Serialize)]
struct ErrResponse {
    error: String,
}

pub fn register() -> Router<Arc<crate::AppState>> {
    router! {
        { "/hc",
            ("/", get(health_check::root)),
            ("/ws", get(health_check::ws_handler))
        }
        { "/api",
            ("/", post(api::root)),
            ("/item/{id}", post(api::item))
        }
        service! {
            ("/assets", ServeDir::new("assets"))
        }
        { // pages
            ("/", get(pages::root)),
            ("/movie/{id}", get(pages::movie))
            fallback! { ServeFile::new("assets/404.html") }
        }
    }
}

// Router::new()
//     .nest(
//         "/hc",
//         Router::new()
//             .route("/", get(health_check::root))
//             .route("/ws", get(health_check::ws_handler)),
//     )
//     .nest(
//         "/api",
//         Router::new().route("/", post(api::root)), // .route("/item/{id}", get(api::item)),
//     )
//     .nest_service("/assets", ServeDir::new("assets"))
//     .merge(
//         Router::new()
//             .route("/", get(pages::root))
//             .route("/movie/{id}", get(pages::movie))
//             .fallback_service(ServeFile::new("assets/404.html")),
//     )
