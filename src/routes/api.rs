use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, info};

use crate::{
    db::{movie, titles},
    macros::res,
    routes::ErrResponse,
};

#[derive(Debug, Deserialize)]
pub struct Request {
    title: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    title_type: String,
    year: Option<i64>, // TODO: check js
}
pub async fn root(
    State(state): State<Arc<crate::AppState>>,
    Json(req): Json<Request>,
) -> impl IntoResponse {
    info!("request {req:?}");
    let titles = res!(
        titles::TitleQuery::new()
            .like(req.title)
            .title_type(req.title_type)
            .start_year(req.year)
            .limit(100)
            .fetch(&state.db)
            .await,
        (
            StatusCode::NOT_FOUND,
            Json(ErrResponse {
                error: "not found".into(),
            })
            .into_response(),
        )
    );
    (StatusCode::OK, Json(titles).into_response())
}

pub async fn item(
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

    (StatusCode::OK, Json(movie).into_response())
}
