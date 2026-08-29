use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use rand::{distr::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

pub const DEMO_COOKIE: &str = "rp_demo";
const DEMO_TTL: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Clone, Default)]
pub struct DemoStore {
    limits: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

#[derive(Clone)]
struct DemoSession {
    workspace_id: String,
    expires_at: u64,
    advanced: u8,
    owner: u8,
    resolution: u8,
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
    pub request_id: String,
}

pub fn problem_response(status: StatusCode, code: &'static str, message: &'static str) -> Response {
    let request_id = uuid::Uuid::new_v4().to_string();
    tracing::warn!(
        request_id = %request_id,
        status = status.as_u16(),
        code,
        "API request rejected"
    );
    let mut response = (
        status,
        Json(Problem {
            code,
            message,
            request_id: request_id.clone(),
        }),
    )
        .into_response();
    response.headers_mut().insert(
        "x-request-id",
        HeaderValue::from_str(&request_id).expect("UUID is a valid header value"),
    );
    response
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: &'static str,
    retry_after: Option<u64>,
}

impl ApiError {
    pub fn new(status: StatusCode, code: &'static str, message: &'static str) -> Self {
        Self {
            status,
            code,
            message,
            retry_after: None,
        }
    }

    pub fn rate_limited(retry_after: u64) -> Self {
        Self {
            status: StatusCode::TOO_MANY_REQUESTS,
            code: "rate_limited",
            message: "Too many demo actions. Wait, then try again.",
            retry_after: Some(retry_after.max(1)),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut response = problem_response(self.status, self.code, self.message);
        if let Some(seconds) = self.retry_after {
            response.headers_mut().insert(
                header::RETRY_AFTER,
                HeaderValue::from_str(&seconds.to_string()).expect("valid retry-after"),
            );
        }
        response
    }
}

impl DemoStore {
    pub fn new() -> Self {
        Self {
            limits: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create(&self, client: &str) -> Result<(String, DemoData), ApiError> {
        self.check_limit(&format!("create:{client}"), 5, Duration::from_secs(60 * 60))
            .await?;
        let session = DemoSession::new();
        let data = session.data();
        Ok((session.encode(), data))
    }

    async fn check_limit(&self, key: &str, maximum: usize, span: Duration) -> Result<(), ApiError> {
        let now = Instant::now();
        let mut limits = self.limits.lock().await;
        let requests = limits.entry(key.to_owned()).or_default();
        requests.retain(|then| now.duration_since(*then) < span);
        if requests.len() >= maximum {
            let elapsed = now.duration_since(requests[0]);
            return Err(ApiError::rate_limited(
                span.saturating_sub(elapsed).as_secs(),
            ));
        }
        requests.push(now);
        Ok(())
    }

    fn session(&self, headers: &HeaderMap) -> Result<DemoSession, ApiError> {
        let raw = headers
            .get(header::COOKIE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| {
                value
                    .split(';')
                    .map(str::trim)
                    .find(|part| part.starts_with("rp_demo="))
            })
            .and_then(|part| part.strip_prefix("rp_demo="))
            .ok_or_else(|| {
                ApiError::new(
                    StatusCode::UNAUTHORIZED,
                    "demo_cookie_missing",
                    "Start a sample clinic first.",
                )
            })?;
        DemoSession::decode(raw)
    }

    async fn with_workspace(
        &self,
        headers: &HeaderMap,
        write: bool,
        operation: impl FnOnce(&mut DemoData) -> Result<(), ApiError>,
    ) -> Result<(String, DemoData), ApiError> {
        let mut session = self.session(headers)?;
        if write {
            self.check_limit(
                &format!("write:{}", session.workspace_id),
                30,
                Duration::from_secs(60),
            )
            .await?;
        }
        if session.expires_at <= epoch_seconds() {
            return Err(ApiError::new(
                StatusCode::GONE,
                "demo_expired",
                "This demo has expired. Start a new sample clinic.",
            ));
        }
        let mut data = session.data();
        operation(&mut data)?;
        session.capture(&data);
        Ok((session.encode(), data))
    }
}

impl DemoSession {
    fn new() -> Self {
        Self {
            workspace_id: random_id(),
            expires_at: epoch_seconds() + DEMO_TTL.as_secs(),
            advanced: 0,
            owner: 0,
            resolution: 0,
        }
    }

    fn encode(&self) -> String {
        format!(
            "1:{}:{}:{}:{}:{}",
            self.workspace_id, self.expires_at, self.advanced, self.owner, self.resolution
        )
    }

    fn decode(value: &str) -> Result<Self, ApiError> {
        let fields = value.split(':').collect::<Vec<_>>();
        let valid_id = fields.get(1).is_some_and(|id| {
            id.len() == 32
                && id
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric())
        });
        if fields.len() != 6 || fields[0] != "1" || !valid_id {
            return Err(invalid_cookie());
        }
        let session = Self {
            workspace_id: fields[1].to_owned(),
            expires_at: fields[2].parse().map_err(|_| invalid_cookie())?,
            advanced: fields[3].parse().map_err(|_| invalid_cookie())?,
            owner: fields[4].parse().map_err(|_| invalid_cookie())?,
            resolution: fields[5].parse().map_err(|_| invalid_cookie())?,
        };
        if session.advanced > 3 || session.owner > 2 || session.resolution > 4 {
            return Err(invalid_cookie());
        }
        Ok(session)
    }

    fn data(&self) -> DemoData {
        let mut data = seed(&self.workspace_id);
        if self.advanced & 1 != 0 {
            advance_mina(&mut data);
        }
        if self.advanced & 2 != 0 {
            advance_jordan(&mut data);
        }
        if let Some(exception) = data
            .reminders
            .iter_mut()
            .find_map(|item| item.exception.as_mut())
        {
            exception.owner = match self.owner {
                1 => Some("Sam Rivera".to_owned()),
                2 => Some("Avery Chen".to_owned()),
                _ => None,
            };
            if let Some(resolution) = resolution_name(self.resolution) {
                exception.state = "resolved";
                exception.resolution = Some(resolution.to_owned());
                exception.undo_available = true;
            } else if exception.owner.is_some() {
                exception.state = "assigned";
            }
        }
        data
    }

    fn capture(&mut self, data: &DemoData) {
        self.advanced = u8::from(
            data.reminders
                .iter()
                .any(|item| item.id == "mina" && item.state == "delivered"),
        ) | (u8::from(
            data.reminders
                .iter()
                .any(|item| item.id == "jordan" && item.state == "delivered"),
        ) << 1);
        if let Some(exception) = data
            .reminders
            .iter()
            .find_map(|item| item.exception.as_ref())
        {
            self.owner = match exception.owner.as_deref() {
                Some("Sam Rivera") => 1,
                Some("Avery Chen") => 2,
                _ => 0,
            };
            self.resolution = match exception.resolution.as_deref() {
                Some("Called patient") => 1,
                Some("Corrected contact") => 2,
                Some("Appointment cancelled") => 3,
                Some("No safe channel") => 4,
                _ => 0,
            };
        }
    }
}

fn invalid_cookie() -> ApiError {
    ApiError::new(
        StatusCode::UNAUTHORIZED,
        "demo_cookie_invalid",
        "Start a new sample clinic.",
    )
}

fn epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn resolution_name(code: u8) -> Option<&'static str> {
    [
        None,
        Some("Called patient"),
        Some("Corrected contact"),
        Some("Appointment cancelled"),
        Some("No safe channel"),
    ]
    .get(code as usize)
    .copied()
    .flatten()
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
    Ok(demo_response(cookie, data))
}

pub async fn state(
    State(store): State<DemoStore>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let (cookie, data) = store.with_workspace(&headers, false, |_| Ok(())).await?;
    Ok(demo_response(cookie, data))
}

pub async fn advance(
    State(store): State<DemoStore>,
    headers: HeaderMap,
    Path(reminder_id): Path<String>,
) -> Result<Response, ApiError> {
    let (cookie, data) = store
        .with_workspace(&headers, true, |data| {
            let reminder = data
                .reminders
                .iter_mut()
                .find(|reminder| reminder.id == reminder_id)
                .ok_or_else(|| {
                    ApiError::new(
                        StatusCode::NOT_FOUND,
                        "reminder_missing",
                        "That sample reminder does not exist.",
                    )
                })?;
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
            Ok(())
        })
        .await?;
    Ok(demo_response(cookie, data))
}

pub async fn advance_due(
    State(store): State<DemoStore>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let (cookie, data) = store
        .with_workspace(&headers, true, |data| {
            for reminder in &mut data.reminders {
                if reminder.id == "mina" && reminder.state == "scheduled" {
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
                if reminder.id == "jordan" && reminder.state == "scheduled" {
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
            }
            Ok(())
        })
        .await?;
    Ok(demo_response(cookie, data))
}

pub async fn assign_exception(
    State(store): State<DemoStore>,
    headers: HeaderMap,
    Path(exception_id): Path<String>,
    Json(input): Json<AssignInput>,
) -> Result<Response, ApiError> {
    if !["Sam Rivera", "Avery Chen"].contains(&input.owner.as_str()) {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "owner_invalid",
            "Choose a staff member in this sample clinic.",
        ));
    }
    let (cookie, data) = store
        .with_workspace(&headers, true, |data| {
            let exception = find_exception(data, &exception_id)?;
            exception.owner = Some(input.owner);
            exception.state = "assigned";
            Ok(())
        })
        .await?;
    Ok(demo_response(cookie, data))
}

pub async fn resolve_exception(
    State(store): State<DemoStore>,
    headers: HeaderMap,
    Path(exception_id): Path<String>,
    Json(input): Json<ResolveInput>,
) -> Result<Response, ApiError> {
    if ![
        "Called patient",
        "Corrected contact",
        "Appointment cancelled",
        "No safe channel",
    ]
    .contains(&input.resolution.as_str())
    {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "resolution_invalid",
            "Choose a safe sample resolution.",
        ));
    }
    let (cookie, data) = store
        .with_workspace(&headers, true, |data| {
            let exception = find_exception(data, &exception_id)?;
            if exception.owner.is_none() {
                return Err(ApiError::new(
                    StatusCode::CONFLICT,
                    "owner_required",
                    "Assign an owner before resolving this exception.",
                ));
            }
            exception.state = "resolved";
            exception.resolution = Some(input.resolution);
            exception.undo_available = true;
            Ok(())
        })
        .await?;
    Ok(demo_response(cookie, data))
}

pub async fn undo_exception(
    State(store): State<DemoStore>,
    headers: HeaderMap,
    Path(exception_id): Path<String>,
) -> Result<Response, ApiError> {
    let (cookie, data) = store
        .with_workspace(&headers, true, |data| {
            let exception = find_exception(data, &exception_id)?;
            if exception.state != "resolved" || !exception.undo_available {
                return Err(ApiError::new(
                    StatusCode::CONFLICT,
                    "undo_unavailable",
                    "That resolution can no longer be undone.",
                ));
            }
            exception.state = "assigned";
            exception.resolution = None;
            exception.undo_available = false;
            Ok(())
        })
        .await?;
    Ok(demo_response(cookie, data))
}

pub async fn reset_workspace(
    State(store): State<DemoStore>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let _ = store.session(&headers)?;
    let (cookie, data) = store.create(&client_ip(&headers)).await?;
    Ok(demo_response(cookie, data))
}

fn demo_response(cookie: String, data: DemoData) -> Response {
    let mut response = Json(DemoEnvelope { demo: data }).into_response();
    let set_cookie = format!(
        "{DEMO_COOKIE}={cookie}; Path=/api/v1/demo; HttpOnly; Secure; SameSite=Lax; Max-Age={}",
        DEMO_TTL.as_secs()
    );
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&set_cookie).expect("safe cookie value"),
    );
    response
}

fn find_exception<'a>(data: &'a mut DemoData, id: &str) -> Result<&'a mut DemoException, ApiError> {
    data.reminders
        .iter_mut()
        .filter_map(|reminder| reminder.exception.as_mut())
        .find(|exception| exception.id == id)
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::NOT_FOUND,
                "exception_missing",
                "That sample exception does not exist.",
            )
        })
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
    EvidenceEvent {
        at,
        kind,
        label: label.to_owned(),
        channel,
        provider_result,
        outcome,
        simulated: true,
    }
}

fn advance_mina(data: &mut DemoData) {
    if let Some(reminder) = data
        .reminders
        .iter_mut()
        .find(|reminder| reminder.id == "mina" && reminder.state == "scheduled")
    {
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
}

fn advance_jordan(data: &mut DemoData) {
    if let Some(reminder) = data
        .reminders
        .iter_mut()
        .find(|reminder| reminder.id == "jordan" && reminder.state == "scheduled")
    {
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
}

fn seed(workspace_id: &str) -> DemoData {
    DemoData {
        workspace_id: workspace_id.to_owned(),
        clinic: Clinic {
            name: "Northline Sample Clinic",
            timezone: "America/Chicago",
            simulated: true,
        },
        staff: vec![
            Staff {
                id: "sam",
                name: "Sam Rivera",
            },
            Staff {
                id: "avery",
                name: "Avery Chen",
            },
        ],
        reminders: vec![
            Reminder {
                id: "mina",
                patient_alias: "Mina P.",
                appointment_time: "Today, 09:00",
                appointment: "Hygiene visit",
                state: "scheduled",
                due: true,
                events: vec![
                    event(
                        "07:55",
                        "source",
                        "Source appointment is due.",
                        None,
                        None,
                        "Scheduled",
                    ),
                    event(
                        "07:56",
                        "consent",
                        "SMS consent is recorded.",
                        Some("SMS"),
                        None,
                        "Allowed",
                    ),
                ],
                exception: None,
            },
            Reminder {
                id: "jordan",
                patient_alias: "Jordan L.",
                appointment_time: "Today, 10:30",
                appointment: "Follow-up visit",
                state: "scheduled",
                due: true,
                events: vec![
                    event(
                        "09:25",
                        "source",
                        "Source appointment is due.",
                        None,
                        None,
                        "Scheduled",
                    ),
                    event(
                        "09:26",
                        "consent",
                        "WhatsApp and email consent are recorded.",
                        Some("WhatsApp"),
                        None,
                        "Allowed",
                    ),
                ],
                exception: None,
            },
            Reminder {
                id: "sofia",
                patient_alias: "Sofia R.",
                appointment_time: "Today, 14:00",
                appointment: "New patient visit",
                state: "exception",
                due: true,
                events: vec![
                    event(
                        "13:20",
                        "source",
                        "Source appointment is due.",
                        None,
                        None,
                        "Scheduled",
                    ),
                    event(
                        "13:21",
                        "consent",
                        "SMS is blocked by an opt-out. No provider attempt was made.",
                        Some("SMS"),
                        None,
                        "Blocked",
                    ),
                ],
                exception: Some(DemoException {
                    id: "sofia-exception",
                    reason: "SMS is opted out. No other channel is allowed.",
                    next_action: "Assign someone to follow up.",
                    owner: None,
                    state: "open",
                    resolution: None,
                    undo_available: false,
                }),
            },
            Reminder {
                id: "eli",
                patient_alias: "Eli K.",
                appointment_time: "Today, 15:30",
                appointment: "Review visit",
                state: "delivered",
                due: true,
                events: vec![
                    event(
                        "14:50",
                        "source",
                        "Source appointment is due.",
                        None,
                        None,
                        "Scheduled",
                    ),
                    event(
                        "14:51",
                        "consent",
                        "Email consent is recorded.",
                        Some("Email"),
                        None,
                        "Allowed",
                    ),
                    event(
                        "14:52",
                        "attempt",
                        "Email accepted by the simulated provider.",
                        Some("Email"),
                        Some("DELIVERED-200"),
                        "Delivered",
                    ),
                    event(
                        "14:54",
                        "response",
                        "Sample patient reply: YES.",
                        None,
                        Some("REPLY-YES"),
                        "Replied",
                    ),
                ],
                exception: None,
            },
            Reminder {
                id: "noor",
                patient_alias: "Noor A.",
                appointment_time: "Tomorrow, 08:30",
                appointment: "Cleaning",
                state: "cancelled",
                due: false,
                events: vec![event(
                    "16:00",
                    "source",
                    "Source cancelled the appointment before the reminder was due.",
                    None,
                    None,
                    "Cancelled",
                )],
                exception: None,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn workspace_data_is_isolated_and_reset_reseeds() {
        let store = DemoStore::new();
        let (first_cookie, first) = store.create("one").await.unwrap();
        let (_, second) = store.create("two").await.unwrap();
        assert_ne!(first.workspace_id, second.workspace_id);
        let headers = cookie_headers(&first_cookie);
        let changed = store
            .with_workspace(&headers, true, |data| {
                data.reminders[0].state = "delivered";
                Ok(())
            })
            .await
            .unwrap();
        assert_eq!(changed.1.reminders[0].state, "delivered");
        assert_eq!(second.reminders[0].state, "scheduled");
    }

    #[tokio::test]
    async fn consent_block_never_creates_provider_attempt() {
        let store = DemoStore::new();
        let (cookie, _) = store.create("one").await.unwrap();
        let data = store
            .with_workspace(&cookie_headers(&cookie), false, |_| Ok(()))
            .await
            .unwrap();
        let sofia = data
            .1
            .reminders
            .iter()
            .find(|reminder| reminder.id == "sofia")
            .unwrap();
        assert!(sofia.events.iter().all(|event| event.kind != "attempt"));
        assert_eq!(sofia.exception.as_ref().unwrap().state, "open");
    }

    #[tokio::test]
    async fn workspace_creation_limit_returns_429() {
        let store = DemoStore::new();
        for _ in 0..5 {
            store.create("limit").await.unwrap();
        }
        let error = store.create("limit").await.unwrap_err();
        assert_eq!(error.status, StatusCode::TOO_MANY_REQUESTS);
    }

    fn cookie_headers(cookie: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_str(&format!("rp_demo={cookie}")).unwrap(),
        );
        headers
    }
}
