use crate::db::{movie, titles};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

#[derive(Debug, Deserialize)]
pub struct Request {
    title: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    title_type: String,
    year: Option<i64>, // TODO: check js
}

#[derive(Serialize)]
struct ErrResponse {
    error: String,
}

pub async fn root(
    State(state): State<Arc<crate::AppState>>,
    Json(req): Json<Request>,
) -> impl IntoResponse {
    info!("request {req:?}");
    let Ok(titles) = titles::TitleQuery::new()
        .like(req.title)
        .title_type(req.title_type)
        .start_year(req.year)
        .fetch(&state.db)
        .await
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(ErrResponse {
                error: "not found".into(),
            })
            .into_response(),
        );
    };
    (StatusCode::OK, Json(titles).into_response())
}

pub async fn item(
    State(state): State<Arc<crate::AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!("request {id:?}");
    let movie = match movie::get(&state.db, id).await {
        Ok(m) => m,
        Err(e) => {
            error!("{e}");
            return (
                StatusCode::NOT_FOUND,
                Json(ErrResponse {
                    error: "not found".into(),
                })
                .into_response(),
            );
        }
    };
    (StatusCode::OK, Json(movie).into_response())
}
