use std::{collections::HashMap, sync::Arc, time::{Duration, Instant}};

use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use hmac::{Hmac, Mac};
use rand::{distr::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tokio::sync::Mutex;

type HmacSha256 = Hmac<Sha256>;
pub const DEMO_COOKIE: &str = "rp_demo";
const DEMO_TTL: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Clone)]
pub struct DemoStore {
    secret: Arc<Vec<u8>>,
    workspaces: Arc<Mutex<HashMap<String, Workspace>>>,
    limits: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

#[derive(Clone)]
struct Workspace {
    expires_at: Instant,
    data: DemoData,
}

#[derive(Clone, Debug, Serialize)]
pub struct DemoData {
    pub workspace_id: String,
    pub clinic: Clinic,
    pub staff: Vec<Staff>,
    pub reminders: Vec<Reminder>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Clinic {
    pub name: &'static str,
    pub timezone: &'static str,
    pub simulated: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct Staff {
    pub id: &'static str,
    pub name: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct Reminder {
    pub id: &'static str,
    pub patient_alias: &'static str,
    pub appointment_time: &'static str,
    pub appointment: &'static str,
    pub state: &'static str,
    pub due: bool,
    pub events: Vec<EvidenceEvent>,
    pub exception: Option<DemoException>,
}

#[derive(Clone, Debug, Serialize)]
pub struct EvidenceEvent {
    pub at: &'static str,
    pub kind: &'static str,
    pub label: String,
    pub channel: Option<&'static str>,
    pub provider_result: Option<&'static str>,
    pub outcome: &'static str,
    pub simulated: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct DemoException {
    pub id: &'static str,
    pub reason: &'static str,
    pub next_action: &'static str,
    pub owner: Option<String>,
    pub state: &'static str,
    pub resolution: Option<String>,
    pub undo_available: bool,
}

#[derive(Serialize)]
pub struct DemoEnvelope {
    pub demo: DemoData,
}

#[derive(Deserialize)]
pub struct AssignInput {
    pub owner: String,
}

#[derive(Deserialize)]
pub struct ResolveInput {
    pub resolution: String,
}

#[derive(Serialize)]
pub struct Problem {
    pub code: &'static str,
    pub message: &'static str,
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: &'static str,
}

impl ApiError {
    pub fn new(status: StatusCode, code: &'static str, message: &'static str) -> Self {
        Self { status, code, message }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(Problem {
                code: self.code,
                message: self.message,
            }),
        )
            .into_response()
    }
}

impl DemoStore {
    pub fn new(secret: Vec<u8>) -> Self {
        Self {
            secret: Arc::new(secret),
            workspaces: Arc::new(Mutex::new(HashMap::new())),
            limits: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create(&self, client: &str) -> Result<(String, DemoData), ApiError> {
        self.check_limit(&format!("create:{client}"), 5, Duration::from_secs(60 * 60))
            .await?;
        self.cleanup().await;
        let id = random_id();
        let data = seed(&id);
        self.workspaces.lock().await.insert(
            id.clone(),
            Workspace {
                expires_at: Instant::now() + DEMO_TTL,
                data: data.clone(),
            },
        );
        Ok((self.cookie_value(&id), data))
    }

    async fn check_limit(&self, key: &str, maximum: usize, span: Duration) -> Result<(), ApiError> {
        let now = Instant::now();
        let mut limits = self.limits.lock().await;
        let requests = limits.entry(key.to_owned()).or_default();
        requests.retain(|then| now.duration_since(*then) < span);
        if requests.len() >= maximum {
            return Err(ApiError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "Too many demo actions. Wait a moment and try again.",
            ));
        }
        requests.push(now);
        Ok(())
    }

    async fn cleanup(&self) {
        let now = Instant::now();
        self.workspaces
            .lock()
            .await
            .retain(|_, workspace| workspace.expires_at > now);
    }

    fn cookie_value(&self, id: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(&self.secret).expect("valid HMAC key");
        mac.update(id.as_bytes());
        format!("{id}.{}", hex::encode(mac.finalize().into_bytes()))
    }

    fn workspace_id(&self, headers: &HeaderMap) -> Result<String, ApiError> {
        let raw = headers
            .get(header::COOKIE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(';').map(str::trim).find(|part| part.starts_with("rp_demo=")))
            .and_then(|part| part.strip_prefix("rp_demo="))
            .ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "demo_cookie_missing", "Start a sample clinic first."))?;
        let (id, signature) = raw
            .rsplit_once('.')
            .ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "demo_cookie_invalid", "Start a new sample clinic."))?;
        let received = hex::decode(signature)
            .map_err(|_| ApiError::new(StatusCode::UNAUTHORIZED, "demo_cookie_invalid", "Start a new sample clinic."))?;
        let mut mac = HmacSha256::new_from_slice(&self.secret).expect("valid HMAC key");
        mac.update(id.as_bytes());
        mac.verify_slice(&received)
            .map_err(|_| ApiError::new(StatusCode::UNAUTHORIZED, "demo_cookie_invalid", "Start a new sample clinic."))?;
        Ok(id.to_owned())
    }

    async fn with_workspace<R>(
        &self,
        headers: &HeaderMap,
        write: bool,
        operation: impl FnOnce(&mut DemoData) -> Result<R, ApiError>,
    ) -> Result<R, ApiError> {
        let id = self.workspace_id(headers)?;
        if write {
            self.check_limit(&format!("write:{id}"), 30, Duration::from_secs(60)).await?;
        }
        let mut workspaces = self.workspaces.lock().await;
        let workspace = workspaces.get_mut(&id).ok_or_else(|| {
            ApiError::new(StatusCode::GONE, "demo_expired", "This demo has expired. Start a new sample clinic.")
        })?;
        if workspace.expires_at <= Instant::now() {
            workspaces.remove(&id);
            return Err(ApiError::new(
                StatusCode::GONE,
                "demo_expired",
                "This demo has expired. Start a new sample clinic.",
            ));
        }
        operation(&mut workspace.data)
    }

    async fn delete(&self, headers: &HeaderMap) -> Result<(), ApiError> {
        let id = self.workspace_id(headers)?;
        self.workspaces.lock().await.remove(&id);
        Ok(())
    }
}

pub fn client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("local")
        .to_owned()
}

pub async fn create_workspace(
    State(store): State<DemoStore>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let (cookie, data) = store.create(&client_ip(&headers)).await?;
    let mut response = Json(DemoEnvelope { demo: data }).into_response();
    let set_cookie = format!(
        "{DEMO_COOKIE}={cookie}; Path=/api/v1/demo; HttpOnly; SameSite=Lax; Max-Age={}",
        DEMO_TTL.as_secs()
    );
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&set_cookie).expect("safe cookie value"),
    );
    Ok(response)
}

pub async fn state(
    State(store): State<DemoStore>,
    headers: HeaderMap,
) -> Result<Json<DemoEnvelope>, ApiError> {
    let data = store.with_workspace(&headers, false, |data| Ok(data.clone())).await?;
    Ok(Json(DemoEnvelope { demo: data }))
}

pub async fn advance(
    State(store): State<DemoStore>,
    headers: HeaderMap,
    Path(reminder_id): Path<String>,
) -> Result<Json<DemoEnvelope>, ApiError> {
    let data = store
        .with_workspace(&headers, true, |data| {
            let reminder = data
                .reminders
                .iter_mut()
                .find(|reminder| reminder.id == reminder_id)
                .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, "reminder_missing", "That sample reminder does not exist."))?;
            match reminder.id {
                "mina" if reminder.state == "scheduled" => {
                    reminder.state = "delivered";
                    reminder.events.push(event(
                        "08:01",
                        "attempt",
                        "SMS attempt accepted by the simulated provider.",
                        Some("SMS"),
                        Some("DELIVERED-200"),
                        "Delivered",
                    ));
                }
                "jordan" if reminder.state == "scheduled" => {
                    reminder.state = "delivered";
                    reminder.events.push(event(
                        "09:31",
                        "attempt",
                        "Approved WhatsApp template rejected by the simulated provider.",
                        Some("WhatsApp"),
                        Some("TEMPLATE_REJECTED"),
                        "Failed",
                    ));
                    reminder.events.push(event(
                        "09:32",
                        "attempt",
                        "Email fallback accepted by the simulated provider.",
                        Some("Email"),
                        Some("DELIVERED-200"),
                        "Delivered",
                    ));
                }
                _ => {}
            }
            Ok(data.clone())
        })
        .await?;
    Ok(Json(DemoEnvelope { demo: data }))
}

pub async fn advance_due(
    State(store): State<DemoStore>,
    headers: HeaderMap,
) -> Result<Json<DemoEnvelope>, ApiError> {
    let data = store
        .with_workspace(&headers, true, |data| {
            for reminder in &mut data.reminders {
                if reminder.id == "mina" && reminder.state == "scheduled" {
                    reminder.state = "delivered";
                    reminder.events.push(event("08:01", "attempt", "SMS attempt accepted by the simulated provider.", Some("SMS"), Some("DELIVERED-200"), "Delivered"));
                }
                if reminder.id == "jordan" && reminder.state == "scheduled" {
                    reminder.state = "delivered";
                    reminder.events.push(event("09:31", "attempt", "Approved WhatsApp template rejected by the simulated provider.", Some("WhatsApp"), Some("TEMPLATE_REJECTED"), "Failed"));
                    reminder.events.push(event("09:32", "attempt", "Email fallback accepted by the simulated provider.", Some("Email"), Some("DELIVERED-200"), "Delivered"));
                }
            }
            Ok(data.clone())
        })
        .await?;
    Ok(Json(DemoEnvelope { demo: data }))
}

pub async fn assign_exception(
    State(store): State<DemoStore>,
    headers: HeaderMap,
    Path(exception_id): Path<String>,
    Json(input): Json<AssignInput>,
) -> Result<Json<DemoEnvelope>, ApiError> {
    if !["Sam Rivera", "Avery Chen"].contains(&input.owner.as_str()) {
        return Err(ApiError::new(StatusCode::UNPROCESSABLE_ENTITY, "owner_invalid", "Choose a staff member in this sample clinic."));
    }
    let data = store.with_workspace(&headers, true, |data| {
        let exception = find_exception(data, &exception_id)?;
        exception.owner = Some(input.owner);
        exception.state = "assigned";
        Ok(data.clone())
    }).await?;
    Ok(Json(DemoEnvelope { demo: data }))
}

pub async fn resolve_exception(
    State(store): State<DemoStore>,
    headers: HeaderMap,
    Path(exception_id): Path<String>,
    Json(input): Json<ResolveInput>,
) -> Result<Json<DemoEnvelope>, ApiError> {
    if !["Called patient", "Corrected contact", "Appointment cancelled", "No safe channel"].contains(&input.resolution.as_str()) {
        return Err(ApiError::new(StatusCode::UNPROCESSABLE_ENTITY, "resolution_invalid", "Choose a safe sample resolution."));
    }
    let data = store.with_workspace(&headers, true, |data| {
        let exception = find_exception(data, &exception_id)?;
        if exception.owner.is_none() {
            return Err(ApiError::new(StatusCode::CONFLICT, "owner_required", "Assign an owner before resolving this exception."));
        }
        exception.state = "resolved";
        exception.resolution = Some(input.resolution);
        exception.undo_available = true;
        Ok(data.clone())
    }).await?;
    Ok(Json(DemoEnvelope { demo: data }))
}

pub async fn undo_exception(
    State(store): State<DemoStore>,
    headers: HeaderMap,
    Path(exception_id): Path<String>,
) -> Result<Json<DemoEnvelope>, ApiError> {
    let data = store.with_workspace(&headers, true, |data| {
        let exception = find_exception(data, &exception_id)?;
        if exception.state != "resolved" || !exception.undo_available {
            return Err(ApiError::new(StatusCode::CONFLICT, "undo_unavailable", "That resolution can no longer be undone."));
        }
        exception.state = "assigned";
        exception.resolution = None;
        exception.undo_available = false;
        Ok(data.clone())
    }).await?;
    Ok(Json(DemoEnvelope { demo: data }))
}

pub async fn reset_workspace(
    State(store): State<DemoStore>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    store.delete(&headers).await?;
    let (cookie, data) = store.create(&client_ip(&headers)).await?;
    let mut response = Json(DemoEnvelope { demo: data }).into_response();
    let set_cookie = format!(
        "{DEMO_COOKIE}={cookie}; Path=/api/v1/demo; HttpOnly; SameSite=Lax; Max-Age={}",
        DEMO_TTL.as_secs()
    );
    response.headers_mut().insert(header::SET_COOKIE, HeaderValue::from_str(&set_cookie).expect("safe cookie value"));
    Ok(response)
}

fn find_exception<'a>(data: &'a mut DemoData, id: &str) -> Result<&'a mut DemoException, ApiError> {
    data.reminders
        .iter_mut()
        .filter_map(|reminder| reminder.exception.as_mut())
        .find(|exception| exception.id == id)
        .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, "exception_missing", "That sample exception does not exist."))
}

fn random_id() -> String {
    let mut rng = rand::rng();
    (0..32).map(|_| rng.sample(Alphanumeric) as char).collect()
}

fn event(
    at: &'static str,
    kind: &'static str,
    label: &str,
    channel: Option<&'static str>,
    provider_result: Option<&'static str>,
    outcome: &'static str,
) -> EvidenceEvent {
    EvidenceEvent { at, kind, label: label.to_owned(), channel, provider_result, outcome, simulated: true }
}

fn seed(workspace_id: &str) -> DemoData {
    DemoData {
        workspace_id: workspace_id.to_owned(),
        clinic: Clinic { name: "Northline Sample Clinic", timezone: "America/Chicago", simulated: true },
        staff: vec![Staff { id: "sam", name: "Sam Rivera" }, Staff { id: "avery", name: "Avery Chen" }],
        reminders: vec![
            Reminder {
                id: "mina", patient_alias: "Mina P.", appointment_time: "Today, 09:00", appointment: "Hygiene visit", state: "scheduled", due: true,
                events: vec![event("07:55", "source", "Source appointment is due.", None, None, "Scheduled"), event("07:56", "consent", "SMS consent is recorded.", Some("SMS"), None, "Allowed")], exception: None,
            },
            Reminder {
                id: "jordan", patient_alias: "Jordan L.", appointment_time: "Today, 10:30", appointment: "Follow-up visit", state: "scheduled", due: true,
                events: vec![event("09:25", "source", "Source appointment is due.", None, None, "Scheduled"), event("09:26", "consent", "WhatsApp and email consent are recorded.", Some("WhatsApp"), None, "Allowed")], exception: None,
            },
            Reminder {
                id: "sofia", patient_alias: "Sofia R.", appointment_time: "Today, 14:00", appointment: "New patient visit", state: "exception", due: true,
                events: vec![event("13:20", "source", "Source appointment is due.", None, None, "Scheduled"), event("13:21", "consent", "SMS is blocked by an opt-out. No provider attempt was made.", Some("SMS"), None, "Blocked")],
                exception: Some(DemoException { id: "sofia-exception", reason: "SMS is opted out. No other channel is allowed.", next_action: "Assign someone to follow up.", owner: None, state: "open", resolution: None, undo_available: false }),
            },
            Reminder {
                id: "eli", patient_alias: "Eli K.", appointment_time: "Today, 15:30", appointment: "Review visit", state: "delivered", due: true,
                events: vec![event("14:50", "source", "Source appointment is due.", None, None, "Scheduled"), event("14:51", "consent", "Email consent is recorded.", Some("Email"), None, "Allowed"), event("14:52", "attempt", "Email accepted by the simulated provider.", Some("Email"), Some("DELIVERED-200"), "Delivered"), event("14:54", "response", "Sample patient reply: YES.", None, Some("REPLY-YES"), "Replied")], exception: None,
            },
            Reminder {
                id: "noor", patient_alias: "Noor A.", appointment_time: "Tomorrow, 08:30", appointment: "Cleaning", state: "cancelled", due: false,
                events: vec![event("16:00", "source", "Source cancelled the appointment before the reminder was due.", None, None, "Cancelled")], exception: None,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn workspace_data_is_isolated_and_reset_reseeds() {
        let store = DemoStore::new(vec![7; 32]);
        let (first_cookie, first) = store.create("one").await.unwrap();
        let (_, second) = store.create("two").await.unwrap();
        assert_ne!(first.workspace_id, second.workspace_id);
        let headers = cookie_headers(&first_cookie);
        let changed = store.with_workspace(&headers, true, |data| { data.reminders[0].state = "delivered"; Ok(data.clone()) }).await.unwrap();
        assert_eq!(changed.reminders[0].state, "delivered");
        assert_eq!(second.reminders[0].state, "scheduled");
    }

    #[tokio::test]
    async fn consent_block_never_creates_provider_attempt() {
        let store = DemoStore::new(vec![7; 32]);
        let (cookie, _) = store.create("one").await.unwrap();
        let data = store.with_workspace(&cookie_headers(&cookie), false, |data| Ok(data.clone())).await.unwrap();
        let sofia = data.reminders.iter().find(|reminder| reminder.id == "sofia").unwrap();
        assert!(sofia.events.iter().all(|event| event.kind != "attempt"));
        assert_eq!(sofia.exception.as_ref().unwrap().state, "open");
    }

    #[tokio::test]
    async fn workspace_creation_limit_returns_429() {
        let store = DemoStore::new(vec![7; 32]);
        for _ in 0..5 { store.create("limit").await.unwrap(); }
        let error = store.create("limit").await.unwrap_err();
        assert_eq!(error.status, StatusCode::TOO_MANY_REQUESTS);
    }

    fn cookie_headers(cookie: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::COOKIE, HeaderValue::from_str(&format!("rp_demo={cookie}")).unwrap());
        headers
    }
}
