use std::path::PathBuf;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Clone)]
struct AppState {
    build_sha: &'static str,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    build_sha: &'static str,
}

pub fn app(build_sha: &'static str, dist_dir: impl Into<PathBuf>) -> Router {
    let dist_dir = dist_dir.into();
    let spa_fallback = ServeFile::new(dist_dir.join("index.html"));
    let static_files = ServeDir::new(dist_dir).not_found_service(spa_fallback);

    Router::new()
        .route("/health", get(health))
        .fallback_service(static_files)
        .with_state(AppState { build_sha })
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "ok",
            build_sha: state.build_sha,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_reports_build_identity() {
        let response = app("test-sha", "../../dist")
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            std::str::from_utf8(&body).unwrap(),
            r#"{"status":"ok","build_sha":"test-sha"}"#
        );
    }
}
