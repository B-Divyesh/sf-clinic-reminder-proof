pub mod auth;
pub mod clinic;
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
    limit::RequestBodyLimitLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use crate::clinic::ClinicState;
use crate::demo::{
    advance, advance_due, assign_exception, create_workspace, problem_response, reset_workspace,
    resolve_exception, state as demo_state, undo_exception, DemoStore,
};

const API_BODY_LIMIT: usize = 16 * 1024;

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
            .and_then(|value| value.split(',').next())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("local")
            .parse()
            .map_err(|_| GovernorError::UnableToExtractKey)
    }
}

pub fn app(build_sha: &'static str, dist_dir: impl Into<PathBuf>) -> Router {
    let clinic_state = ClinicState::from_env().expect("initialize durable clinic store");
    app_with_clinic_state(build_sha, dist_dir, clinic_state)
}

fn app_with_clinic_state(
    build_sha: &'static str,
    dist_dir: impl Into<PathBuf>,
    clinic_state: ClinicState,
) -> Router {
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
    let billing_governor = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(120)
            .burst_size(5)
            .key_extractor(TrustedProxyIpExtractor)
            .use_headers()
            .finish()
            .expect("valid billing API rate limit"),
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
        .layer(DefaultBodyLimit::max(API_BODY_LIMIT))
        .layer(RequestBodyLimitLayer::new(API_BODY_LIMIT))
        .layer(GovernorLayer::new(api_governor.clone()).error_handler(governor_error))
        .layer(middleware::from_fn(normalize_api_errors));
    let billing_state = clinic_state.clone();
    let clinic_api = Router::new()
        .route("/v1/auth/config", get(clinic::auth_config))
        .route("/v1/me", get(clinic::get_me))
        .route(
            "/v1/organizations",
            get(clinic::list_organizations).post(clinic::onboard),
        )
        .route(
            "/v1/locations",
            get(clinic::list_locations).post(clinic::update_location),
        )
        .route(
            "/v1/memberships",
            get(clinic::list_memberships).post(clinic::add_membership),
        )
        .route("/v1/exports", get(clinic::export_workspace))
        .route(
            "/v1/account-deletion",
            post(clinic::schedule_account_deletion).delete(clinic::cancel_account_deletion),
        )
        .route(
            "/v1/clinic",
            get(clinic::get_workspace)
                .post(clinic::onboard)
                .delete(clinic::delete_workspace),
        )
        .route("/v1/clinic/export", get(clinic::export_workspace))
        .route("/v1/clinic/connectors", post(clinic::configure_connector))
        .route("/v1/clinic/providers", post(clinic::configure_provider))
        .route("/v1/clinic/reminders/dispatch", post(clinic::dispatch))
        .route(
            "/v1/clinic/exceptions/{id}/assign",
            post(clinic::assign_exception),
        )
        .route(
            "/v1/clinic/exceptions/{id}/resolve",
            post(clinic::resolve_exception),
        )
        .route("/v1/connectors/intake", post(clinic::connector_intake))
        .route(
            "/v1/providers/{id}/receipts",
            post(clinic::provider_receipt),
        )
        .route(
            "/v1/providers/twilio/{id}/receipts",
            post(clinic::twilio_receipt),
        )
        .route(
            "/v1/providers/resend/{id}/receipts",
            post(clinic::resend_receipt),
        )
        .with_state(clinic_state)
        .layer(DefaultBodyLimit::max(API_BODY_LIMIT))
        .layer(RequestBodyLimitLayer::new(API_BODY_LIMIT))
        .layer(GovernorLayer::new(api_governor.clone()).error_handler(governor_error))
        .layer(middleware::from_fn(normalize_api_errors));
    let billing_api = Router::new()
        .route("/v1/billing/checkout", post(clinic::billing_checkout))
        .route("/v1/billing/return", post(clinic::billing_return))
        .with_state(billing_state)
        .layer(DefaultBodyLimit::max(API_BODY_LIMIT))
        .layer(RequestBodyLimitLayer::new(API_BODY_LIMIT))
        .layer(GovernorLayer::new(billing_governor).error_handler(governor_error))
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
        .nest("/api", clinic_api)
        .nest("/api", billing_api)
        .route_service("/", spa.clone())
        .route_service("/demo", spa.clone())
        .route_service("/demo/reminders/{*path}", spa.clone())
        .route_service("/privacy", spa.clone())
        .route_service("/terms", spa.clone())
        .route_service("/start", spa.clone())
        .route_service("/sign-in", spa.clone())
        .route_service("/onboarding/clinic", spa.clone())
        .route_service("/onboarding/location", spa.clone())
        .route_service("/onboarding/staff", spa.clone())
        .route_service("/app", spa.clone())
        .route_service("/app/settings/members", spa.clone())
        .route_service("/app/settings/billing", spa.clone())
        .route_service("/app/settings/privacy", spa.clone())
        .route_service("/auth/callback", spa.clone())
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
    let mut response = problem_response(
        StatusCode::TOO_MANY_REQUESTS,
        "rate_limited",
        "Too many requests. Wait, then try again.",
    );
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
        StatusCode::BAD_REQUEST
            | StatusCode::UNSUPPORTED_MEDIA_TYPE
            | StatusCode::UNPROCESSABLE_ENTITY
            | StatusCode::PAYLOAD_TOO_LARGE
    ) && response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_none_or(|value| !value.starts_with("application/json"))
    {
        let (code, message) = match response.status() {
            StatusCode::PAYLOAD_TOO_LARGE => (
                "body_too_large",
                "The request body is too large. Send no more than 16 KB.",
            ),
            StatusCode::UNSUPPORTED_MEDIA_TYPE => (
                "content_type_invalid",
                "Send this request as application/json, then try again.",
            ),
            _ => (
                "json_invalid",
                "The request is not valid JSON. Check the fields and try again.",
            ),
        };
        return problem_response(response.status(), code, message);
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
        HeaderValue::from_static("default-src 'self'; base-uri 'self'; connect-src 'self' https://sociobotcustomers.ciamlogin.com https://api.sociobot.in; font-src 'self'; frame-ancestors 'none'; frame-src https://sociobotcustomers.ciamlogin.com; img-src 'self' data:; object-src 'none'; script-src 'self'; style-src 'self'"),
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
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use hmac::{Hmac, Mac};
    use http_body_util::BodyExt;
    use sha2::Sha256;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tower::ServiceExt;
    use uuid::Uuid;

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

    async fn assert_problem(
        response: Response,
        expected_status: StatusCode,
        expected_code: &str,
    ) -> String {
        assert_eq!(response.status(), expected_status);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );
        let header_id = response
            .headers()
            .get("x-request-id")
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        Uuid::parse_str(&header_id).expect("request ID is a UUID");
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let problem: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(problem["code"], expected_code);
        assert_eq!(problem["request_id"], header_id);
        header_id
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
                    format!("198.51.100.9, 203.0.113.{attempt}"),
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
    async fn workspace_allowance_uses_client_first_hop_and_has_retry_after() {
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
                            format!("198.51.100.18, 203.0.113.{attempt}"),
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
    async fn m2_billing_start_has_the_stricter_allowance() {
        let application = test_app();
        for attempt in 0..6 {
            let response = application
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/billing/checkout")
                        .header("x-forwarded-for", "198.51.100.219")
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(r#"{"tier":"clinic"}"#))
                        .unwrap(),
                )
                .await
                .unwrap();
            if attempt < 5 {
                assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
            } else {
                assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
                assert!(response.headers().get(header::RETRY_AFTER).is_some());
            }
        }
    }

    #[test]
    fn rate_limit_state_has_one_production_topology_owner() {
        let deployment: serde_json::Value =
            serde_json::from_str(include_str!("../../../deployment/containerapp.json")).unwrap();
        assert_eq!(
            deployment["properties"]["template"]["scale"]["maxReplicas"], 1,
            "the in-process rate limiter is safe only with one production replica"
        );
        assert_eq!(
            deployment["properties"]["template"]["volumes"][0]["storageName"],
            "clinic-reminder-proof-data"
        );
        assert_eq!(
            deployment["properties"]["template"]["volumes"][1]["storageName"],
            "clinic-reminder-proof-backups"
        );
        assert!(deployment["properties"]["template"]
            .get("initContainers")
            .is_none());
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

    #[tokio::test]
    async fn every_json_write_boundary_and_auth_error_has_a_correlatable_request_id() {
        let application = test_app();
        let wrong_content_type = application
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/billing/checkout")
                    .header("x-forwarded-for", "198.51.100.71")
                    .header(header::CONTENT_TYPE, "text/plain")
                    .body(Body::from(r#"{"tier":"clinic"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let content_type_id = assert_problem(
            wrong_content_type,
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "content_type_invalid",
        )
        .await;

        let oversized = application
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/billing/checkout")
                    .header("x-forwarded-for", "198.51.100.72")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::CONTENT_LENGTH, "17027")
                    .body(Body::from(format!(
                        r#"{{"tier":"clinic","padding":"{}"}}"#,
                        "x".repeat(17_000)
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        let oversized_id =
            assert_problem(oversized, StatusCode::PAYLOAD_TOO_LARGE, "body_too_large").await;

        let unauthorized = application
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/billing/checkout")
                    .header("x-forwarded-for", "198.51.100.73")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"tier":"clinic"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            unauthorized
                .headers()
                .get(header::WWW_AUTHENTICATE)
                .unwrap(),
            "Bearer"
        );
        let unauthorized_id =
            assert_problem(unauthorized, StatusCode::UNAUTHORIZED, "bearer_required").await;

        assert_ne!(content_type_id, oversized_id);
        assert_ne!(oversized_id, unauthorized_id);
    }

    #[tokio::test]
    async fn managed_claim_clinic_flow_is_authenticated_signed_durable_and_consent_aware() {
        let path = std::env::temp_dir().join(format!("reminder-proof-api-{}", Uuid::new_v4()));
        let clinic_state = ClinicState::for_tests(path.clone()).unwrap();
        let application = app_with_clinic_state("managed-test", "../../dist", clinic_state);

        let unauthorized = application
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/clinic")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            unauthorized
                .headers()
                .get(header::WWW_AUTHENTICATE)
                .unwrap(),
            "Bearer"
        );

        let created = application.clone().oneshot(Request::builder().method("POST").uri("/api/v1/clinic").header(header::AUTHORIZATION, "Bearer test:clinic-owner-a").header(header::CONTENT_TYPE, "application/json").body(Body::from(r#"{"clinic_name":"Oak Street Dental","location_name":"High Street","timezone":"Europe/London"}"#)).unwrap()).await.unwrap();
        assert_eq!(created.status(), StatusCode::CREATED);

        let secret = "calendar-signing-secret-123";
        let connector = application
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/clinic/connectors")
                    .header(header::AUTHORIZATION, "Bearer test:clinic-owner-a")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(format!(
                        r#"{{"kind":"signed-calendar-webhook","webhook_secret":"{secret}"}}"#
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(connector.status(), StatusCode::CREATED);
        let connector_body = connector.into_body().collect().await.unwrap().to_bytes();
        let connector_json: serde_json::Value = serde_json::from_slice(&connector_body).unwrap();
        let connector_id = connector_json["connector"]["id"].as_str().unwrap();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let canonical = format!("{timestamp}:{connector_id}:2");
        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(canonical.as_bytes());
        let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
        let intake_body = serde_json::json!({
            "connector_id": connector_id,
            "appointments": [
                {
                    "source_id": "emr-appointment-42",
                    "patient_alias": "Patient A",
                    "first_name": "A",
                    "appointment_time": "2026-09-01T09:00:00+01:00",
                    "channels": [
                        {"channel":"sms","destination":"+447700900001","consent":"blocked","consent_source":"EMR opt-out","consent_captured_at":"2026-08-27T12:00:00Z"},
                        {"channel":"email","destination":"patient@example.test","consent":"unknown","consent_source":"no record","consent_captured_at":"2026-08-27T12:00:00Z"}
                    ]
                },
                {
                    "source_id": "emr-appointment-43",
                    "patient_alias": "Patient B",
                    "first_name": "B",
                    "appointment_time": "2026-09-01T10:00:00+01:00",
                    "channels": [
                        {"channel":"sms","destination":"+447700900002","consent":"allowed","consent_source":"EMR opt-in","consent_captured_at":"2026-08-27T12:00:00Z"}
                    ]
                }
            ]
        });
        let intake = application
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/connectors/intake")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("x-reminder-timestamp", timestamp.to_string())
                    .header("x-reminder-signature", signature)
                    .body(Body::from(intake_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(intake.status(), StatusCode::OK);
        let intake_json: serde_json::Value =
            serde_json::from_slice(&intake.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        let reminder_id = intake_json["reminders"][0]["id"].as_str().unwrap();

        let dispatch = application
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/clinic/reminders/dispatch")
                    .header(header::AUTHORIZATION, "Bearer test:clinic-owner-a")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("idempotency-key", "dispatch-once-42")
                    .body(Body::from(format!(r#"{{"reminder_id":"{reminder_id}"}}"#)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(dispatch.status(), StatusCode::OK);
        let dispatch_text = String::from_utf8(
            dispatch
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec(),
        )
        .unwrap();
        assert!(dispatch_text.contains("No allowed channel has recorded consent"));
        assert!(!dispatch_text.contains(secret));

        let paid_reminder_id = intake_json["reminders"][1]["id"].as_str().unwrap();
        let unpaid_dispatch = application
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/clinic/reminders/dispatch")
                    .header(header::AUTHORIZATION, "Bearer test:clinic-owner-a")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("idempotency-key", "dispatch-paid-43")
                    .body(Body::from(format!(
                        r#"{{"reminder_id":"{paid_reminder_id}"}}"#
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unpaid_dispatch.status(), StatusCode::PAYMENT_REQUIRED);
        let unpaid_text = String::from_utf8(
            unpaid_dispatch
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec(),
        )
        .unwrap();
        assert!(unpaid_text.contains("subscription_required"));
        assert!(!unpaid_text.contains(secret));

        let other_tenant = application
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/clinic")
                    .header(header::AUTHORIZATION, "Bearer test:clinic-owner-b")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(other_tenant.status(), StatusCode::NOT_FOUND);
        drop(application);

        let restarted = app_with_clinic_state(
            "managed-test",
            "../../dist",
            ClinicState::for_tests(path.clone()).unwrap(),
        );
        let durable = restarted
            .oneshot(
                Request::builder()
                    .uri("/api/v1/clinic")
                    .header(header::AUTHORIZATION, "Bearer test:clinic-owner-a")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(durable.status(), StatusCode::OK);
        let durable_text = String::from_utf8(
            durable
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec(),
        )
        .unwrap();
        assert!(durable_text.contains("emr-appointment-42"));
        assert!(durable_text.contains("exception"));
        let _ = std::fs::remove_dir_all(path);
    }
}
