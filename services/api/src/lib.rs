pub mod demo;

use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use axum::{
    extract::{DefaultBodyLimit, Request},
    http::{header, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use tower_governor::{
    errors::GovernorError, governor::GovernorConfigBuilder, key_extractor::KeyExtractor,
    GovernorLayer,
};
use tower_http::{
    compression::CompressionLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use crate::demo::{
    advance, advance_due, assign_exception, create_workspace, reset_workspace, resolve_exception,
    state as demo_state, undo_exception, DemoStore, Problem,
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

static HTTP_REQUESTS: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
struct TrustedProxyIpExtractor;

impl KeyExtractor for TrustedProxyIpExtractor {
    type Key = String;

    fn extract<T>(&self, request: &axum::http::Request<T>) -> Result<Self::Key, GovernorError> {
        request
            .headers()
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(',').next_back())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("local")
            .parse()
            .map_err(|_| GovernorError::UnableToExtractKey)
    }
}

pub fn app(build_sha: &'static str, dist_dir: impl Into<PathBuf>) -> Router {
    let state = Arc::new(AppState { build_sha });
    let demo_store = DemoStore::new();
    let api_governor = Arc::new(
        GovernorConfigBuilder::default()
            .per_millisecond(50)
            .burst_size(40)
            .key_extractor(TrustedProxyIpExtractor)
            .use_headers()
            .finish()
            .expect("valid public API rate limit"),
    );
    let api = Router::new()
        .route(
            "/v1/demo/workspaces",
            post(create_workspace).delete(reset_workspace),
        )
        .route("/v1/demo/state", get(demo_state))
        .route("/v1/demo/reminders/advance-due", post(advance_due))
        .route("/v1/demo/reminders/{id}/advance", post(advance))
        .route("/v1/demo/exceptions/{id}/assign", post(assign_exception))
        .route("/v1/demo/exceptions/{id}/resolve", post(resolve_exception))
        .route("/v1/demo/exceptions/{id}/undo", post(undo_exception))
        .with_state(demo_store)
        .layer(DefaultBodyLimit::max(16 * 1024))
        .layer(GovernorLayer::new(api_governor.clone()).error_handler(governor_error))
        .layer(middleware::from_fn(normalize_api_errors));
    let operations = Router::new()
        .route("/metrics", get(metrics))
        .layer(GovernorLayer::new(api_governor).error_handler(governor_error));

    let dist_dir = dist_dir.into();
    let spa = ServeFile::new(dist_dir.join("index.html"));
    let static_files = ServeDir::new(dist_dir).not_found_service(spa.clone());

    Router::new()
        .route("/health", get(health))
        .merge(operations)
        .nest("/api", api)
        .route_service("/", spa.clone())
        .route_service("/demo", spa.clone())
        .route_service("/demo/reminders/{*path}", spa.clone())
        .route_service("/privacy", spa.clone())
        .route_service("/terms", spa.clone())
        .route_service("/start", spa.clone())
        .route_service("/404", spa.clone())
        .fallback_service(static_files)
        .with_state(state)
        .layer(CompressionLayer::new())
        .layer(middleware::from_fn(cache_headers))
        .layer(middleware::from_fn(request_metrics))
        .layer(middleware::from_fn(security_headers))
        .layer(TraceLayer::new_for_http())
}

async fn metrics() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
        format!(
            "# HELP reminder_proof_http_requests_total HTTP requests handled.\n# TYPE reminder_proof_http_requests_total counter\nreminder_proof_http_requests_total {}\n",
            HTTP_REQUESTS.load(Ordering::Relaxed)
        ),
    )
}

fn governor_error(error: GovernorError) -> Response {
    let wait_time = match error {
        GovernorError::TooManyRequests { wait_time, .. } => wait_time.max(1),
        _ => 1,
    };
    let mut response = (
        StatusCode::TOO_MANY_REQUESTS,
        Json(Problem {
            code: "rate_limited",
            message: "Too many requests. Wait, then try again.",
            request_id: "available-in-response-header",
        }),
    )
        .into_response();
    response.headers_mut().insert(
        header::RETRY_AFTER,
        HeaderValue::from_str(&wait_time.to_string()).expect("valid retry-after"),
    );
    response
}

async fn request_metrics(request: Request, next: Next) -> Response {
    HTTP_REQUESTS.fetch_add(1, Ordering::Relaxed);
    next.run(request).await
}

async fn normalize_api_errors(request: Request, next: Next) -> Response {
    let response = next.run(request).await;
    if matches!(
        response.status(),
        StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY | StatusCode::PAYLOAD_TOO_LARGE
    ) && response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_none_or(|value| !value.starts_with("application/json"))
    {
        let (code, message) = if response.status() == StatusCode::PAYLOAD_TOO_LARGE {
            (
                "body_too_large",
                "The request body is too large. Send no more than 16 KB.",
            )
        } else {
            (
                "json_invalid",
                "The request is not valid JSON. Check the fields and try again.",
            )
        };
        let mut normalized = (
            response.status(),
            Json(Problem {
                code,
                message,
                request_id: "available-in-response-header",
            }),
        )
            .into_response();
        normalized
            .headers_mut()
            .insert("x-request-id", HeaderValue::from_static("local-request"));
        return normalized;
    }
    response
}

async fn cache_headers(request: Request, next: Next) -> Response {
    let immutable = request.uri().path().starts_with("/assets/");
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(if immutable {
            "public, max-age=31536000, immutable"
        } else {
            "no-cache"
        }),
    );
    response
}

async fn health(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> impl IntoResponse {
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
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'self'; base-uri 'self'; connect-src 'self'; font-src 'self'; frame-ancestors 'none'; img-src 'self' data:; object-src 'none'; script-src 'self'; style-src 'self'"),
    );
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static("camera=(), geolocation=(), microphone=()"),
    );
    headers.insert(
        "cross-origin-opener-policy",
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn test_app() -> Router {
        app("test-sha", "../../dist")
    }

    fn cookie(response: &Response) -> String {
        response
            .headers()
            .get(header::SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_owned()
    }

    #[tokio::test]
    async fn health_reports_build_identity_and_security_headers() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("x-content-type-options").unwrap(),
            "nosniff"
        );
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            std::str::from_utf8(&body).unwrap(),
            r#"{"status":"ok","build_sha":"test-sha"}"#
        );
    }

    #[tokio::test]
    async fn demo_rate_limit_returns_retry_after_from_x_forwarded_for() {
        let application = test_app();
        let mut last = StatusCode::OK;
        for attempt in 0..41 {
            let request = Request::builder()
                .method("GET")
                .uri("/api/v1/demo/state")
                .header(
                    "x-forwarded-for",
                    format!("198.51.100.{attempt}, 203.0.113.9"),
                )
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

    #[tokio::test]
    async fn workspace_allowance_uses_trusted_last_hop_and_has_retry_after() {
        let application = test_app();
        for attempt in 0..6 {
            let response = application
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/demo/workspaces")
                        .header(
                            "x-forwarded-for",
                            format!("198.51.100.{attempt}, 203.0.113.18"),
                        )
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            if attempt < 5 {
                assert_eq!(response.status(), StatusCode::OK);
            } else {
                assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
                assert!(response.headers().get(header::RETRY_AFTER).is_some());
            }
        }
    }

    #[tokio::test]
    async fn demo_state_survives_a_different_instance_and_secret() {
        let first_instance = app("first", "../../dist");
        let second_instance = app("second", "../../dist");
        let created = first_instance
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/demo/workspaces")
                    .header("x-forwarded-for", "203.0.113.40")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let created_cookie = cookie(&created);
        let advanced = first_instance
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/demo/reminders/mina/advance")
                    .header("x-forwarded-for", "203.0.113.40")
                    .header(header::COOKIE, created_cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let updated_cookie = cookie(&advanced);
        let response = second_instance
            .oneshot(
                Request::builder()
                    .uri("/api/v1/demo/state")
                    .header("x-forwarded-for", "203.0.113.40")
                    .header(header::COOKIE, updated_cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert!(std::str::from_utf8(&body)
            .unwrap()
            .contains("DELIVERED-200"));
    }

    #[tokio::test]
    async fn demo_write_allowance_returns_retry_after() {
        let application = test_app();
        let created = application
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/demo/workspaces")
                    .header("x-forwarded-for", "203.0.113.60")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let mut current_cookie = cookie(&created);
        for attempt in 0..31 {
            let response = application
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/demo/exceptions/sofia-exception/assign")
                        .header("x-forwarded-for", "203.0.113.60")
                        .header(header::COOKIE, &current_cookie)
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(r#"{"owner":"Sam Rivera"}"#))
                        .unwrap(),
                )
                .await
                .unwrap();
            if attempt < 30 {
                assert_eq!(response.status(), StatusCode::OK);
                current_cookie = cookie(&response);
            } else {
                assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
                assert!(response.headers().get(header::RETRY_AFTER).is_some());
            }
        }
    }

    #[tokio::test]
    async fn demo_write_body_is_capped() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/demo/exceptions/sofia-exception/assign")
                    .header("x-forwarded-for", "198.51.100.17")
                    .header("content-type", "application/json")
                    .header("content-length", "17000")
                    .body(Body::from(vec![b'x'; 17_000]))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );
    }

    #[tokio::test]
    async fn malformed_json_uses_the_problem_shape() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/demo/exceptions/sofia-exception/assign")
                    .header("x-forwarded-for", "203.0.113.41")
                    .header("content-type", "application/json")
                    .body(Body::from("{"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert!(std::str::from_utf8(&body).unwrap().contains("json_invalid"));
    }
}
