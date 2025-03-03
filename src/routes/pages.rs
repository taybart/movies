use crate::{
    db::movie,
    macros::{page, res},
    routes::ErrResponse,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use std::sync::Arc;
use tracing::{error, info};

pub async fn root(State(state): State<Arc<crate::AppState>>) -> impl IntoResponse {
    page!(state, "index.html")
}

pub async fn movie(
    State(state): State<Arc<crate::AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!("request {id:?}");
    let movie = res!(
        movie::get(&state.db, id).await,
        (
            StatusCode::NOT_FOUND,
            Json(ErrResponse {
                error: "not found".into(),
            })
            .into_response(),
        )
    );

    page!(state, "movie.html", movie)
}
