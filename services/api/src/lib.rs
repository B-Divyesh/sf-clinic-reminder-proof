pub mod demo;

use std::{path::PathBuf, sync::Arc};

use axum::{
    extract::Request,
    http::{header, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use tower_governor::{
    governor::GovernorConfigBuilder,
    key_extractor::SmartIpKeyExtractor,
    GovernorLayer,
};
use tower_http::{services::{ServeDir, ServeFile}, trace::TraceLayer};

use crate::demo::{
    advance, advance_due, assign_exception, create_workspace, reset_workspace, resolve_exception,
    state as demo_state, undo_exception, DemoStore,
};

#[derive(Clone)]
struct AppState {
    build_sha: &'static str,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    build_sha: &'static str,
}

pub fn app(build_sha: &'static str, dist_dir: impl Into<PathBuf>, secret: Vec<u8>) -> Router {
    let state = Arc::new(AppState { build_sha });
    let demo_store = DemoStore::new(secret);
    let api_governor = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(20)
            .burst_size(40)
            .key_extractor(SmartIpKeyExtractor)
            .use_headers()
            .finish()
            .expect("valid public API rate limit"),
    );
    let api = Router::new()
        .route("/v1/demo/workspaces", post(create_workspace).delete(reset_workspace))
        .route("/v1/demo/state", get(demo_state))
        .route("/v1/demo/reminders/advance-due", post(advance_due))
        .route("/v1/demo/reminders/{id}/advance", post(advance))
        .route("/v1/demo/exceptions/{id}/assign", post(assign_exception))
        .route("/v1/demo/exceptions/{id}/resolve", post(resolve_exception))
        .route("/v1/demo/exceptions/{id}/undo", post(undo_exception))
        .with_state(demo_store)
        .layer(GovernorLayer::new(api_governor));

    let dist_dir = dist_dir.into();
    let spa_fallback = ServeFile::new(dist_dir.join("index.html"));
    let static_files = ServeDir::new(dist_dir).not_found_service(spa_fallback);

    Router::new()
        .route("/health", get(health))
        .nest("/api", api)
        .fallback_service(static_files)
        .with_state(state)
        .layer(middleware::from_fn(security_headers))
        .layer(TraceLayer::new_for_http())
}

async fn health(axum::extract::State(state): axum::extract::State<Arc<AppState>>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "ok",
            build_sha: state.build_sha,
        }),
    )
}

async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(header::REFERRER_POLICY, HeaderValue::from_static("strict-origin-when-cross-origin"));
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'self'; base-uri 'self'; connect-src 'self'; font-src 'self'; frame-ancestors 'none'; img-src 'self' data:; object-src 'none'; script-src 'self'; style-src 'self'"),
    );
    headers.insert("permissions-policy", HeaderValue::from_static("camera=(), geolocation=(), microphone=()"));
    headers.insert("cross-origin-opener-policy", HeaderValue::from_static("same-origin"));
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn test_app() -> Router {
        app("test-sha", "../../dist", vec![3; 32])
    }

    #[tokio::test]
    async fn health_reports_build_identity_and_security_headers() {
        let response = test_app()
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("x-content-type-options").unwrap(), "nosniff");
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(std::str::from_utf8(&body).unwrap(), r#"{"status":"ok","build_sha":"test-sha"}"#);
    }

    #[tokio::test]
    async fn demo_rate_limit_returns_retry_after_from_x_forwarded_for() {
        let application = test_app();
        let mut last = StatusCode::OK;
        for _ in 0..41 {
            let request = Request::builder()
                .method("GET")
                .uri("/api/v1/demo/state")
                .header("x-forwarded-for", "198.51.100.14, 10.0.0.1")
                .body(Body::empty())
                .unwrap();
            let response = application.clone().oneshot(request).await.unwrap();
            last = response.status();
            if last == StatusCode::TOO_MANY_REQUESTS {
                assert!(response.headers().get(header::RETRY_AFTER).is_some());
                break;
            }
        }
        assert_eq!(last, StatusCode::TOO_MANY_REQUESTS);
    }
}
