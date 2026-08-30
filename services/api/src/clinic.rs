#![allow(clippy::result_large_err)]

use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use axum::{
    extract::{Path as AxumPath, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use rand::RngCore;
use rusqlite::{params, Connection, OptionalExtension, Transaction, MAIN_DB};
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    auth::{AuthConfig, AuthService, Identity},
    demo::ApiError,
};

#[derive(Clone)]
pub struct ClinicState {
    pub auth: AuthService,
    store: ClinicStore,
    client: reqwest::Client,
    billing_base_url: String,
    provider_fixture_base_url: Option<String>,
}

#[derive(Clone)]
struct ClinicStore {
    connection: Arc<Mutex<Connection>>,
    cipher: Arc<Aes256Gcm>,
    key_path: Arc<PathBuf>,
    durable_dir: Arc<PathBuf>,
    backup_dir: Arc<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClinicWorkspace {
    #[serde(default)]
    owner_oid: String,
    pub organization_id: String,
    pub clinic_name: String,
    pub location_name: String,
    pub timezone: String,
    #[serde(default = "default_jurisdiction")]
    pub jurisdiction: String,
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
    #[serde(default)]
    pub members: Vec<ClinicMember>,
    #[serde(default)]
    pub deletion: Option<DeletionSchedule>,
    pub connector: Option<ConnectorPublic>,
    #[serde(default)]
    encrypted_connector_secret: Option<String>,
    #[serde(default)]
    provider_configs: Vec<StoredProvider>,
    #[serde(default)]
    pub reminders: Vec<ClinicReminder>,
    #[serde(default)]
    pub subscription: Subscription,
    #[serde(default)]
    pub audit: Vec<AuditEvent>,
}

impl Default for ClinicWorkspace {
    fn default() -> Self {
        Self {
            owner_oid: String::new(),
            organization_id: String::new(),
            clinic_name: String::new(),
            location_name: String::new(),
            timezone: String::new(),
            jurisdiction: default_jurisdiction(),
            retention_days: default_retention_days(),
            members: Vec::new(),
            deletion: None,
            connector: None,
            encrypted_connector_secret: None,
            provider_configs: Vec::new(),
            reminders: Vec::new(),
            subscription: Subscription::default(),
            audit: Vec::new(),
        }
    }
}

fn default_jurisdiction() -> String {
    "other".to_owned()
}

const fn default_retention_days() -> u32 {
    365
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClinicMember {
    pub id: String,
    pub user_oid: Option<String>,
    pub display_name: String,
    pub email: String,
    pub role: String,
    pub state: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeletionSchedule {
    pub scheduled_at: u64,
    pub cancel_until: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConnectorPublic {
    pub id: String,
    pub kind: String,
    pub connected_at: u64,
    pub last_received_at: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct StoredProvider {
    id: String,
    channel: String,
    kind: String,
    account_id: String,
    encrypted_secret: String,
    from: String,
    approved_template_id: String,
    encrypted_webhook_secret: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClinicReminder {
    pub id: String,
    pub source_id: String,
    pub patient_alias: String,
    pub first_name: String,
    pub appointment_time: String,
    pub status: String,
    pub channels: Vec<ReminderChannel>,
    pub timeline: Vec<ClinicEvent>,
    pub exception: Option<ClinicException>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReminderChannel {
    pub channel: String,
    pub destination: String,
    pub consent: String,
    pub consent_source: String,
    pub consent_captured_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClinicEvent {
    pub at: u64,
    pub kind: String,
    pub channel: Option<String>,
    pub outcome: String,
    pub provider_reference: Option<String>,
    pub provider_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClinicException {
    pub id: String,
    pub reason: String,
    pub owner: Option<String>,
    pub state: String,
    pub resolution: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Subscription {
    pub tier: Option<String>,
    pub status: Option<String>,
    pub checked_at: Option<u64>,
    /// Kept encrypted so a future daily entitlement refresh can use the
    /// Sociobot token without ever returning it to the browser or export.
    #[serde(default)]
    encrypted_entitlement: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuditEvent {
    pub at: u64,
    pub actor: String,
    pub action: String,
    pub target: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OnboardInput {
    clinic_name: String,
    location_name: String,
    timezone: String,
    #[serde(default)]
    jurisdiction: Option<String>,
    #[serde(default)]
    retention_days: Option<u32>,
    #[serde(default)]
    owner_name: Option<String>,
    #[serde(default)]
    staff: Vec<MemberInput>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemberInput {
    #[serde(default)]
    user_oid: Option<String>,
    display_name: String,
    #[serde(default)]
    email: String,
    role: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocationInput {
    name: String,
    timezone: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderInput {
    channel: String,
    kind: String,
    account_id: String,
    secret: String,
    from: String,
    approved_template_id: String,
    webhook_secret: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectorInput {
    kind: String,
    webhook_secret: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntakeInput {
    connector_id: String,
    appointments: Vec<AppointmentInput>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppointmentInput {
    source_id: String,
    patient_alias: String,
    first_name: String,
    appointment_time: String,
    channels: Vec<ReminderChannel>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchInput {
    reminder_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptInput {
    provider_reference: String,
    provider_event_id: String,
    outcome: String,
    occurred_at: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssignInput {
    owner: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolveInput {
    resolution: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BillingQuery {
    tier: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BillingReturnInput {
    license: String,
    #[serde(default)]
    tier: Option<String>,
}

#[derive(Serialize)]
pub struct CheckoutResponse {
    checkout_url: String,
    tier: String,
}

#[derive(Serialize)]
pub struct MeResponse {
    oid: String,
    organization_id: Option<String>,
    role: Option<String>,
}

#[derive(Serialize)]
pub struct OrganizationSummary {
    id: String,
    name: String,
    jurisdiction: String,
    retention_days: u32,
}

#[derive(Serialize)]
pub struct LocationSummary {
    id: String,
    organization_id: String,
    name: String,
    timezone: String,
}

#[derive(Serialize)]
pub struct DeletionResponse {
    status: &'static str,
    cancel_until: Option<u64>,
}

#[derive(Serialize)]
pub struct ClinicExport {
    schema_version: u8,
    exported_at: u64,
    clinic: WorkspaceResponse,
    audit_events: Vec<AuditEvent>,
    notification_preferences: Vec<NotificationPreferenceExport>,
}

#[derive(Serialize)]
pub struct NotificationPreferenceExport {
    membership_id: String,
    digest_enabled: bool,
    exception_email: bool,
}

#[derive(Serialize)]
pub struct ConnectorCreated {
    connector: ConnectorPublic,
    signing_secret: String,
    intake_url: &'static str,
}

#[derive(Serialize)]
struct ProviderPublic {
    id: String,
    channel: String,
    kind: String,
    from: String,
    approved_template_id: String,
}

#[derive(Serialize)]
pub struct WorkspaceResponse {
    organization_id: String,
    clinic_name: String,
    location_name: String,
    timezone: String,
    jurisdiction: String,
    retention_days: u32,
    members: Vec<ClinicMember>,
    deletion: Option<DeletionSchedule>,
    connector: Option<ConnectorPublic>,
    providers: Vec<ProviderPublic>,
    reminders: Vec<ClinicReminder>,
    subscription: SubscriptionPublic,
}

#[derive(Serialize)]
struct SubscriptionPublic {
    tier: Option<String>,
    status: Option<String>,
    checked_at: Option<u64>,
}

impl From<ClinicWorkspace> for WorkspaceResponse {
    fn from(mut value: ClinicWorkspace) -> Self {
        let providers = value
            .provider_configs
            .iter()
            .map(|item| ProviderPublic {
                id: item.id.clone(),
                channel: item.channel.clone(),
                kind: item.kind.clone(),
                from: item.from.clone(),
                approved_template_id: item.approved_template_id.clone(),
            })
            .collect();
        // Contact destinations are dispatch inputs, not browser or export
        // output. Keep the channel and consent proof while ensuring an
        // authorized response cannot become a second plaintext contact list.
        for reminder in &mut value.reminders {
            for channel in &mut reminder.channels {
                channel.destination = "[encrypted contact]".to_owned();
            }
        }
        Self {
            organization_id: value.organization_id,
            clinic_name: value.clinic_name,
            location_name: value.location_name,
            timezone: value.timezone,
            jurisdiction: value.jurisdiction,
            retention_days: value.retention_days,
            members: value.members,
            deletion: value.deletion,
            connector: value.connector,
            providers,
            reminders: value.reminders,
            subscription: SubscriptionPublic {
                tier: value.subscription.tier,
                status: value.subscription.status,
                checked_at: value.subscription.checked_at,
            },
        }
    }
}

impl ClinicState {
    pub fn from_env() -> Result<Self, String> {
        let dir = data_dir();
        let durable = env::var_os("DURABLE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                if dir == Path::new("/data") {
                    PathBuf::from("/durable")
                } else {
                    dir.join("durable")
                }
            });
        let backups = env::var_os("BACKUP_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                if dir == Path::new("/data") {
                    PathBuf::from("/backups")
                } else {
                    dir.join("backups")
                }
            });
        ensure_required_storage_mounts(&durable, &backups)?;
        Self::new(dir, durable, backups, AuthService::from_env(), None, None)
    }

    #[cfg(test)]
    pub fn for_tests(path: PathBuf) -> Result<Self, String> {
        let backups = path.join("backups");
        let durable = path.join("durable");
        Self::new(path, durable, backups, AuthService::for_tests(), None, None)
    }

    #[cfg(test)]
    fn for_tests_with_fixtures(
        path: PathBuf,
        billing_base_url: String,
        provider_fixture_base_url: String,
    ) -> Result<Self, String> {
        let backups = path.join("backups");
        let durable = path.join("durable");
        Self::new(
            path,
            durable,
            backups,
            AuthService::for_tests(),
            Some(billing_base_url),
            Some(provider_fixture_base_url),
        )
    }

    fn new(
        dir: PathBuf,
        durable_dir: PathBuf,
        backup_dir: PathBuf,
        auth: AuthService,
        billing_base_url: Option<String>,
        provider_fixture_base_url: Option<String>,
    ) -> Result<Self, String> {
        fs::create_dir_all(&dir).map_err(|error| format!("create data directory: {error}"))?;
        fs::create_dir_all(&backup_dir)
            .map_err(|error| format!("create backup directory: {error}"))?;
        fs::create_dir_all(&durable_dir)
            .map_err(|error| format!("create durable directory: {error}"))?;
        restrict_path(&dir, 0o700)?;
        restrict_path(&durable_dir, 0o700)?;
        restrict_path(&backup_dir, 0o700)?;
        let key_path = dir.join("clinic-data.key");
        let database_path = dir.join("clinic-data.sqlite3");
        restore_durable_pair(&key_path, &database_path, &durable_dir)?;
        let key_was_generated = !key_path.exists();
        let key = load_or_create_key(&key_path)?;
        let connection = Connection::open(&database_path)
            .map_err(|error| format!("open clinic database: {error}"))?;
        // SQLite honours the process umask on first creation. Make this
        // explicit because the database contains encrypted, but still
        // sensitive, patient-operation metadata.
        restrict_path(&database_path, 0o600)?;
        connection
            .execute_batch(include_str!("../migrations/0001_managed_clinic.sql"))
            .map_err(|error| format!("migrate clinic database: {error}"))?;
        connection
            .execute_batch(include_str!(
                "../migrations/0002_accounts_subscriptions.up.sql"
            ))
            .map_err(|error| format!("migrate M2 account database: {error}"))?;
        tracing::info!(data_store = %dir.display(), data_key = if key_was_generated { "generated" } else { "persisted" }, entra_config = "defaults-or-environment", "managed clinic storage ready");
        Ok(Self {
            auth,
            store: ClinicStore {
                connection: Arc::new(Mutex::new(connection)),
                cipher: Arc::new(Aes256Gcm::new_from_slice(&key).map_err(|_| "invalid data key")?),
                key_path: Arc::new(key_path),
                durable_dir: Arc::new(durable_dir),
                backup_dir: Arc::new(backup_dir),
            },
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(12))
                .build()
                .map_err(|error| error.to_string())?,
            billing_base_url: billing_base_url
                .or_else(|| env::var("SOCIOBOT_BILLING_BASE_URL").ok())
                .unwrap_or_else(|| "https://pilot-api.sociobot.in/api/v1".to_owned())
                .trim_end_matches('/')
                .to_owned(),
            provider_fixture_base_url,
        })
    }
}

fn billing_product_url(state: &ClinicState, suffix: &str) -> String {
    format!(
        "{}/products/clinic-reminder-proof/{suffix}",
        state.billing_base_url
    )
}

fn data_dir() -> PathBuf {
    env::var_os("DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/reminder-proof-data"))
}

/// The container image opts into this check while local development keeps
/// portable directories below `DATA_DIR`. A missing Azure Files mount must
/// make a production revision unhealthy instead of accepting clinic records
/// on ephemeral container storage.
fn ensure_required_storage_mounts(durable: &Path, backups: &Path) -> Result<(), String> {
    if env::var("REQUIRE_DURABLE_MOUNTS").ok().as_deref() != Some("1") {
        return Ok(());
    }

    let mount_info = fs::read_to_string("/proc/self/mountinfo")
        .map_err(|error| format!("read container mount information: {error}"))?;
    let missing = missing_required_mounts(&mount_info, &[durable, backups]);
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "required durable storage mounts are missing: {}; refusing unsafe production storage",
            missing.join(", ")
        ))
    }
}

fn missing_required_mounts(mount_info: &str, paths: &[&Path]) -> Vec<String> {
    paths
        .iter()
        .filter_map(|path| {
            let target = path.to_string_lossy();
            let mounted = mount_info.lines().any(|line| {
                line.split_once(" - ")
                    .and_then(|(before_separator, _)| before_separator.split_whitespace().nth(4))
                    == Some(target.as_ref())
            });
            (!mounted).then(|| target.into_owned())
        })
        .collect()
}

fn load_or_create_key(path: &Path) -> Result<[u8; 32], String> {
    if path.exists() {
        let bytes = fs::read(path).map_err(|error| format!("read data key: {error}"))?;
        return bytes
            .try_into()
            .map_err(|_| "stored data key has wrong length".to_owned());
    }
    let mut key = [0_u8; 32];
    rand::rng().fill_bytes(&mut key);
    fs::write(path, key).map_err(|error| format!("persist data key: {error}"))?;
    restrict_path(path, 0o600)?;
    Ok(key)
}

fn restore_durable_pair(key: &Path, database: &Path, durable_dir: &Path) -> Result<(), String> {
    if key.exists() || database.exists() {
        return Ok(());
    }
    let durable_key = durable_dir.join("clinic-data.latest.key");
    let durable_database = durable_dir.join("clinic-data.latest.sqlite3");
    match (durable_key.exists(), durable_database.exists()) {
        (false, false) => Ok(()),
        (true, true) => {
            fs::copy(&durable_key, key).map_err(|error| format!("restore data key: {error}"))?;
            fs::copy(&durable_database, database)
                .map_err(|error| format!("restore clinic database: {error}"))?;
            restrict_path(key, 0o600)?;
            restrict_path(database, 0o600)
        }
        _ => Err(
            "durable clinic snapshot is incomplete; restore its matching key and database"
                .to_owned(),
        ),
    }
}

fn restrict_path(path: &Path, mode: u32) -> Result<(), String> {
    #[cfg(unix)]
    {
        if let Err(error) = fs::set_permissions(path, fs::Permissions::from_mode(mode)) {
            // Azure Files is mounted over SMB in the production Container App
            // and rejects chmod. Access there is enforced by the private share
            // mount and its account credential; local files still receive and
            // are regression-tested for the requested POSIX modes.
            if error.kind() != std::io::ErrorKind::PermissionDenied {
                return Err(format!("restrict {} permissions: {error}", path.display()));
            }
        }
    }
    #[cfg(not(unix))]
    let _ = (path, mode);
    Ok(())
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn validate_text(value: &str, maximum: usize, field: &'static str) -> Result<(), ApiError> {
    if value.trim().is_empty() || value.chars().count() > maximum {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "field_invalid",
            field,
        ));
    }
    Ok(())
}

fn validate_jurisdiction(value: &str) -> Result<(), ApiError> {
    if matches!(value, "us" | "uk" | "eu" | "ca" | "au" | "other") {
        Ok(())
    } else {
        Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "jurisdiction_invalid",
            "Choose the clinic jurisdiction from the list.",
        ))
    }
}

fn validate_retention_days(value: u32) -> Result<(), ApiError> {
    if matches!(value, 30 | 90 | 365) {
        Ok(())
    } else {
        Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "retention_invalid",
            "Choose 30, 90, or 365 days for reminder records.",
        ))
    }
}

fn validate_role(value: &str) -> Result<(), ApiError> {
    if matches!(value, "manager" | "staff" | "viewer") {
        Ok(())
    } else {
        Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "role_invalid",
            "Choose manager, staff, or viewer for this person.",
        ))
    }
}

fn require_owner(
    state: &ClinicState,
    identity: &Identity,
    workspace: &ClinicWorkspace,
) -> Result<(), Response> {
    let role = state
        .store
        .membership_role(&identity.oid, &workspace.organization_id)
        .map_err(IntoResponse::into_response)?;
    if role.as_deref() == Some("owner") || workspace.owner_oid == identity.oid {
        Ok(())
    } else {
        Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "owner_required",
            "Only a clinic owner can complete this action.",
        )
        .into_response())
    }
}

impl ClinicStore {
    fn encrypt(&self, value: &str) -> Result<String, ApiError> {
        let mut nonce = [0_u8; 12];
        rand::rng().fill_bytes(&mut nonce);
        let encrypted = self
            .cipher
            .encrypt(Nonce::from_slice(&nonce), value.as_bytes())
            .map_err(|_| internal())?;
        Ok(format!(
            "{}.{}",
            URL_SAFE_NO_PAD.encode(nonce),
            URL_SAFE_NO_PAD.encode(encrypted)
        ))
    }

    fn decrypt(&self, value: &str) -> Result<String, ApiError> {
        let (nonce, data) = value.split_once('.').ok_or_else(internal)?;
        let nonce = URL_SAFE_NO_PAD.decode(nonce).map_err(|_| internal())?;
        let data = URL_SAFE_NO_PAD.decode(data).map_err(|_| internal())?;
        let clear = self
            .cipher
            .decrypt(Nonce::from_slice(&nonce), data.as_ref())
            .map_err(|_| internal())?;
        String::from_utf8(clear).map_err(|_| internal())
    }

    fn get(&self, oid: &str) -> Result<Option<ClinicWorkspace>, ApiError> {
        let connection = self.connection.lock().map_err(|_| internal())?;
        purge_due_deletions(&connection)?;
        let mut statement = connection
            .prepare(
                "SELECT cw.oid,cw.state_json FROM clinic_workspaces cw
                 WHERE cw.oid=?1 OR cw.organization_id=(
                   SELECT organization_id FROM memberships
                   WHERE user_oid=?1 AND state='active' LIMIT 1
                 )
                 ORDER BY CASE WHEN cw.oid=?1 THEN 0 ELSE 1 END LIMIT 1",
            )
            .map_err(|_| internal())?;
        let mut rows = statement.query(params![oid]).map_err(|_| internal())?;
        let Some(row) = rows.next().map_err(|_| internal())? else {
            return Ok(None);
        };
        let owner_oid: String = row.get(0).map_err(|_| internal())?;
        let encrypted: String = row.get(1).map_err(|_| internal())?;
        let mut workspace: ClinicWorkspace =
            serde_json::from_str(&self.decrypt(&encrypted)?).map_err(|_| internal())?;
        if workspace.owner_oid.is_empty() {
            workspace.owner_oid = owner_oid;
        }
        Ok(Some(workspace))
    }

    fn by_connector(
        &self,
        connector_id: &str,
    ) -> Result<Option<(String, ClinicWorkspace)>, ApiError> {
        let connection = self.connection.lock().map_err(|_| internal())?;
        let mut statement = connection
            .prepare("SELECT oid,state_json FROM clinic_workspaces WHERE connector_id=?1")
            .map_err(|_| internal())?;
        let mut rows = statement
            .query(params![connector_id])
            .map_err(|_| internal())?;
        let Some(row) = rows.next().map_err(|_| internal())? else {
            return Ok(None);
        };
        let oid: String = row.get(0).map_err(|_| internal())?;
        let encrypted: String = row.get(1).map_err(|_| internal())?;
        let workspace = serde_json::from_str(&self.decrypt(&encrypted)?).map_err(|_| internal())?;
        Ok(Some((oid, workspace)))
    }

    fn by_provider(
        &self,
        provider_id: &str,
    ) -> Result<Option<(String, ClinicWorkspace)>, ApiError> {
        let connection = self.connection.lock().map_err(|_| internal())?;
        let mut statement = connection
            .prepare("SELECT oid,state_json FROM clinic_workspaces")
            .map_err(|_| internal())?;
        let mut rows = statement.query([]).map_err(|_| internal())?;
        while let Some(row) = rows.next().map_err(|_| internal())? {
            let oid: String = row.get(0).map_err(|_| internal())?;
            let encrypted: String = row.get(1).map_err(|_| internal())?;
            let workspace: ClinicWorkspace =
                serde_json::from_str(&self.decrypt(&encrypted)?).map_err(|_| internal())?;
            if workspace
                .provider_configs
                .iter()
                .any(|item| item.id == provider_id)
            {
                return Ok(Some((oid, workspace)));
            }
        }
        Ok(None)
    }

    fn save(&self, oid: &str, workspace: &ClinicWorkspace) -> Result<(), ApiError> {
        let mut workspace = workspace.clone();
        if workspace.owner_oid.is_empty() {
            workspace.owner_oid = oid.to_owned();
        }
        let json = serde_json::to_string(&workspace).map_err(|_| internal())?;
        let encrypted = self.encrypt(&json)?;
        let connector_id = workspace.connector.as_ref().map(|item| item.id.as_str());
        let mut connection = self.connection.lock().map_err(|_| internal())?;
        let transaction = connection.transaction().map_err(|_| internal())?;
        transaction.execute("INSERT INTO clinic_workspaces(oid,organization_id,connector_id,state_json,updated_at) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(organization_id) DO UPDATE SET connector_id=excluded.connector_id,state_json=excluded.state_json,updated_at=excluded.updated_at", params![workspace.owner_oid, workspace.organization_id, connector_id, encrypted, now()]).map_err(|_| internal())?;
        sync_account_records(&transaction, &workspace)?;
        transaction.commit().map_err(|_| internal())?;
        self.backup_locked(&connection)?;
        Ok(())
    }

    fn membership_role(
        &self,
        oid: &str,
        organization_id: &str,
    ) -> Result<Option<String>, ApiError> {
        self.connection
            .lock()
            .map_err(|_| internal())?
            .query_row(
                "SELECT role FROM memberships WHERE organization_id=?1 AND user_oid=?2 AND state='active'",
                params![organization_id, oid],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| internal())
    }

    fn record_export(&self, oid: &str, organization_id: &str) -> Result<(), ApiError> {
        let connection = self.connection.lock().map_err(|_| internal())?;
        connection
            .execute(
                "INSERT INTO export_jobs(id,organization_id,requester_oid,state,created_at,expires_at) VALUES(?1,?2,?3,'ready',?4,?5)",
                params![Uuid::new_v4().to_string(), organization_id, oid, now(), now() + 86_400],
            )
            .map_err(|_| internal())?;
        self.backup_locked(&connection)
    }

    fn backup_locked(&self, connection: &Connection) -> Result<(), ApiError> {
        let database_tmp = self.durable_dir.join("clinic-data.latest.sqlite3.tmp");
        let database = self.durable_dir.join("clinic-data.latest.sqlite3");
        let key_tmp = self.durable_dir.join("clinic-data.latest.key.tmp");
        let key = self.durable_dir.join("clinic-data.latest.key");
        connection
            .backup(MAIN_DB, &database_tmp, None)
            .map_err(|_| internal())?;
        fs::copy(self.key_path.as_ref(), &key_tmp).map_err(|_| internal())?;
        restrict_path(&database_tmp, 0o600).map_err(|_| internal())?;
        restrict_path(&key_tmp, 0o600).map_err(|_| internal())?;
        fs::rename(&key_tmp, &key).map_err(|_| internal())?;
        fs::rename(&database_tmp, &database).map_err(|_| internal())?;
        let today = now() / 86_400;
        let daily_database = self
            .backup_dir
            .join(format!("clinic-data.day-{today}.sqlite3"));
        let daily_key = self.backup_dir.join(format!("clinic-data.day-{today}.key"));
        if !daily_database.exists() {
            fs::copy(&database, &daily_database).map_err(|_| internal())?;
            fs::copy(&key, &daily_key).map_err(|_| internal())?;
            restrict_path(&daily_database, 0o600).map_err(|_| internal())?;
            restrict_path(&daily_key, 0o600).map_err(|_| internal())?;
        }
        self.prune_daily_backups(today)?;
        Ok(())
    }

    fn prune_daily_backups(&self, today: u64) -> Result<(), ApiError> {
        for entry in fs::read_dir(self.backup_dir.as_ref()).map_err(|_| internal())? {
            let entry = entry.map_err(|_| internal())?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            let Some(day) = name
                .strip_prefix("clinic-data.day-")
                .and_then(|suffix| suffix.split('.').next())
                .and_then(|value| value.parse::<u64>().ok())
            else {
                continue;
            };
            // Retain today's point plus the preceding 30 complete UTC days.
            // A pair exactly 30 days old is still within the published
            // retention window; only older pairs are removed.
            if day.saturating_add(30) < today {
                fs::remove_file(entry.path()).map_err(|_| internal())?;
            }
        }
        Ok(())
    }

    fn receipt_once(&self, event_id: &str, organization_id: &str) -> Result<bool, ApiError> {
        let changed = self.connection.lock().map_err(|_| internal())?.execute("INSERT OR IGNORE INTO provider_receipts(provider_event_id,organization_id,received_at) VALUES(?1,?2,?3)", params![event_id, organization_id, now()]).map_err(|_| internal())?;
        Ok(changed == 1)
    }

    fn delete(&self, oid: &str) -> Result<(), ApiError> {
        let connection = self.connection.lock().map_err(|_| internal())?;
        let organization_id = connection
            .query_row(
                "SELECT organization_id FROM clinic_workspaces WHERE oid=?1",
                params![oid],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|_| internal())?;
        connection
            .execute("DELETE FROM clinic_workspaces WHERE oid=?1", params![oid])
            .map_err(|_| internal())?;
        if let Some(organization_id) = organization_id {
            connection
                .execute(
                    "DELETE FROM organizations WHERE id=?1",
                    params![organization_id],
                )
                .map_err(|_| internal())?;
        }
        self.backup_locked(&connection)
    }
}

fn sync_account_records(
    transaction: &Transaction<'_>,
    workspace: &ClinicWorkspace,
) -> Result<(), ApiError> {
    let timestamp = now();
    let owner_name = workspace
        .members
        .iter()
        .find(|member| member.role == "owner")
        .map(|member| member.display_name.as_str())
        .unwrap_or("Clinic owner");
    transaction
        .execute(
            "INSERT INTO users(oid,display_name,last_sign_in) VALUES(?1,?2,?3)
             ON CONFLICT(oid) DO UPDATE SET display_name=excluded.display_name,last_sign_in=excluded.last_sign_in",
            params![workspace.owner_oid, owner_name, timestamp],
        )
        .map_err(|_| internal())?;
    transaction
        .execute(
            "INSERT INTO organizations(id,owner_oid,display_name,jurisdiction,retention_days,deletion_scheduled_at,deletion_cancel_until,created_at,updated_at)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?8)
             ON CONFLICT(id) DO UPDATE SET display_name=excluded.display_name,jurisdiction=excluded.jurisdiction,
               retention_days=excluded.retention_days,deletion_scheduled_at=excluded.deletion_scheduled_at,
               deletion_cancel_until=excluded.deletion_cancel_until,updated_at=excluded.updated_at",
            params![
                workspace.organization_id,
                workspace.owner_oid,
                workspace.clinic_name,
                workspace.jurisdiction,
                workspace.retention_days,
                workspace.deletion.as_ref().map(|item| item.scheduled_at),
                workspace.deletion.as_ref().map(|item| item.cancel_until),
                timestamp
            ],
        )
        .map_err(|_| internal())?;
    let location_id = format!("location:{}", workspace.organization_id);
    transaction
        .execute(
            "INSERT INTO locations(id,organization_id,name,timezone,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?5)
             ON CONFLICT(id) DO UPDATE SET name=excluded.name,timezone=excluded.timezone,updated_at=excluded.updated_at",
            params![location_id, workspace.organization_id, workspace.location_name, workspace.timezone, timestamp],
        )
        .map_err(|_| internal())?;

    let owner_membership_id = format!("membership:{}:owner", workspace.organization_id);
    transaction
        .execute(
            "INSERT INTO memberships(id,organization_id,user_oid,display_name,email,role,state,created_at,updated_at)
             VALUES(?1,?2,?3,?4,'','owner','active',?5,?5)
             ON CONFLICT(id) DO UPDATE SET user_oid=excluded.user_oid,display_name=excluded.display_name,updated_at=excluded.updated_at",
            params![owner_membership_id, workspace.organization_id, workspace.owner_oid, owner_name, timestamp],
        )
        .map_err(|_| internal())?;
    transaction
        .execute(
            "INSERT INTO notification_preferences(membership_id,digest_enabled,exception_email,updated_at)
             VALUES(?1,0,0,?2) ON CONFLICT(membership_id) DO NOTHING",
            params![owner_membership_id, timestamp],
        )
        .map_err(|_| internal())?;

    transaction
        .execute(
            "DELETE FROM memberships WHERE organization_id=?1 AND role<>'owner'",
            params![workspace.organization_id],
        )
        .map_err(|_| internal())?;

    for member in &workspace.members {
        if member.role == "owner" {
            continue;
        }
        if let Some(user_oid) = member.user_oid.as_deref() {
            transaction
                .execute(
                    "INSERT INTO users(oid,display_name,last_sign_in) VALUES(?1,?2,?3)
                     ON CONFLICT(oid) DO UPDATE SET display_name=excluded.display_name",
                    params![user_oid, member.display_name, timestamp],
                )
                .map_err(|_| internal())?;
        }
        transaction
            .execute(
                "INSERT INTO memberships(id,organization_id,user_oid,display_name,email,role,state,created_at,updated_at)
                 VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?8)
                 ON CONFLICT(id) DO UPDATE SET user_oid=excluded.user_oid,display_name=excluded.display_name,
                   email=excluded.email,role=excluded.role,state=excluded.state,updated_at=excluded.updated_at",
                params![member.id, workspace.organization_id, member.user_oid, member.display_name, member.email, member.role, member.state, timestamp],
            )
            .map_err(|_| internal())?;
    }

    let entitlement_hash = workspace
        .subscription
        .encrypted_entitlement
        .as_deref()
        .map(|value| format!("{:x}", Sha256::digest(value.as_bytes())));
    transaction
        .execute(
            "INSERT INTO subscriptions(id,organization_id,entitlement_hash,tier,status,checked_at,expires_at,updated_at)
             VALUES(?1,?2,?3,?4,?5,?6,NULL,?7)
             ON CONFLICT(organization_id) DO UPDATE SET entitlement_hash=excluded.entitlement_hash,tier=excluded.tier,
               status=excluded.status,checked_at=excluded.checked_at,updated_at=excluded.updated_at",
            params![
                format!("subscription:{}", workspace.organization_id),
                workspace.organization_id,
                entitlement_hash,
                workspace.subscription.tier,
                workspace.subscription.status.as_deref().unwrap_or("none"),
                workspace.subscription.checked_at,
                timestamp
            ],
        )
        .map_err(|_| internal())?;

    for event in &workspace.audit {
        let id_source = format!(
            "{}:{}:{}:{}:{}",
            workspace.organization_id, event.at, event.actor, event.action, event.target
        );
        let event_id = format!("{:x}", Sha256::digest(id_source.as_bytes()));
        transaction
            .execute(
                "INSERT OR IGNORE INTO audit_events(id,organization_id,actor_oid,action,target,occurred_at)
                 VALUES(?1,?2,?3,?4,?5,?6)",
                params![event_id, workspace.organization_id, event.actor, event.action, event.target, event.at],
            )
            .map_err(|_| internal())?;
    }
    Ok(())
}

fn purge_due_deletions(connection: &Connection) -> Result<(), ApiError> {
    let mut statement = connection
        .prepare("SELECT id FROM organizations WHERE deletion_cancel_until IS NOT NULL AND deletion_cancel_until<=?1")
        .map_err(|_| internal())?;
    let due = statement
        .query_map(params![now()], |row| row.get::<_, String>(0))
        .map_err(|_| internal())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| internal())?;
    drop(statement);
    for organization_id in due {
        connection
            .execute(
                "DELETE FROM clinic_workspaces WHERE organization_id=?1",
                params![organization_id],
            )
            .map_err(|_| internal())?;
        connection
            .execute(
                "DELETE FROM organizations WHERE id=?1",
                params![organization_id],
            )
            .map_err(|_| internal())?;
    }
    Ok(())
}

fn internal() -> ApiError {
    ApiError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "clinic_store_failed",
        "Clinic data could not be saved. Try again and contact support if this continues.",
    )
}
fn missing() -> ApiError {
    ApiError::new(
        StatusCode::NOT_FOUND,
        "clinic_missing",
        "Create your clinic workspace before using this action.",
    )
}

async fn identity(state: &ClinicState, headers: &HeaderMap) -> Result<Identity, Response> {
    state
        .auth
        .identity(headers)
        .await
        .map_err(IntoResponse::into_response)
}

pub async fn auth_config(State(state): State<ClinicState>) -> Json<AuthConfig> {
    Json(state.auth.public_config())
}

pub async fn get_me(
    State(state): State<ClinicState>,
    headers: HeaderMap,
) -> Result<Json<MeResponse>, Response> {
    let identity = identity(&state, &headers).await?;
    let workspace = state
        .store
        .get(&identity.oid)
        .map_err(IntoResponse::into_response)?;
    let organization_id = workspace
        .as_ref()
        .map(|workspace| workspace.organization_id.clone());
    let role = if let Some(workspace) = workspace.as_ref() {
        state
            .store
            .membership_role(&identity.oid, &workspace.organization_id)
            .map_err(IntoResponse::into_response)?
    } else {
        None
    };
    Ok(Json(MeResponse {
        oid: identity.oid,
        organization_id,
        role,
    }))
}

pub async fn list_organizations(
    State(state): State<ClinicState>,
    headers: HeaderMap,
) -> Result<Json<Vec<OrganizationSummary>>, Response> {
    let identity = identity(&state, &headers).await?;
    let organizations = state
        .store
        .get(&identity.oid)
        .map_err(IntoResponse::into_response)?
        .into_iter()
        .map(|workspace| OrganizationSummary {
            id: workspace.organization_id,
            name: workspace.clinic_name,
            jurisdiction: workspace.jurisdiction,
            retention_days: workspace.retention_days,
        })
        .collect();
    Ok(Json(organizations))
}

pub async fn list_locations(
    State(state): State<ClinicState>,
    headers: HeaderMap,
) -> Result<Json<Vec<LocationSummary>>, Response> {
    let identity = identity(&state, &headers).await?;
    let workspace = state
        .store
        .get(&identity.oid)
        .map_err(IntoResponse::into_response)?
        .ok_or_else(|| missing().into_response())?;
    Ok(Json(vec![LocationSummary {
        id: format!("location:{}", workspace.organization_id),
        organization_id: workspace.organization_id,
        name: workspace.location_name,
        timezone: workspace.timezone,
    }]))
}

pub async fn update_location(
    State(state): State<ClinicState>,
    headers: HeaderMap,
    Json(input): Json<LocationInput>,
) -> Result<Json<WorkspaceResponse>, Response> {
    let identity = identity(&state, &headers).await?;
    let mut workspace = state
        .store
        .get(&identity.oid)
        .map_err(IntoResponse::into_response)?
        .ok_or_else(|| missing().into_response())?;
    require_owner(&state, &identity, &workspace)?;
    validate_text(
        &input.name,
        100,
        "Enter a location name up to 100 characters.",
    )
    .map_err(IntoResponse::into_response)?;
    validate_text(
        &input.timezone,
        64,
        "Enter an IANA timezone such as Europe/London.",
    )
    .map_err(IntoResponse::into_response)?;
    workspace.location_name = input.name.trim().to_owned();
    workspace.timezone = input.timezone.trim().to_owned();
    workspace.audit.push(AuditEvent {
        at: now(),
        actor: identity.oid.clone(),
        action: "location.saved".to_owned(),
        target: workspace.organization_id.clone(),
    });
    state
        .store
        .save(&identity.oid, &workspace)
        .map_err(IntoResponse::into_response)?;
    Ok(Json(workspace.into()))
}

pub async fn list_memberships(
    State(state): State<ClinicState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ClinicMember>>, Response> {
    let identity = identity(&state, &headers).await?;
    let workspace = state
        .store
        .get(&identity.oid)
        .map_err(IntoResponse::into_response)?
        .ok_or_else(|| missing().into_response())?;
    Ok(Json(workspace.members))
}

pub async fn add_membership(
    State(state): State<ClinicState>,
    headers: HeaderMap,
    Json(input): Json<MemberInput>,
) -> Result<(StatusCode, Json<ClinicMember>), Response> {
    let identity = identity(&state, &headers).await?;
    let mut workspace = state
        .store
        .get(&identity.oid)
        .map_err(IntoResponse::into_response)?
        .ok_or_else(|| missing().into_response())?;
    require_owner(&state, &identity, &workspace)?;
    validate_text(
        &input.display_name,
        100,
        "Enter a staff name up to 100 characters.",
    )
    .map_err(IntoResponse::into_response)?;
    validate_role(&input.role).map_err(IntoResponse::into_response)?;
    let member = ClinicMember {
        id: Uuid::new_v4().to_string(),
        state: if input.user_oid.is_some() {
            "active".to_owned()
        } else {
            "pending".to_owned()
        },
        user_oid: input.user_oid,
        display_name: input.display_name.trim().to_owned(),
        email: input.email.trim().to_owned(),
        role: input.role,
    };
    workspace.members.push(member.clone());
    workspace.audit.push(AuditEvent {
        at: now(),
        actor: identity.oid.clone(),
        action: "membership.created".to_owned(),
        target: member.id.clone(),
    });
    state
        .store
        .save(&identity.oid, &workspace)
        .map_err(IntoResponse::into_response)?;
    Ok((StatusCode::CREATED, Json(member)))
}

pub async fn get_workspace(
    State(state): State<ClinicState>,
    headers: HeaderMap,
) -> Result<Json<WorkspaceResponse>, Response> {
    let identity = identity(&state, &headers).await?;
    let workspace = state
        .store
        .get(&identity.oid)
        .map_err(IntoResponse::into_response)?
        .ok_or_else(|| missing().into_response())?;
    Ok(Json(workspace.into()))
}

pub async fn onboard(
    State(state): State<ClinicState>,
    headers: HeaderMap,
    Json(input): Json<OnboardInput>,
) -> Result<(StatusCode, Json<WorkspaceResponse>), Response> {
    let identity = identity(&state, &headers).await?;
    validate_text(
        &input.clinic_name,
        100,
        "Enter a clinic name up to 100 characters.",
    )
    .map_err(IntoResponse::into_response)?;
    validate_text(
        &input.location_name,
        100,
        "Enter a location name up to 100 characters.",
    )
    .map_err(IntoResponse::into_response)?;
    validate_text(
        &input.timezone,
        64,
        "Enter an IANA timezone such as Europe/London.",
    )
    .map_err(IntoResponse::into_response)?;
    let jurisdiction = input.jurisdiction.unwrap_or_else(default_jurisdiction);
    let retention_days = input.retention_days.unwrap_or_else(default_retention_days);
    validate_jurisdiction(&jurisdiction).map_err(IntoResponse::into_response)?;
    validate_retention_days(retention_days).map_err(IntoResponse::into_response)?;
    for member in &input.staff {
        validate_text(
            &member.display_name,
            100,
            "Enter each staff name using 100 characters or fewer.",
        )
        .map_err(IntoResponse::into_response)?;
        if !member.email.is_empty() {
            validate_text(
                &member.email,
                254,
                "Enter each staff email using 254 characters or fewer.",
            )
            .map_err(IntoResponse::into_response)?;
        }
        validate_role(&member.role).map_err(IntoResponse::into_response)?;
    }
    let mut workspace = state
        .store
        .get(&identity.oid)
        .map_err(IntoResponse::into_response)?
        .unwrap_or_else(|| ClinicWorkspace {
            owner_oid: identity.oid.clone(),
            organization_id: Uuid::new_v4().to_string(),
            ..Default::default()
        });
    if !workspace.owner_oid.is_empty() && workspace.owner_oid != identity.oid {
        require_owner(&state, &identity, &workspace)?;
    }
    workspace.clinic_name = input.clinic_name.trim().to_owned();
    workspace.location_name = input.location_name.trim().to_owned();
    workspace.timezone = input.timezone.trim().to_owned();
    workspace.jurisdiction = jurisdiction;
    workspace.retention_days = retention_days;
    let owner_name = input
        .owner_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Clinic owner")
        .to_owned();
    let mut members = vec![ClinicMember {
        id: format!("membership:{}:owner", workspace.organization_id),
        user_oid: Some(workspace.owner_oid.clone()),
        display_name: owner_name,
        email: String::new(),
        role: "owner".to_owned(),
        state: "active".to_owned(),
    }];
    members.extend(input.staff.into_iter().map(|member| ClinicMember {
        id: Uuid::new_v4().to_string(),
        state: if member.user_oid.is_some() {
            "active".to_owned()
        } else {
            "pending".to_owned()
        },
        user_oid: member.user_oid,
        display_name: member.display_name.trim().to_owned(),
        email: member.email.trim().to_owned(),
        role: member.role,
    }));
    workspace.members = members;
    workspace.audit.push(AuditEvent {
        at: now(),
        actor: identity.oid.clone(),
        action: "clinic.saved".to_owned(),
        target: workspace.organization_id.clone(),
    });
    state
        .store
        .save(&identity.oid, &workspace)
        .map_err(IntoResponse::into_response)?;
    Ok((StatusCode::CREATED, Json(workspace.into())))
}

pub async fn configure_connector(
    State(state): State<ClinicState>,
    headers: HeaderMap,
    Json(input): Json<ConnectorInput>,
) -> Result<(StatusCode, Json<ConnectorCreated>), Response> {
    let identity = identity(&state, &headers).await?;
    if input.kind != "signed-calendar-webhook" {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "connector_invalid",
            "Choose the signed calendar webhook connector.",
        )
        .into_response());
    }
    validate_text(
        &input.webhook_secret,
        200,
        "Use a webhook signing secret between 16 and 200 characters.",
    )
    .map_err(IntoResponse::into_response)?;
    if input.webhook_secret.len() < 16 {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "connector_secret_weak",
            "Use a webhook signing secret with at least 16 characters.",
        )
        .into_response());
    }
    let mut workspace = state
        .store
        .get(&identity.oid)
        .map_err(IntoResponse::into_response)?
        .ok_or_else(|| missing().into_response())?;
    let connector = ConnectorPublic {
        id: Uuid::new_v4().to_string(),
        kind: input.kind,
        connected_at: now(),
        last_received_at: None,
    };
    let stored = state
        .store
        .encrypt(&input.webhook_secret)
        .map_err(IntoResponse::into_response)?;
    workspace.audit.push(AuditEvent {
        at: now(),
        actor: identity.oid.clone(),
        action: "connector.configured".to_owned(),
        target: connector.id.clone(),
    });
    workspace.connector = Some(connector.clone());
    workspace.encrypted_connector_secret = Some(stored);
    state
        .store
        .save(&identity.oid, &workspace)
        .map_err(IntoResponse::into_response)?;
    Ok((
        StatusCode::CREATED,
        Json(ConnectorCreated {
            connector,
            signing_secret: input.webhook_secret,
            intake_url: "/api/v1/connectors/intake",
        }),
    ))
}

fn connector_secret(state: &ClinicState, workspace: &ClinicWorkspace) -> Result<String, ApiError> {
    let encrypted = workspace
        .encrypted_connector_secret
        .as_deref()
        .ok_or_else(internal)?;
    state.store.decrypt(encrypted)
}

pub async fn connector_intake(
    State(state): State<ClinicState>,
    headers: HeaderMap,
    Json(input): Json<IntakeInput>,
) -> Result<Json<WorkspaceResponse>, Response> {
    let Some((oid, mut workspace)) = state
        .store
        .by_connector(&input.connector_id)
        .map_err(IntoResponse::into_response)?
    else {
        return Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "connector_missing",
            "This calendar connector is not active.",
        )
        .into_response());
    };
    let timestamp = headers
        .get("x-reminder-timestamp")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::UNAUTHORIZED,
                "signature_missing",
                "Send a signed connector timestamp and signature.",
            )
            .into_response()
        })?;
    if now().abs_diff(timestamp) > 300 {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "signature_expired",
            "The connector signature is older than five minutes.",
        )
        .into_response());
    }
    let signature = headers
        .get("x-reminder-signature")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::UNAUTHORIZED,
                "signature_missing",
                "Send a signed connector timestamp and signature.",
            )
            .into_response()
        })?;
    let secret = connector_secret(&state, &workspace).map_err(IntoResponse::into_response)?;
    let canonical = format!(
        "{timestamp}:{}:{}",
        input.connector_id,
        input.appointments.len()
    );
    if !valid_signature(&secret, canonical.as_bytes(), signature) {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "signature_invalid",
            "The calendar connector signature is invalid.",
        )
        .into_response());
    }
    for item in input.appointments {
        validate_appointment(&item).map_err(IntoResponse::into_response)?;
        if let Some(existing) = workspace
            .reminders
            .iter_mut()
            .find(|reminder| reminder.source_id == item.source_id)
        {
            existing.patient_alias = item.patient_alias;
            existing.first_name = item.first_name;
            existing.appointment_time = item.appointment_time;
            existing.channels = item.channels;
        } else {
            workspace.reminders.push(ClinicReminder {
                id: Uuid::new_v4().to_string(),
                source_id: item.source_id,
                patient_alias: item.patient_alias,
                first_name: item.first_name,
                appointment_time: item.appointment_time,
                status: "scheduled".to_owned(),
                channels: item.channels,
                timeline: vec![ClinicEvent {
                    at: now(),
                    kind: "source".to_owned(),
                    channel: None,
                    outcome: "Appointment received from signed calendar connector".to_owned(),
                    provider_reference: None,
                    provider_code: None,
                }],
                exception: None,
            });
        }
    }
    if let Some(connector) = workspace.connector.as_mut() {
        connector.last_received_at = Some(now());
    }
    workspace.audit.push(AuditEvent {
        at: now(),
        actor: "calendar-connector".to_owned(),
        action: "appointments.upserted".to_owned(),
        target: workspace.organization_id.clone(),
    });
    state
        .store
        .save(&oid, &workspace)
        .map_err(IntoResponse::into_response)?;
    Ok(Json(workspace.into()))
}

fn validate_appointment(input: &AppointmentInput) -> Result<(), ApiError> {
    validate_text(&input.source_id, 100, "Each appointment needs a source ID.")?;
    validate_text(
        &input.patient_alias,
        80,
        "Each appointment needs a short patient alias.",
    )?;
    validate_text(
        &input.first_name,
        80,
        "Each appointment needs a first name for the approved template.",
    )?;
    validate_text(
        &input.appointment_time,
        80,
        "Each appointment needs a date and time.",
    )?;
    if input.channels.is_empty() || input.channels.len() > 3 {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "channels_invalid",
            "Provide one to three channels in fallback order.",
        ));
    }
    for channel in &input.channels {
        if !matches!(channel.channel.as_str(), "sms" | "email" | "whatsapp")
            || !matches!(channel.consent.as_str(), "allowed" | "blocked" | "unknown")
        {
            return Err(ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "consent_invalid",
                "Each channel needs an allowed, blocked, or unknown consent state.",
            ));
        }
        validate_text(
            &channel.consent_source,
            100,
            "Record where channel consent came from.",
        )?;
        validate_text(
            &channel.consent_captured_at,
            80,
            "Record when channel consent was captured.",
        )?;
        let destination_is_valid = match channel.channel.as_str() {
            "email" => {
                let value = channel.destination.trim();
                value.contains('@')
                    && !value.starts_with('@')
                    && !value.ends_with('@')
                    && value.len() <= 254
            }
            "sms" | "whatsapp" => {
                let digits = channel
                    .destination
                    .chars()
                    .filter(|value| value.is_ascii_digit())
                    .count();
                channel.destination.starts_with('+') && (8..=15).contains(&digits)
            }
            _ => false,
        };
        if !destination_is_valid {
            return Err(ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "destination_invalid",
                "Use a valid email address or international phone number for the approved channel.",
            ));
        }
    }
    Ok(())
}

pub async fn configure_provider(
    State(state): State<ClinicState>,
    headers: HeaderMap,
    Json(input): Json<ProviderInput>,
) -> Result<(StatusCode, Json<WorkspaceResponse>), Response> {
    let identity = identity(&state, &headers).await?;
    let valid = matches!(
        (input.channel.as_str(), input.kind.as_str()),
        ("sms", "twilio") | ("whatsapp", "twilio") | ("email", "resend")
    );
    if !valid {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "provider_invalid",
            "Use Twilio for SMS or approved WhatsApp, or Resend for email.",
        )
        .into_response());
    }
    for (value, message) in [
        (&input.secret, "Enter the messaging-provider credential."),
        (&input.from, "Enter the approved sender."),
        (
            &input.approved_template_id,
            "Enter the approved template ID.",
        ),
        (&input.webhook_secret, "Enter a webhook signing secret."),
    ] {
        validate_text(value, 300, message).map_err(IntoResponse::into_response)?;
    }
    let mut workspace = state
        .store
        .get(&identity.oid)
        .map_err(IntoResponse::into_response)?
        .ok_or_else(|| missing().into_response())?;
    workspace
        .provider_configs
        .retain(|item| item.channel != input.channel);
    workspace.provider_configs.push(StoredProvider {
        id: Uuid::new_v4().to_string(),
        channel: input.channel,
        kind: input.kind,
        account_id: input.account_id,
        encrypted_secret: state
            .store
            .encrypt(&input.secret)
            .map_err(IntoResponse::into_response)?,
        from: input.from,
        approved_template_id: input.approved_template_id,
        encrypted_webhook_secret: state
            .store
            .encrypt(&input.webhook_secret)
            .map_err(IntoResponse::into_response)?,
    });
    workspace.audit.push(AuditEvent {
        at: now(),
        actor: identity.oid.clone(),
        action: "provider.configured".to_owned(),
        target: workspace.organization_id.clone(),
    });
    state
        .store
        .save(&identity.oid, &workspace)
        .map_err(IntoResponse::into_response)?;
    Ok((StatusCode::CREATED, Json(workspace.into())))
}

pub async fn dispatch(
    State(state): State<ClinicState>,
    headers: HeaderMap,
    Json(input): Json<DispatchInput>,
) -> Result<Json<WorkspaceResponse>, Response> {
    let identity = identity(&state, &headers).await?;
    let idempotency = headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .filter(|value| value.len() >= 8)
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::BAD_REQUEST,
                "idempotency_required",
                "Send an Idempotency-Key of at least eight characters.",
            )
            .into_response()
        })?;
    let mut workspace = state
        .store
        .get(&identity.oid)
        .map_err(IntoResponse::into_response)?
        .ok_or_else(|| missing().into_response())?;
    let index = workspace
        .reminders
        .iter()
        .position(|item| item.id == input.reminder_id)
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::NOT_FOUND,
                "reminder_missing",
                "Choose a reminder from this clinic.",
            )
            .into_response()
        })?;
    if workspace.reminders[index]
        .timeline
        .iter()
        .any(|event| event.provider_code.as_deref() == Some(idempotency))
    {
        return Ok(Json(workspace.into()));
    }
    let plans = eligible_channels(&workspace.reminders[index]);
    if plans.is_empty() {
        open_exception(
            &mut workspace.reminders[index],
            "No allowed channel has recorded consent.",
        );
    } else {
        ensure_active_subscription(&state, &mut workspace)
            .await
            .map_err(IntoResponse::into_response)?;
        let mut delivered = false;
        for channel in plans {
            let Some(provider) = workspace
                .provider_configs
                .iter()
                .find(|item| item.channel == channel.channel)
                .cloned()
            else {
                continue;
            };
            let result = send_provider(
                &state,
                &workspace,
                &workspace.reminders[index],
                &channel,
                &provider,
                idempotency,
            )
            .await;
            let event = match result {
                Ok(reference) => {
                    delivered = true;
                    ClinicEvent {
                        at: now(),
                        kind: "provider-accepted".to_owned(),
                        channel: Some(channel.channel.clone()),
                        outcome:
                            "Messaging provider accepted the approved reminder; delivery receipt is pending"
                                .to_owned(),
                        provider_reference: Some(reference),
                        provider_code: Some(idempotency.to_owned()),
                    }
                }
                Err(code) => ClinicEvent {
                    at: now(),
                    kind: "provider-failed".to_owned(),
                    channel: Some(channel.channel.clone()),
                    outcome:
                        "Messaging provider rejected this attempt; fallback evaluation continued"
                            .to_owned(),
                    provider_reference: None,
                    provider_code: Some(format!("{idempotency}:{code}")),
                },
            };
            workspace.reminders[index].timeline.push(event);
            if delivered {
                workspace.reminders[index].status = "provider-accepted".to_owned();
                break;
            }
        }
        if !delivered {
            open_exception(
                &mut workspace.reminders[index],
                "No configured, consented messaging provider accepted this reminder.",
            );
        }
    }
    workspace.audit.push(AuditEvent {
        at: now(),
        actor: identity.oid.clone(),
        action: "reminder.dispatch".to_owned(),
        target: input.reminder_id,
    });
    state
        .store
        .save(&identity.oid, &workspace)
        .map_err(IntoResponse::into_response)?;
    Ok(Json(workspace.into()))
}

async fn ensure_active_subscription(
    state: &ClinicState,
    workspace: &mut ClinicWorkspace,
) -> Result<(), ApiError> {
    if !matches!(
        workspace.subscription.status.as_deref(),
        Some("active" | "grace")
    ) {
        return Err(ApiError::new(
            StatusCode::PAYMENT_REQUIRED,
            "subscription_required",
            "Activate the Clinic plan through Sociobot before dispatching reminders.",
        ));
    }
    if workspace
        .subscription
        .checked_at
        .is_some_and(|checked| now().saturating_sub(checked) < 86_400)
    {
        return Ok(());
    }
    let encrypted = workspace
        .subscription
        .encrypted_entitlement
        .as_deref()
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::PAYMENT_REQUIRED,
                "subscription_refresh_required",
                "Recheck your Clinic subscription through Sociobot before dispatching reminders.",
            )
        })?;
    let token = state.store.decrypt(encrypted)?;
    let response = state
        .client
        .get(billing_product_url(state, "verify"))
        .query(&[("license", token)])
        .send()
        .await
        .map_err(|_| {
            ApiError::new(
                StatusCode::BAD_GATEWAY,
                "billing_unavailable",
                "Sociobot billing could not be reached. Try again shortly.",
            )
        })?;
    let verdict = response.json::<serde_json::Value>().await.map_err(|_| {
        ApiError::new(
            StatusCode::BAD_GATEWAY,
            "billing_invalid",
            "Sociobot billing returned an unreadable response.",
        )
    })?;
    let status = subscription_status(&verdict);
    if !matches!(status, "active" | "grace") {
        workspace.subscription.status = Some(status.to_owned());
        workspace.subscription.checked_at = Some(now());
        state.store.save(&workspace.owner_oid, workspace)?;
        return Err(ApiError::new(
            StatusCode::PAYMENT_REQUIRED,
            "subscription_inactive",
            "This subscription is not active. Check billing in Sociobot.",
        ));
    }
    workspace.subscription.status = Some(status.to_owned());
    workspace.subscription.checked_at = Some(now());
    Ok(())
}

fn subscription_status(verdict: &serde_json::Value) -> &'static str {
    if let Some(status) = verdict.get("status").and_then(|value| value.as_str()) {
        if matches!(
            status,
            "active" | "grace" | "past_due" | "cancelled" | "revoked"
        ) {
            return match status {
                "active" => "active",
                "grace" => "grace",
                "past_due" => "past_due",
                "cancelled" => "cancelled",
                _ => "revoked",
            };
        }
    }
    if verdict.get("valid").and_then(|value| value.as_bool()) == Some(true) {
        return "active";
    }
    match verdict.get("reason").and_then(|value| value.as_str()) {
        Some("revoked") | Some("wrong_product") => "revoked",
        Some("expired") => "cancelled",
        _ => "past_due",
    }
}

fn eligible_channels(reminder: &ClinicReminder) -> Vec<ReminderChannel> {
    reminder
        .channels
        .iter()
        .filter(|channel| channel.consent == "allowed")
        .cloned()
        .collect()
}

fn untried_eligible_channels(reminder: &ClinicReminder) -> Vec<ReminderChannel> {
    reminder
        .channels
        .iter()
        .filter(|channel| channel.consent == "allowed")
        .filter(|channel| {
            !reminder.timeline.iter().any(|event| {
                matches!(event.kind.as_str(), "provider-accepted" | "provider-failed")
                    && event.channel.as_deref() == Some(channel.channel.as_str())
            })
        })
        .cloned()
        .collect()
}

fn open_exception(reminder: &mut ClinicReminder, reason: &str) {
    reminder.status = "exception".to_owned();
    reminder.exception = Some(ClinicException {
        id: Uuid::new_v4().to_string(),
        reason: reason.to_owned(),
        owner: None,
        state: "open".to_owned(),
        resolution: None,
    });
    reminder.timeline.push(ClinicEvent {
        at: now(),
        kind: "exception".to_owned(),
        channel: None,
        outcome: reason.to_owned(),
        provider_reference: None,
        provider_code: None,
    });
}

async fn send_provider(
    state: &ClinicState,
    workspace: &ClinicWorkspace,
    reminder: &ClinicReminder,
    channel: &ReminderChannel,
    provider: &StoredProvider,
    idempotency: &str,
) -> Result<String, String> {
    let secret = state
        .store
        .decrypt(&provider.encrypted_secret)
        .map_err(|_| "credential".to_owned())?;
    if let Some(base) = &state.provider_fixture_base_url {
        let response = state
            .client
            .post(format!(
                "{}/send/{}",
                base.trim_end_matches('/'),
                channel.channel
            ))
            .header("Idempotency-Key", idempotency)
            .bearer_auth(&secret)
            .json(&serde_json::json!({
                "from": provider.from,
                "to": channel.destination,
                "template": provider.approved_template_id,
                "first_name": reminder.first_name,
                "clinic": workspace.clinic_name,
                "appointment_time": reminder.appointment_time,
            }))
            .send()
            .await
            .map_err(|_| "network".to_owned())?;
        let status = response.status();
        let value: serde_json::Value = response.json().await.unwrap_or_default();
        return if status.is_success() {
            value
                .get("id")
                .and_then(|value| value.as_str())
                .map(str::to_owned)
                .ok_or_else(|| "reference-missing".to_owned())
        } else {
            Err(format!("http-{}", status.as_u16()))
        };
    }
    let message = appointment_reminder_message(reminder, workspace);
    let response = if provider.kind == "twilio" {
        let endpoint = format!("https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json", provider.account_id);
        let mut form = vec![("From", provider.from.clone()), ("To", if channel.channel == "whatsapp" { format!("whatsapp:{}", channel.destination) } else { channel.destination.clone() })];
        let public_origin = env::var("PUBLIC_ORIGIN").unwrap_or_else(|_| "https://clinic-reminder-proof.sociobot.in".to_owned());
        form.push(("StatusCallback", format!("{public_origin}/api/v1/providers/twilio/{}/receipts", provider.id)));
        if channel.channel == "whatsapp" { form.push(("ContentSid", provider.approved_template_id.clone())); form.push(("ContentVariables", serde_json::json!({"1": reminder.first_name, "2": workspace.clinic_name, "3": reminder.appointment_time}).to_string())); } else { form.push(("Body", message)); }
        state.client.post(endpoint).basic_auth(&provider.account_id, Some(secret)).header("Idempotency-Key", idempotency).form(&form).send().await
    } else {
        state.client.post("https://api.resend.com/emails").bearer_auth(secret).header("Idempotency-Key", idempotency).json(&serde_json::json!({"from": provider.from, "to": [channel.destination], "subject": "Appointment reminder", "text": message, "headers": {"X-Entity-Ref-ID": idempotency}})).send().await
    }.map_err(|_| "network".to_owned())?;
    let status = response.status();
    let value: serde_json::Value = response.json().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("http-{}", status.as_u16()));
    }
    value
        .get("sid")
        .or_else(|| value.get("id"))
        .and_then(|value| value.as_str())
        .map(str::to_owned)
        .ok_or_else(|| "reference-missing".to_owned())
}

fn appointment_reminder_message(reminder: &ClinicReminder, workspace: &ClinicWorkspace) -> String {
    format!(
        "Hi {}, reminder: appointment at {} on {}. Reply STOP to opt out.",
        reminder.first_name, workspace.clinic_name, reminder.appointment_time
    )
}

pub async fn provider_receipt(
    State(state): State<ClinicState>,
    AxumPath(connector_id): AxumPath<String>,
    headers: HeaderMap,
    Json(input): Json<ReceiptInput>,
) -> Result<StatusCode, Response> {
    let Some((_oid, workspace)) = state
        .store
        .by_provider(&connector_id)
        .map_err(IntoResponse::into_response)?
    else {
        return Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "connector_missing",
            "This messaging-provider webhook is not active.",
        )
        .into_response());
    };
    let provider = workspace
        .provider_configs
        .iter()
        .find(|item| item.id == connector_id)
        .cloned()
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::NOT_FOUND,
                "provider_missing",
                "This messaging-provider webhook is not active.",
            )
            .into_response()
        })?;
    let timestamp = headers
        .get("x-reminder-timestamp")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::UNAUTHORIZED,
                "signature_missing",
                "Send a signed messaging-provider timestamp and signature.",
            )
            .into_response()
        })?;
    if now().abs_diff(timestamp) > 300 {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "signature_expired",
            "The messaging-provider signature is older than five minutes.",
        )
        .into_response());
    }
    let signature = headers
        .get("x-reminder-signature")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::UNAUTHORIZED,
                "signature_missing",
                "Send a signed messaging-provider timestamp and signature.",
            )
            .into_response()
        })?;
    let secret = state
        .store
        .decrypt(&provider.encrypted_webhook_secret)
        .map_err(IntoResponse::into_response)?;
    let canonical = format!(
        "{timestamp}:{}:{}:{}:{}",
        input.provider_event_id, input.provider_reference, input.outcome, input.occurred_at
    );
    if !valid_signature(&secret, canonical.as_bytes(), signature) {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "signature_invalid",
            "The messaging-provider receipt signature is invalid.",
        )
        .into_response());
    }
    record_receipt(&state, &connector_id, input).await
}

pub async fn twilio_receipt(
    State(state): State<ClinicState>,
    AxumPath(provider_id): AxumPath<String>,
    headers: HeaderMap,
    body: String,
) -> Result<StatusCode, Response> {
    let Some((_oid, workspace)) = state
        .store
        .by_provider(&provider_id)
        .map_err(IntoResponse::into_response)?
    else {
        return Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "provider_missing",
            "This Twilio receipt endpoint is not active.",
        )
        .into_response());
    };
    let provider = workspace
        .provider_configs
        .iter()
        .find(|item| item.id == provider_id && item.kind == "twilio")
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::NOT_FOUND,
                "provider_missing",
                "This Twilio receipt endpoint is not active.",
            )
            .into_response()
        })?;
    let signature = headers
        .get("x-twilio-signature")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::UNAUTHORIZED,
                "signature_missing",
                "The Twilio signature header is required.",
            )
            .into_response()
        })?;
    let params: BTreeMap<String, String> = url::form_urlencoded::parse(body.as_bytes())
        .into_owned()
        .collect();
    let public_origin = env::var("PUBLIC_ORIGIN")
        .unwrap_or_else(|_| "https://clinic-reminder-proof.sociobot.in".to_owned());
    let callback_url = format!("{public_origin}/api/v1/providers/twilio/{provider_id}/receipts");
    let auth_token = state
        .store
        .decrypt(&provider.encrypted_secret)
        .map_err(IntoResponse::into_response)?;
    if !valid_twilio_signature(&auth_token, &callback_url, &params, signature) {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "signature_invalid",
            "The Twilio receipt signature is invalid.",
        )
        .into_response());
    }
    let reference = params.get("MessageSid").cloned().ok_or_else(|| {
        ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "receipt_invalid",
            "The Twilio receipt has no message reference.",
        )
        .into_response()
    })?;
    let outcome = params
        .get("MessageStatus")
        .cloned()
        .unwrap_or_else(|| "unknown".to_owned());
    let event_id = format!("twilio:{reference}:{outcome}");
    record_receipt(
        &state,
        &provider_id,
        ReceiptInput {
            provider_reference: reference,
            provider_event_id: event_id,
            outcome,
            occurred_at: now(),
        },
    )
    .await
}

/// Accept a Resend webhook only after verifying its Svix signature.  The
/// webhook secret is intentionally separate from the Resend API key: neither
/// value is sent back in a workspace response or export.
pub async fn resend_receipt(
    State(state): State<ClinicState>,
    AxumPath(provider_id): AxumPath<String>,
    headers: HeaderMap,
    body: String,
) -> Result<StatusCode, Response> {
    let Some((_oid, workspace)) = state
        .store
        .by_provider(&provider_id)
        .map_err(IntoResponse::into_response)?
    else {
        return Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "provider_missing",
            "This Resend receipt endpoint is not active.",
        )
        .into_response());
    };
    let provider = workspace
        .provider_configs
        .iter()
        .find(|item| item.id == provider_id && item.kind == "resend")
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::NOT_FOUND,
                "provider_missing",
                "This Resend receipt endpoint is not active.",
            )
            .into_response()
        })?;
    let svix_id = required_header(&headers, "svix-id", "The Resend webhook ID is required.")?;
    let timestamp = required_header(
        &headers,
        "svix-timestamp",
        "The Resend webhook timestamp is required.",
    )?
    .parse::<u64>()
    .map_err(|_| {
        ApiError::new(
            StatusCode::UNAUTHORIZED,
            "signature_invalid",
            "The Resend webhook timestamp is invalid.",
        )
        .into_response()
    })?;
    if now().abs_diff(timestamp) > 300 {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "signature_expired",
            "The Resend webhook signature is older than five minutes.",
        )
        .into_response());
    }
    let signature = required_header(
        &headers,
        "svix-signature",
        "The Resend webhook signature is required.",
    )?;
    let secret = state
        .store
        .decrypt(&provider.encrypted_webhook_secret)
        .map_err(IntoResponse::into_response)?;
    if !valid_svix_signature(&secret, &svix_id, timestamp, &body, &signature) {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "signature_invalid",
            "The Resend webhook signature is invalid.",
        )
        .into_response());
    }
    let payload: serde_json::Value = serde_json::from_str(&body).map_err(|_| {
        ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "receipt_invalid",
            "The Resend receipt is not valid JSON.",
        )
        .into_response()
    })?;
    let reference = payload
        .pointer("/data/email_id")
        .or_else(|| payload.pointer("/data/id"))
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "receipt_invalid",
                "The Resend receipt has no email reference.",
            )
            .into_response()
        })?;
    let event = payload
        .get("type")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let outcome = match event {
        "email.delivered" => "delivered",
        "email.sent" => "sent",
        "email.bounced" | "email.complained" | "email.failed" => "failed",
        _ => {
            return Err(ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "outcome_invalid",
                "The Resend event is not a supported delivery outcome.",
            )
            .into_response())
        }
    };
    record_receipt(
        &state,
        &provider_id,
        ReceiptInput {
            provider_reference: reference.to_owned(),
            provider_event_id: format!("resend:{svix_id}"),
            outcome: outcome.to_owned(),
            occurred_at: timestamp,
        },
    )
    .await
}

fn required_header(
    headers: &HeaderMap,
    name: &'static str,
    message: &'static str,
) -> Result<String, Response> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            ApiError::new(StatusCode::UNAUTHORIZED, "signature_missing", message).into_response()
        })
}

fn valid_twilio_signature(
    secret: &str,
    callback_url: &str,
    params: &BTreeMap<String, String>,
    supplied: &str,
) -> bool {
    let mut canonical = callback_url.to_owned();
    for (key, value) in params {
        canonical.push_str(key);
        canonical.push_str(value);
    }
    let Ok(mut mac) = <Hmac<Sha1> as Mac>::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(canonical.as_bytes());
    let Ok(signature) = base64::engine::general_purpose::STANDARD.decode(supplied) else {
        return false;
    };
    mac.verify_slice(&signature).is_ok()
}

fn valid_svix_signature(
    signing_secret: &str,
    webhook_id: &str,
    timestamp: u64,
    body: &str,
    supplied: &str,
) -> bool {
    let encoded_secret = signing_secret
        .strip_prefix("whsec_")
        .unwrap_or(signing_secret);
    let Ok(key) = URL_SAFE_NO_PAD.decode(encoded_secret) else {
        return false;
    };
    let Ok(mut mac) = <Hmac<Sha256> as Mac>::new_from_slice(&key) else {
        return false;
    };
    mac.update(format!("{webhook_id}.{timestamp}.{body}").as_bytes());
    supplied.split_whitespace().any(|value| {
        let Some(signature) = value.strip_prefix("v1,") else {
            return false;
        };
        base64::engine::general_purpose::STANDARD
            .decode(signature)
            .map(|bytes| mac.clone().verify_slice(&bytes).is_ok())
            .unwrap_or(false)
    })
}

async fn record_receipt(
    state: &ClinicState,
    provider_id: &str,
    input: ReceiptInput,
) -> Result<StatusCode, Response> {
    let Some((oid, mut workspace)) = state
        .store
        .by_provider(provider_id)
        .map_err(IntoResponse::into_response)?
    else {
        return Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "provider_missing",
            "This messaging-provider receipt endpoint is not active.",
        )
        .into_response());
    };
    if !state
        .store
        .receipt_once(&input.provider_event_id, &workspace.organization_id)
        .map_err(IntoResponse::into_response)?
    {
        return Ok(StatusCode::NO_CONTENT);
    }
    let reminder_index = workspace
        .reminders
        .iter()
        .position(|reminder| {
            reminder
                .timeline
                .iter()
                .any(|event| event.provider_reference.as_deref() == Some(&input.provider_reference))
        })
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::NOT_FOUND,
                "attempt_missing",
                "No reminder attempt matches this messaging-provider reference.",
            )
            .into_response()
        })?;
    let normalized =
        match input.outcome.as_str() {
            "delivered" | "read" => input.outcome,
            "queued" | "accepted" | "sent" => "pending".to_owned(),
            "failed" | "rejected" | "undelivered" => "failed".to_owned(),
            _ => return Err(ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "outcome_invalid",
                "Use queued, accepted, sent, delivered, read, failed, rejected, or undelivered.",
            )
            .into_response()),
        };
    workspace.reminders[reminder_index]
        .timeline
        .push(ClinicEvent {
            at: input.occurred_at,
            kind: "provider-receipt".to_owned(),
            channel: None,
            outcome: normalized.clone(),
            provider_reference: Some(input.provider_reference),
            provider_code: Some(input.provider_event_id.clone()),
        });
    if normalized == "delivered" || normalized == "read" {
        workspace.reminders[reminder_index].status = normalized;
        workspace.reminders[reminder_index].exception = None;
    } else if normalized == "pending" {
        workspace.reminders[reminder_index].status = normalized;
    } else {
        let fallbacks = untried_eligible_channels(&workspace.reminders[reminder_index]);
        let mut accepted = false;
        for channel in fallbacks {
            let Some(fallback_provider) = workspace
                .provider_configs
                .iter()
                .find(|item| item.channel == channel.channel)
                .cloned()
            else {
                continue;
            };
            let snapshot = workspace.reminders[reminder_index].clone();
            let fallback_key = format!("fallback:{}:{}", input.provider_event_id, channel.channel);
            match send_provider(
                state,
                &workspace,
                &snapshot,
                &channel,
                &fallback_provider,
                &fallback_key,
            )
            .await
            {
                Ok(reference) => {
                    workspace.reminders[reminder_index].timeline.push(ClinicEvent { at: now(), kind: "provider-accepted".to_owned(), channel: Some(channel.channel), outcome: "Fallback messaging provider accepted the approved reminder; delivery receipt is pending".to_owned(), provider_reference: Some(reference), provider_code: Some(fallback_key) });
                    workspace.reminders[reminder_index].status = "provider-accepted".to_owned();
                    accepted = true;
                    break;
                }
                Err(code) => workspace.reminders[reminder_index]
                    .timeline
                    .push(ClinicEvent {
                        at: now(),
                        kind: "provider-failed".to_owned(),
                        channel: Some(channel.channel),
                        outcome: "Fallback messaging provider rejected this attempt".to_owned(),
                        provider_reference: None,
                        provider_code: Some(format!("{fallback_key}:{code}")),
                    }),
            }
        }
        if !accepted {
            open_exception(
                &mut workspace.reminders[reminder_index],
                "The messaging provider reported a terminal failure and no allowed fallback was accepted.",
            );
        }
    }
    state
        .store
        .save(&oid, &workspace)
        .map_err(IntoResponse::into_response)?;
    Ok(StatusCode::NO_CONTENT)
}

fn valid_signature(secret: &str, message: &[u8], supplied: &str) -> bool {
    let Ok(mut mac) = <Hmac<Sha256> as Mac>::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(message);
    let Ok(bytes) = URL_SAFE_NO_PAD.decode(supplied) else {
        return false;
    };
    mac.verify_slice(&bytes).is_ok()
}

pub async fn assign_exception(
    State(state): State<ClinicState>,
    AxumPath(id): AxumPath<String>,
    headers: HeaderMap,
    Json(input): Json<AssignInput>,
) -> Result<Json<WorkspaceResponse>, Response> {
    mutate_exception(state, headers, id, Some(input.owner), None).await
}
pub async fn resolve_exception(
    State(state): State<ClinicState>,
    AxumPath(id): AxumPath<String>,
    headers: HeaderMap,
    Json(input): Json<ResolveInput>,
) -> Result<Json<WorkspaceResponse>, Response> {
    mutate_exception(state, headers, id, None, Some(input.resolution)).await
}

async fn mutate_exception(
    state: ClinicState,
    headers: HeaderMap,
    id: String,
    owner: Option<String>,
    resolution: Option<String>,
) -> Result<Json<WorkspaceResponse>, Response> {
    let identity = identity(&state, &headers).await?;
    let mut workspace = state
        .store
        .get(&identity.oid)
        .map_err(IntoResponse::into_response)?
        .ok_or_else(|| missing().into_response())?;
    let exception = workspace
        .reminders
        .iter_mut()
        .find_map(|reminder| {
            reminder
                .exception
                .as_mut()
                .filter(|exception| exception.id == id)
        })
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::NOT_FOUND,
                "exception_missing",
                "Choose an exception from this clinic.",
            )
            .into_response()
        })?;
    if let Some(owner) = owner {
        validate_text(&owner, 100, "Choose an owner up to 100 characters.")
            .map_err(IntoResponse::into_response)?;
        exception.owner = Some(owner);
        exception.state = "assigned".to_owned();
    }
    if let Some(resolution) = resolution {
        if exception.owner.is_none() {
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                "owner_required",
                "Assign an owner before resolving this exception.",
            )
            .into_response());
        }
        validate_text(&resolution, 100, "Choose a short non-clinical resolution.")
            .map_err(IntoResponse::into_response)?;
        exception.resolution = Some(resolution);
        exception.state = "resolved".to_owned();
    }
    state
        .store
        .save(&identity.oid, &workspace)
        .map_err(IntoResponse::into_response)?;
    Ok(Json(workspace.into()))
}

pub async fn export_workspace(
    State(state): State<ClinicState>,
    headers: HeaderMap,
) -> Result<Response, Response> {
    let identity = identity(&state, &headers).await?;
    let workspace = state
        .store
        .get(&identity.oid)
        .map_err(IntoResponse::into_response)?
        .ok_or_else(|| missing().into_response())?;
    require_owner(&state, &identity, &workspace)?;
    state
        .store
        .record_export(&identity.oid, &workspace.organization_id)
        .map_err(IntoResponse::into_response)?;
    let notification_preferences = workspace
        .members
        .iter()
        .map(|member| NotificationPreferenceExport {
            membership_id: member.id.clone(),
            digest_enabled: false,
            exception_email: false,
        })
        .collect();
    let audit_events = workspace.audit.clone();
    let mut response = Json(ClinicExport {
        schema_version: 1,
        exported_at: now(),
        clinic: WorkspaceResponse::from(workspace),
        audit_events,
        notification_preferences,
    })
    .into_response();
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=reminder-proof-export.json"),
    );
    Ok(response)
}

pub async fn schedule_account_deletion(
    State(state): State<ClinicState>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<DeletionResponse>), Response> {
    let identity = identity(&state, &headers).await?;
    let mut workspace = state
        .store
        .get(&identity.oid)
        .map_err(IntoResponse::into_response)?
        .ok_or_else(|| missing().into_response())?;
    require_owner(&state, &identity, &workspace)?;
    let scheduled_at = now();
    let cancel_until = scheduled_at + 7 * 86_400;
    workspace.deletion = Some(DeletionSchedule {
        scheduled_at,
        cancel_until,
    });
    workspace.audit.push(AuditEvent {
        at: scheduled_at,
        actor: identity.oid.clone(),
        action: "organization.deletion_scheduled".to_owned(),
        target: workspace.organization_id.clone(),
    });
    state
        .store
        .save(&identity.oid, &workspace)
        .map_err(IntoResponse::into_response)?;
    Ok((
        StatusCode::ACCEPTED,
        Json(DeletionResponse {
            status: "scheduled",
            cancel_until: Some(cancel_until),
        }),
    ))
}

pub async fn cancel_account_deletion(
    State(state): State<ClinicState>,
    headers: HeaderMap,
) -> Result<Json<DeletionResponse>, Response> {
    let identity = identity(&state, &headers).await?;
    let mut workspace = state
        .store
        .get(&identity.oid)
        .map_err(IntoResponse::into_response)?
        .ok_or_else(|| missing().into_response())?;
    require_owner(&state, &identity, &workspace)?;
    if workspace.deletion.is_none() {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "deletion_not_scheduled",
            "This clinic does not have a scheduled deletion.",
        )
        .into_response());
    }
    workspace.deletion = None;
    workspace.audit.push(AuditEvent {
        at: now(),
        actor: identity.oid.clone(),
        action: "organization.deletion_cancelled".to_owned(),
        target: workspace.organization_id.clone(),
    });
    state
        .store
        .save(&identity.oid, &workspace)
        .map_err(IntoResponse::into_response)?;
    Ok(Json(DeletionResponse {
        status: "cancelled",
        cancel_until: None,
    }))
}

pub async fn delete_workspace(
    State(state): State<ClinicState>,
    headers: HeaderMap,
) -> Result<StatusCode, Response> {
    let identity = identity(&state, &headers).await?;
    let workspace = state
        .store
        .get(&identity.oid)
        .map_err(IntoResponse::into_response)?
        .ok_or_else(|| missing().into_response())?;
    require_owner(&state, &identity, &workspace)?;
    if headers
        .get("x-confirm-delete")
        .and_then(|value| value.to_str().ok())
        != Some(workspace.organization_id.as_str())
    {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "delete_confirmation_required",
            "Confirm this clinic organization before deleting it.",
        )
        .into_response());
    }
    state
        .store
        .delete(&identity.oid)
        .map_err(IntoResponse::into_response)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn billing_checkout(
    State(state): State<ClinicState>,
    headers: HeaderMap,
    Json(query): Json<BillingQuery>,
) -> Result<Json<CheckoutResponse>, Response> {
    let identity = identity(&state, &headers).await?;
    let workspace = state
        .store
        .get(&identity.oid)
        .map_err(IntoResponse::into_response)?
        .ok_or_else(|| missing().into_response())?;
    require_owner(&state, &identity, &workspace)?;
    if !matches!(query.tier.as_str(), "clinic" | "practice" | "network") {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "tier_invalid",
            "Choose the Clinic, Practice, or Network plan.",
        )
        .into_response());
    }
    let url = format!(
        "{}?tier={}&return_url=https%3A%2F%2Fclinic-reminder-proof.sociobot.in%2Fapp%2Fsettings%2Fbilling&organization_id={}",
        billing_product_url(&state, "checkout"),
        query.tier,
        workspace.organization_id
    );
    let probe = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|_| internal().into_response())?
        .get(&url)
        .send()
        .await
        .map_err(|_| {
            ApiError::new(
                StatusCode::BAD_GATEWAY,
                "billing_unavailable",
                "Sociobot checkout could not be reached. Try again shortly.",
            )
            .into_response()
        })?;
    if !(probe.status().is_success() || probe.status().is_redirection()) {
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "billing_product_unavailable",
            "The Clinic plan checkout is not available yet. Do not activate reminders until Sociobot checkout is available.",
        )
        .into_response());
    }
    Ok(Json(CheckoutResponse {
        checkout_url: url,
        tier: query.tier,
    }))
}

pub async fn billing_return(
    State(state): State<ClinicState>,
    headers: HeaderMap,
    Json(input): Json<BillingReturnInput>,
) -> Result<Json<WorkspaceResponse>, Response> {
    let identity = identity(&state, &headers).await?;
    let token = input.license.as_str();
    if token.len() < 16 {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "entitlement_invalid",
            "Return a valid Sociobot entitlement.",
        )
        .into_response());
    }
    let mut workspace = state
        .store
        .get(&identity.oid)
        .map_err(IntoResponse::into_response)?
        .ok_or_else(|| missing().into_response())?;
    require_owner(&state, &identity, &workspace)?;
    let response = state
        .client
        .get(billing_product_url(&state, "verify"))
        .query(&[("license", token)])
        .send()
        .await
        .map_err(|_| {
            ApiError::new(
                StatusCode::BAD_GATEWAY,
                "billing_unavailable",
                "Sociobot billing could not be reached. Try again shortly.",
            )
            .into_response()
        })?;
    let verdict: serde_json::Value = response.json().await.map_err(|_| {
        ApiError::new(
            StatusCode::BAD_GATEWAY,
            "billing_invalid",
            "Sociobot billing returned an unreadable response.",
        )
        .into_response()
    })?;
    let status = subscription_status(&verdict);
    if !matches!(status, "active" | "grace") {
        workspace.subscription.status = Some(status.to_owned());
        workspace.subscription.checked_at = Some(now());
        state
            .store
            .save(&identity.oid, &workspace)
            .map_err(IntoResponse::into_response)?;
        return Err(ApiError::new(
            StatusCode::PAYMENT_REQUIRED,
            "subscription_inactive",
            "This subscription is not active. Check billing in Sociobot.",
        )
        .into_response());
    }
    let tier = input.tier.unwrap_or_else(|| "clinic".to_owned());
    if !matches!(tier.as_str(), "clinic" | "practice" | "network") {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "tier_invalid",
            "Choose the Clinic, Practice, or Network plan.",
        )
        .into_response());
    }
    workspace.subscription = Subscription {
        tier: Some(tier),
        status: Some(status.to_owned()),
        checked_at: Some(now()),
        encrypted_entitlement: Some(
            state
                .store
                .encrypt(token)
                .map_err(IntoResponse::into_response)?,
        ),
    };
    state
        .store
        .save(&identity.oid, &workspace)
        .map_err(IntoResponse::into_response)?;
    Ok(Json(workspace.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State as AxumState;
    use axum::{routing::get, routing::post, Router};
    use http_body_util::BodyExt;
    use tokio::net::TcpListener;
    use tower::ServiceExt;

    #[test]
    fn production_storage_guard_requires_both_azure_file_mounts() {
        let mount_info = "31 22 0:29 / / rw,relatime - overlay overlay rw\n42 31 0:61 / /durable rw,relatime - cifs //storage/data rw\n43 31 0:62 / /backups rw,relatime - cifs //storage/backups rw\n";
        assert!(missing_required_mounts(
            mount_info,
            &[Path::new("/durable"), Path::new("/backups")]
        )
        .is_empty());

        assert_eq!(
            missing_required_mounts(
                "31 22 0:29 / / rw,relatime - overlay overlay rw\n42 31 0:61 / /durable rw,relatime - cifs //storage/data rw\n",
                &[Path::new("/durable"), Path::new("/backups")]
            ),
            vec!["/backups"]
        );
    }

    async fn fixture_gateway() -> (String, tokio::task::JoinHandle<()>) {
        let router = Router::new()
            .route(
                "/send/sms",
                post(|| async { (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error":"rejected"}))) }),
            )
            .route(
                "/send/email",
                post(|| async { Json(serde_json::json!({"id":"email-fixture-42"})) }),
            )
            .route(
                "/products/clinic-reminder-proof/checkout",
                get(|| async { (StatusCode::SEE_OTHER, [(header::LOCATION, "https://clinic-reminder-proof.sociobot.in/app?license=fixture-license-token-1234")]) }),
            )
            .route(
                "/products/clinic-reminder-proof/verify",
                get(|| async { Json(serde_json::json!({"valid":true,"reason":"ok"})) }),
            );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        (format!("http://{address}"), task)
    }

    #[derive(Clone, Default)]
    struct ProviderCapture(Arc<Mutex<Vec<(HeaderMap, serde_json::Value)>>>);

    async fn capture_provider_request(
        AxumState(capture): AxumState<ProviderCapture>,
        headers: HeaderMap,
        Json(body): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        capture.0.lock().unwrap().push((headers, body));
        Json(serde_json::json!({"id":"fixture-message-77"}))
    }

    async fn capturing_provider_gateway() -> (String, ProviderCapture, tokio::task::JoinHandle<()>)
    {
        let capture = ProviderCapture::default();
        let router = Router::new()
            .route("/send/sms", post(capture_provider_request))
            .route("/send/whatsapp", post(capture_provider_request))
            .route("/send/email", post(capture_provider_request))
            .with_state(capture.clone());
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        (format!("http://{address}"), capture, task)
    }

    fn sign_hmac_sha256(secret: &str, value: &str) -> String {
        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(value.as_bytes());
        URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
    }

    fn twilio_signature(
        secret: &str,
        provider_id: &str,
        params: &BTreeMap<String, String>,
    ) -> String {
        let mut canonical = format!(
            "https://clinic-reminder-proof.sociobot.in/api/v1/providers/twilio/{provider_id}/receipts"
        );
        for (key, value) in params {
            canonical.push_str(key);
            canonical.push_str(value);
        }
        let mut mac = <Hmac<Sha1> as Mac>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(canonical.as_bytes());
        base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes())
    }

    fn resend_signature(secret: &str, event_id: &str, timestamp: u64, body: &str) -> String {
        let encoded_secret = secret.strip_prefix("whsec_").unwrap_or(secret);
        let key = URL_SAFE_NO_PAD.decode(encoded_secret).unwrap();
        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(&key).unwrap();
        mac.update(format!("{event_id}.{timestamp}.{body}").as_bytes());
        format!(
            "v1,{}",
            base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes())
        )
    }

    fn provider_with_reference(
        state: &ClinicState,
        id: &str,
        channel: &str,
        kind: &str,
        credential: &str,
        webhook_secret: &str,
        reference: &str,
    ) -> ClinicWorkspace {
        let mut reminder = appointment(&[(channel, "allowed")]);
        reminder.status = "provider-accepted".to_owned();
        reminder.timeline.push(ClinicEvent {
            at: now(),
            kind: "provider-accepted".to_owned(),
            channel: Some(channel.to_owned()),
            outcome: "Messaging provider accepted the approved reminder".to_owned(),
            provider_reference: Some(reference.to_owned()),
            provider_code: Some("fixture-dispatch".to_owned()),
        });
        ClinicWorkspace {
            organization_id: format!("{id}-org"),
            clinic_name: "Fixture Dental".to_owned(),
            location_name: "Main".to_owned(),
            timezone: "UTC".to_owned(),
            provider_configs: vec![StoredProvider {
                id: id.to_owned(),
                channel: channel.to_owned(),
                kind: kind.to_owned(),
                account_id: "fixture-account".to_owned(),
                encrypted_secret: state.store.encrypt(credential).unwrap(),
                from: "+15550000001".to_owned(),
                approved_template_id: "HX-approved-template".to_owned(),
                encrypted_webhook_secret: state.store.encrypt(webhook_secret).unwrap(),
            }],
            reminders: vec![reminder],
            ..Default::default()
        }
    }

    fn appointment(consents: &[(&str, &str)]) -> ClinicReminder {
        ClinicReminder {
            id: "r1".to_owned(),
            source_id: "source-1".to_owned(),
            patient_alias: "Patient A".to_owned(),
            first_name: "A".to_owned(),
            appointment_time: "2026-09-01 09:00".to_owned(),
            status: "scheduled".to_owned(),
            channels: consents
                .iter()
                .map(|(channel, consent)| ReminderChannel {
                    channel: (*channel).to_owned(),
                    destination: if *channel == "email" {
                        "patient@example.test".to_owned()
                    } else {
                        "+447700900001".to_owned()
                    },
                    consent: (*consent).to_owned(),
                    consent_source: "EMR".to_owned(),
                    consent_captured_at: "2026-08-20".to_owned(),
                })
                .collect(),
            timeline: Vec::new(),
            exception: None,
        }
    }

    #[test]
    fn consent_guard_and_fallback_order_are_deterministic() {
        let reminder = appointment(&[
            ("sms", "blocked"),
            ("email", "allowed"),
            ("whatsapp", "allowed"),
        ]);
        let channels = eligible_channels(&reminder);
        assert_eq!(
            channels
                .iter()
                .map(|item| item.channel.as_str())
                .collect::<Vec<_>>(),
            vec!["email", "whatsapp"]
        );
    }

    #[test]
    fn m2_billing_verdicts_cover_subscription_states() {
        assert_eq!(
            subscription_status(&serde_json::json!({"valid":true})),
            "active"
        );
        assert_eq!(
            subscription_status(&serde_json::json!({"valid":true,"status":"grace"})),
            "grace"
        );
        assert_eq!(
            subscription_status(&serde_json::json!({"valid":false,"reason":"invalid"})),
            "past_due"
        );
        assert_eq!(
            subscription_status(&serde_json::json!({"valid":false,"reason":"expired"})),
            "cancelled"
        );
        assert_eq!(
            subscription_status(&serde_json::json!({"valid":false,"reason":"revoked"})),
            "revoked"
        );
    }

    #[test]
    fn managed_claim_terminal_failure_selects_the_next_untried_consented_channel() {
        let mut reminder = appointment(&[
            ("sms", "allowed"),
            ("email", "allowed"),
            ("whatsapp", "blocked"),
        ]);
        reminder.timeline.push(ClinicEvent {
            at: now(),
            kind: "provider-accepted".to_owned(),
            channel: Some("sms".to_owned()),
            outcome: "accepted".to_owned(),
            provider_reference: Some("SM1".to_owned()),
            provider_code: None,
        });
        let fallback = untried_eligible_channels(&reminder);
        assert_eq!(fallback.len(), 1);
        assert_eq!(fallback[0].channel, "email");
    }

    #[tokio::test]
    async fn managed_claim_signed_calendar_intake_is_authenticated_and_idempotent() {
        let path = std::env::temp_dir().join(format!("reminder-proof-{}", Uuid::new_v4()));
        let state = ClinicState::for_tests(path.clone()).unwrap();
        let connector_id = "fixture-calendar-connector";
        let secret = "fixture-calendar-signing-secret";
        state
            .store
            .save(
                "fixture-owner",
                &ClinicWorkspace {
                    organization_id: "fixture-calendar-org".to_owned(),
                    clinic_name: "Fixture Dental".to_owned(),
                    connector: Some(ConnectorPublic {
                        id: connector_id.to_owned(),
                        kind: "signed-calendar-webhook".to_owned(),
                        connected_at: now(),
                        last_received_at: None,
                    }),
                    encrypted_connector_secret: Some(state.store.encrypt(secret).unwrap()),
                    ..Default::default()
                },
            )
            .unwrap();
        let app = crate::app_with_clinic_state("fixture", "../../dist", state.clone());
        let timestamp = now();
        let body = serde_json::json!({
            "connector_id": connector_id,
            "appointments": [{
                "source_id": "source-idempotent-1",
                "patient_alias": "Patient Z",
                "first_name": "Z",
                "appointment_time": "2026-09-04T10:30:00Z",
                "channels": [{
                    "channel": "email",
                    "destination": "patient-z@example.test",
                    "consent": "allowed",
                    "consent_source": "signed source",
                    "consent_captured_at": "2026-08-29T09:00:00Z"
                }]
            }]
        });
        let signature = sign_hmac_sha256(secret, &format!("{timestamp}:{connector_id}:1"));

        for _ in 0..2 {
            let response = app
                .clone()
                .oneshot(
                    axum::http::Request::builder()
                        .method("POST")
                        .uri("/api/v1/connectors/intake")
                        .header(header::CONTENT_TYPE, "application/json")
                        .header("x-reminder-timestamp", timestamp.to_string())
                        .header("x-reminder-signature", &signature)
                        .body(axum::body::Body::from(body.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
        let stored = state.store.get("fixture-owner").unwrap().unwrap();
        assert_eq!(stored.reminders.len(), 1);
        assert_eq!(stored.reminders[0].source_id, "source-idempotent-1");

        let mut altered = body;
        altered["appointments"][0]["patient_alias"] = serde_json::json!("Altered patient");
        let rejected = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/connectors/intake")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("x-reminder-timestamp", timestamp.to_string())
                    .header("x-reminder-signature", "invalid-signature")
                    .body(axum::body::Body::from(altered.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(rejected.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            state.store.get("fixture-owner").unwrap().unwrap().reminders[0].patient_alias,
            "Patient Z"
        );
        let _ = fs::remove_dir_all(path);
    }

    #[tokio::test]
    async fn managed_claim_approved_whatsapp_uses_template_and_records_receipt() {
        let (gateway, capture, gateway_task) = capturing_provider_gateway().await;
        let path = std::env::temp_dir().join(format!("reminder-proof-{}", Uuid::new_v4()));
        let state =
            ClinicState::for_tests_with_fixtures(path.clone(), gateway.clone(), gateway).unwrap();
        let provider_id = "whatsapp-provider";
        let credential = "twilio-whatsapp-token";
        let mut workspace = provider_with_reference(
            &state,
            provider_id,
            "whatsapp",
            "twilio",
            credential,
            "unused-webhook-secret",
            "unused-reference",
        );
        workspace.reminders[0].status = "scheduled".to_owned();
        workspace.reminders[0].timeline.clear();
        workspace.subscription = Subscription {
            tier: Some("clinic".to_owned()),
            status: Some("active".to_owned()),
            checked_at: Some(now()),
            encrypted_entitlement: Some(state.store.encrypt("fixture-license-token").unwrap()),
        };
        state.store.save("fixture-owner", &workspace).unwrap();
        let app = crate::app_with_clinic_state("fixture", "../../dist", state.clone());
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/clinic/reminders/dispatch")
                    .header(header::AUTHORIZATION, "Bearer test:fixture-owner")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("idempotency-key", "whatsapp-template-77")
                    .body(axum::body::Body::from(r#"{"reminder_id":"r1"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        {
            let requests = capture.0.lock().unwrap();
            assert_eq!(requests.len(), 1);
            assert_eq!(
                requests[0].0.get(header::AUTHORIZATION).unwrap(),
                "Bearer twilio-whatsapp-token"
            );
            assert_eq!(
                requests[0].1,
                serde_json::json!({
                    "from": "+15550000001",
                    "to": "+447700900001",
                    "template": "HX-approved-template",
                    "first_name": "A",
                    "clinic": "Fixture Dental",
                    "appointment_time": "2026-09-01 09:00"
                })
            );
            assert!(requests[0].1.get("body").is_none());
        }

        let params = BTreeMap::from([
            ("MessageSid".to_owned(), "fixture-message-77".to_owned()),
            ("MessageStatus".to_owned(), "delivered".to_owned()),
        ]);
        let body = "MessageSid=fixture-message-77&MessageStatus=delivered".to_owned();
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-twilio-signature",
            HeaderValue::from_str(&twilio_signature(credential, provider_id, &params)).unwrap(),
        );
        assert_eq!(
            twilio_receipt(
                State(state.clone()),
                AxumPath(provider_id.to_owned()),
                headers,
                body,
            )
            .await
            .unwrap(),
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            state.store.get("fixture-owner").unwrap().unwrap().reminders[0].status,
            "delivered"
        );
        gateway_task.abort();
        let _ = fs::remove_dir_all(path);
    }

    #[tokio::test]
    async fn managed_claim_twilio_receipt_verification_is_replay_safe() {
        let path = std::env::temp_dir().join(format!("reminder-proof-{}", Uuid::new_v4()));
        let state = ClinicState::for_tests(path.clone()).unwrap();
        let provider_id = "twilio-receipt-provider";
        let secret = "twilio-auth-token";
        state
            .store
            .save(
                "fixture-owner",
                &provider_with_reference(
                    &state,
                    provider_id,
                    "sms",
                    "twilio",
                    secret,
                    "unused-webhook",
                    "SM-fixture-123",
                ),
            )
            .unwrap();
        let params = BTreeMap::from([
            ("MessageSid".to_owned(), "SM-fixture-123".to_owned()),
            ("MessageStatus".to_owned(), "delivered".to_owned()),
        ]);
        let body = "MessageSid=SM-fixture-123&MessageStatus=delivered".to_owned();
        let signature = twilio_signature(secret, provider_id, &params);
        for _ in 0..2 {
            let mut headers = HeaderMap::new();
            headers.insert(
                "x-twilio-signature",
                HeaderValue::from_str(&signature).unwrap(),
            );
            assert_eq!(
                twilio_receipt(
                    State(state.clone()),
                    AxumPath(provider_id.to_owned()),
                    headers,
                    body.clone(),
                )
                .await
                .unwrap(),
                StatusCode::NO_CONTENT
            );
        }
        let before_invalid = state.store.get("fixture-owner").unwrap().unwrap();
        assert_eq!(
            before_invalid.reminders[0]
                .timeline
                .iter()
                .filter(|event| event.kind == "provider-receipt")
                .count(),
            1
        );
        let mut bad_headers = HeaderMap::new();
        bad_headers.insert(
            "x-twilio-signature",
            HeaderValue::from_static("altered-signature"),
        );
        let rejected = twilio_receipt(
            State(state.clone()),
            AxumPath(provider_id.to_owned()),
            bad_headers,
            body,
        )
        .await
        .unwrap_err();
        assert_eq!(rejected.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            state.store.get("fixture-owner").unwrap().unwrap().reminders[0]
                .timeline
                .iter()
                .filter(|event| event.kind == "provider-receipt")
                .count(),
            1
        );
        let _ = fs::remove_dir_all(path);
    }

    #[tokio::test]
    async fn managed_claim_resend_receipt_verification_is_replay_safe() {
        let path = std::env::temp_dir().join(format!("reminder-proof-{}", Uuid::new_v4()));
        let state = ClinicState::for_tests(path.clone()).unwrap();
        let provider_id = "resend-receipt-provider";
        let key = [9_u8; 32];
        let secret = format!("whsec_{}", URL_SAFE_NO_PAD.encode(key));
        state
            .store
            .save(
                "fixture-owner",
                &provider_with_reference(
                    &state,
                    provider_id,
                    "email",
                    "resend",
                    "resend-api-token",
                    &secret,
                    "email-fixture-123",
                ),
            )
            .unwrap();
        let event_id = "msg_fixture_123";
        let timestamp = now();
        let body = r#"{"type":"email.delivered","data":{"email_id":"email-fixture-123"}}"#;
        let signature = resend_signature(&secret, event_id, timestamp, body);
        for _ in 0..2 {
            let mut headers = HeaderMap::new();
            headers.insert("svix-id", HeaderValue::from_static("msg_fixture_123"));
            headers.insert(
                "svix-timestamp",
                HeaderValue::from_str(&timestamp.to_string()).unwrap(),
            );
            headers.insert("svix-signature", HeaderValue::from_str(&signature).unwrap());
            assert_eq!(
                resend_receipt(
                    State(state.clone()),
                    AxumPath(provider_id.to_owned()),
                    headers,
                    body.to_owned(),
                )
                .await
                .unwrap(),
                StatusCode::NO_CONTENT
            );
        }
        assert_eq!(
            state.store.get("fixture-owner").unwrap().unwrap().reminders[0]
                .timeline
                .iter()
                .filter(|event| event.kind == "provider-receipt")
                .count(),
            1
        );
        let mut bad_headers = HeaderMap::new();
        bad_headers.insert("svix-id", HeaderValue::from_static("msg_fixture_123"));
        bad_headers.insert(
            "svix-timestamp",
            HeaderValue::from_str(&timestamp.to_string()).unwrap(),
        );
        bad_headers.insert("svix-signature", HeaderValue::from_static("v1,altered"));
        let rejected = resend_receipt(
            State(state.clone()),
            AxumPath(provider_id.to_owned()),
            bad_headers,
            body.to_owned(),
        )
        .await
        .unwrap_err();
        assert_eq!(rejected.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            state.store.get("fixture-owner").unwrap().unwrap().reminders[0]
                .timeline
                .iter()
                .filter(|event| event.kind == "provider-receipt")
                .count(),
            1
        );
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn managed_claim_twilio_signatures_are_checked_without_string_comparison() {
        let url = "https://clinic-reminder-proof.sociobot.in/api/v1/providers/twilio/p1/receipts";
        let params = BTreeMap::from([
            ("MessageSid".to_owned(), "SM123".to_owned()),
            ("MessageStatus".to_owned(), "delivered".to_owned()),
        ]);
        let mut canonical = url.to_owned();
        for (key, value) in &params {
            canonical.push_str(key);
            canonical.push_str(value);
        }
        let mut mac = <Hmac<Sha1> as Mac>::new_from_slice(b"auth-token").unwrap();
        mac.update(canonical.as_bytes());
        let signature =
            base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
        assert!(valid_twilio_signature(
            "auth-token",
            url,
            &params,
            &signature
        ));
        assert!(!valid_twilio_signature(
            "wrong-token",
            url,
            &params,
            &signature
        ));
    }

    #[test]
    fn managed_claim_resend_receipts_require_a_valid_svix_signature() {
        let signing_key = [7_u8; 32];
        let secret = format!("whsec_{}", URL_SAFE_NO_PAD.encode(signing_key));
        let body = r#"{"type":"email.delivered","data":{"email_id":"email-123"}}"#;
        let canonical = format!("msg_123.1760000000.{body}");
        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(&signing_key).unwrap();
        mac.update(canonical.as_bytes());
        let signature =
            base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
        assert!(valid_svix_signature(
            &secret,
            "msg_123",
            1_760_000_000,
            body,
            &format!("v1,{signature}"),
        ));
        assert!(!valid_svix_signature(
            &secret,
            "msg_123",
            1_760_000_000,
            body,
            "v1,not-a-signature",
        ));
    }

    #[test]
    fn managed_store_is_durable_and_tenant_scoped() {
        let path = std::env::temp_dir().join(format!("reminder-proof-{}", Uuid::new_v4()));
        let first = ClinicState::for_tests(path.clone()).unwrap();
        let a = ClinicWorkspace {
            organization_id: Uuid::new_v4().to_string(),
            clinic_name: "Clinic A".to_owned(),
            ..Default::default()
        };
        first.store.save("oid-a", &a).unwrap();
        drop(first);
        let second = ClinicState::for_tests(path.clone()).unwrap();
        assert_eq!(
            second.store.get("oid-a").unwrap().unwrap().clinic_name,
            "Clinic A"
        );
        assert!(second.store.get("oid-b").unwrap().is_none());
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn managed_backup_pair_restores_after_database_loss() {
        let path = std::env::temp_dir().join(format!("reminder-proof-{}", Uuid::new_v4()));
        let first = ClinicState::for_tests(path.clone()).unwrap();
        let workspace = ClinicWorkspace {
            organization_id: "restore-fixture-org".to_owned(),
            clinic_name: "Restored Clinic".to_owned(),
            ..Default::default()
        };
        first.store.save("restore-owner", &workspace).unwrap();
        let day = now() / 86_400;
        assert!(path
            .join(format!("backups/clinic-data.day-{day}.sqlite3"))
            .is_file());
        assert!(path
            .join(format!("backups/clinic-data.day-{day}.key"))
            .is_file());
        drop(first);

        assert!(path.join("durable/clinic-data.latest.sqlite3").is_file());
        assert!(path.join("durable/clinic-data.latest.key").is_file());
        fs::remove_file(path.join("clinic-data.sqlite3")).unwrap();
        fs::remove_file(path.join("clinic-data.key")).unwrap();
        let restored = ClinicState::for_tests(path.clone()).unwrap();
        assert_eq!(
            restored
                .store
                .get("restore-owner")
                .unwrap()
                .unwrap()
                .clinic_name,
            "Restored Clinic"
        );
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn managed_storage_recovery_claim() {
        let path = std::env::temp_dir().join(format!("reminder-proof-{}", Uuid::new_v4()));
        let first = ClinicState::for_tests(path.clone()).unwrap();
        let workspace = ClinicWorkspace {
            organization_id: "storage-recovery-org".to_owned(),
            clinic_name: "Storage Recovery Clinic".to_owned(),
            ..Default::default()
        };
        first
            .store
            .save("storage-recovery-owner", &workspace)
            .unwrap();
        let today = now() / 86_400;
        assert!(path.join("durable/clinic-data.latest.sqlite3").is_file());
        assert!(path.join("durable/clinic-data.latest.key").is_file());
        assert!(path
            .join(format!("backups/clinic-data.day-{today}.sqlite3"))
            .is_file());
        assert!(path
            .join(format!("backups/clinic-data.day-{today}.key"))
            .is_file());

        for suffix in ["sqlite3", "key"] {
            fs::write(
                path.join(format!("backups/clinic-data.day-{}.{}", today - 30, suffix)),
                "keep",
            )
            .unwrap();
            fs::write(
                path.join(format!("backups/clinic-data.day-{}.{}", today - 31, suffix)),
                "prune",
            )
            .unwrap();
        }
        first.store.prune_daily_backups(today).unwrap();
        assert!(path
            .join(format!("backups/clinic-data.day-{}.sqlite3", today - 30))
            .exists());
        assert!(!path
            .join(format!("backups/clinic-data.day-{}.sqlite3", today - 31))
            .exists());
        drop(first);

        fs::remove_file(path.join("clinic-data.sqlite3")).unwrap();
        fs::remove_file(path.join("clinic-data.key")).unwrap();
        let restored = ClinicState::for_tests(path.clone()).unwrap();
        assert_eq!(
            restored
                .store
                .get("storage-recovery-owner")
                .unwrap()
                .unwrap()
                .clinic_name,
            "Storage Recovery Clinic"
        );
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn managed_claim_provider_secrets_are_encrypted_at_rest() {
        let path = std::env::temp_dir().join(format!("reminder-proof-{}", Uuid::new_v4()));
        let state = ClinicState::for_tests(path.clone()).unwrap();
        let encrypted = state.store.encrypt("provider-secret").unwrap();
        assert!(!encrypted.contains("provider-secret"));
        assert_eq!(state.store.decrypt(&encrypted).unwrap(), "provider-secret");
        let _ = fs::remove_dir_all(path);
    }

    #[tokio::test]
    async fn managed_claim_secrets_and_destinations_are_encrypted_and_adapter_scoped() {
        let (gateway, capture, gateway_task) = capturing_provider_gateway().await;
        let path = std::env::temp_dir().join(format!("reminder-proof-{}", Uuid::new_v4()));
        let state =
            ClinicState::for_tests_with_fixtures(path.clone(), gateway.clone(), gateway).unwrap();
        let credential = "fixture-provider-credential-private";
        let webhook_secret = "fixture-provider-webhook-private";
        let destination = "+447700912345";
        let mut reminder = appointment(&[("sms", "allowed")]);
        reminder.channels[0].destination = destination.to_owned();
        let workspace = ClinicWorkspace {
            organization_id: "secret-scope-org".to_owned(),
            clinic_name: "Fixture Dental".to_owned(),
            location_name: "Main".to_owned(),
            timezone: "UTC".to_owned(),
            provider_configs: vec![StoredProvider {
                id: "secret-provider".to_owned(),
                channel: "sms".to_owned(),
                kind: "twilio".to_owned(),
                account_id: "fixture-account".to_owned(),
                encrypted_secret: state.store.encrypt(credential).unwrap(),
                from: "+15550000001".to_owned(),
                approved_template_id: "approved-sms-template".to_owned(),
                encrypted_webhook_secret: state.store.encrypt(webhook_secret).unwrap(),
            }],
            reminders: vec![reminder],
            subscription: Subscription {
                tier: Some("clinic".to_owned()),
                status: Some("active".to_owned()),
                checked_at: Some(now()),
                encrypted_entitlement: Some(state.store.encrypt("fixture-license-token").unwrap()),
            },
            ..Default::default()
        };
        state.store.save("fixture-owner", &workspace).unwrap();

        for stored_path in [
            path.join("clinic-data.sqlite3"),
            path.join("durable/clinic-data.latest.sqlite3"),
        ] {
            let bytes = fs::read(stored_path).unwrap();
            assert!(!bytes
                .windows(credential.len())
                .any(|part| part == credential.as_bytes()));
            assert!(!bytes
                .windows(webhook_secret.len())
                .any(|part| part == webhook_secret.as_bytes()));
            assert!(!bytes
                .windows(destination.len())
                .any(|part| part == destination.as_bytes()));
        }
        let exported = serde_json::to_string(&WorkspaceResponse::from(workspace)).unwrap();
        assert!(!exported.contains(credential));
        assert!(!exported.contains(webhook_secret));
        assert!(!exported.contains(destination));
        assert!(exported.contains("[encrypted contact]"));

        let app = crate::app_with_clinic_state("fixture", "../../dist", state);
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/clinic/reminders/dispatch")
                    .header(header::AUTHORIZATION, "Bearer test:fixture-owner")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("idempotency-key", "secret-scope-dispatch")
                    .body(axum::body::Body::from(r#"{"reminder_id":"r1"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let requests = capture.0.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].0.get(header::AUTHORIZATION).unwrap(),
            "Bearer fixture-provider-credential-private"
        );
        assert_eq!(requests[0].1["to"], destination);
        drop(requests);
        gateway_task.abort();
        let _ = fs::remove_dir_all(path);
    }

    #[tokio::test]
    async fn managed_claim_clinical_fields_are_rejected_at_every_json_write() {
        assert!(serde_json::from_value::<OnboardInput>(serde_json::json!({
            "clinic_name":"Clinic", "location_name":"Main", "timezone":"UTC", "diagnosis":"x"
        }))
        .is_err());
        assert!(serde_json::from_value::<ConnectorInput>(serde_json::json!({
            "kind":"signed-calendar-webhook", "webhook_secret":"0123456789abcdef", "clinical_note":"x"
        })).is_err());
        assert!(serde_json::from_value::<ProviderInput>(serde_json::json!({
            "channel":"sms", "kind":"twilio", "account_id":"a", "secret":"s", "from":"+15550000001",
            "approved_template_id":"t", "webhook_secret":"w", "treatment":"x"
        }))
        .is_err());
        assert!(serde_json::from_value::<DispatchInput>(serde_json::json!({
            "reminder_id":"r1", "diagnosis":"x"
        }))
        .is_err());
        assert!(serde_json::from_value::<ReceiptInput>(serde_json::json!({
            "provider_reference":"p", "provider_event_id":"e", "outcome":"sent", "occurred_at":1,
            "treatment_detail":"x"
        }))
        .is_err());
        assert!(serde_json::from_value::<AssignInput>(serde_json::json!({
            "owner":"Sam", "clinical_note":"x"
        }))
        .is_err());
        assert!(serde_json::from_value::<ResolveInput>(serde_json::json!({
            "resolution":"Called patient", "diagnosis":"x"
        }))
        .is_err());
        assert!(serde_json::from_value::<BillingQuery>(serde_json::json!({
            "tier":"clinic", "treatment":"x"
        }))
        .is_err());
        assert!(
            serde_json::from_value::<BillingReturnInput>(serde_json::json!({
                "license":"fixture-license-token", "diagnosis":"x"
            }))
            .is_err()
        );

        let path = std::env::temp_dir().join(format!("reminder-proof-{}", Uuid::new_v4()));
        let state = ClinicState::for_tests(path.clone()).unwrap();
        let connector_id = "minimal-intake-connector";
        let secret = "minimal-intake-signing-secret";
        state
            .store
            .save(
                "fixture-owner",
                &ClinicWorkspace {
                    organization_id: "minimal-intake-org".to_owned(),
                    clinic_name: "Fixture Dental".to_owned(),
                    connector: Some(ConnectorPublic {
                        id: connector_id.to_owned(),
                        kind: "signed-calendar-webhook".to_owned(),
                        connected_at: now(),
                        last_received_at: None,
                    }),
                    encrypted_connector_secret: Some(state.store.encrypt(secret).unwrap()),
                    ..Default::default()
                },
            )
            .unwrap();
        let timestamp = now();
        let signature = sign_hmac_sha256(secret, &format!("{timestamp}:{connector_id}:1"));
        let rejected = crate::app_with_clinic_state("fixture", "../../dist", state.clone())
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/connectors/intake")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("x-reminder-timestamp", timestamp.to_string())
                    .header("x-reminder-signature", signature)
                    .body(axum::body::Body::from(serde_json::json!({
                        "connector_id": connector_id,
                        "appointments": [{
                            "source_id":"source-clinical-1", "patient_alias":"Patient A", "first_name":"A",
                            "appointment_time":"2026-09-04T10:30:00Z", "clinical_note":"root canal",
                            "channels":[{"channel":"email", "destination":"a@example.test", "consent":"allowed",
                                "consent_source":"signed source", "consent_captured_at":"2026-08-29T09:00:00Z"}]
                        }]
                    }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(rejected.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let stored = state.store.get("fixture-owner").unwrap().unwrap();
        assert!(stored.reminders.is_empty());
        let export = serde_json::to_string(&WorkspaceResponse::from(stored)).unwrap();
        assert!(!export.contains("clinical_note"));
        assert!(!export.contains("root canal"));
        let _ = fs::remove_dir_all(path);
    }

    #[cfg(unix)]
    #[test]
    fn managed_storage_files_are_owner_only() {
        use std::os::unix::fs::PermissionsExt;

        let path = std::env::temp_dir().join(format!("reminder-proof-{}", Uuid::new_v4()));
        let state = ClinicState::for_tests(path.clone()).unwrap();
        drop(state);
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(path.join("clinic-data.key"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(path.join("clinic-data.sqlite3"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(path.join("backups"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn connector_secret_is_not_an_audit_event_or_exported_workspace_field() {
        let path = std::env::temp_dir().join(format!("reminder-proof-{}", Uuid::new_v4()));
        let state = ClinicState::for_tests(path.clone()).unwrap();
        let secret = state.store.encrypt("calendar-signing-secret").unwrap();
        let workspace = ClinicWorkspace {
            organization_id: Uuid::new_v4().to_string(),
            encrypted_connector_secret: Some(secret),
            audit: vec![AuditEvent {
                at: now(),
                actor: "owner".to_owned(),
                action: "connector.configured".to_owned(),
                target: "connector".to_owned(),
            }],
            ..Default::default()
        };
        assert_eq!(
            connector_secret(&state, &workspace).unwrap(),
            "calendar-signing-secret"
        );
        let exported = serde_json::to_string(&WorkspaceResponse::from(workspace)).unwrap();
        assert!(!exported.contains("calendar-signing-secret"));
        assert!(!exported.contains("connector.configured"));
        let _ = fs::remove_dir_all(path);
    }

    #[tokio::test]
    async fn managed_claim_no_marketing_campaigns() {
        assert!(serde_json::from_str::<DispatchInput>(
            r#"{"reminder_id":"r1","marketing_message":"Book another visit today"}"#
        )
        .is_err());

        let workspace = ClinicWorkspace {
            clinic_name: "Fixture Dental".to_owned(),
            ..Default::default()
        };
        let reminder = appointment(&[("sms", "allowed")]);
        let content = appointment_reminder_message(&reminder, &workspace);
        assert_eq!(
            content,
            "Hi A, reminder: appointment at Fixture Dental on 2026-09-01 09:00. Reply STOP to opt out."
        );
        assert!(!content.contains("Book another visit"));

        let path = std::env::temp_dir().join(format!("reminder-proof-{}", Uuid::new_v4()));
        let state = ClinicState::for_tests(path.clone()).unwrap();
        let application = crate::app_with_clinic_state("fixture", "../../dist", state);
        let rejected = application
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/clinic/reminders/dispatch")
                    .header(header::AUTHORIZATION, "Bearer test:fixture-owner")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("idempotency-key", "fixture-campaign-guard")
                    .body(axum::body::Body::from(
                        r#"{"reminder_id":"r1","marketing_message":"Book another visit today"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(rejected.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = rejected.into_body().collect().await.unwrap().to_bytes();
        assert!(std::str::from_utf8(&body).unwrap().contains("json_invalid"));
        let _ = fs::remove_dir_all(path);
    }

    #[tokio::test]
    async fn managed_claim_export_and_delete_are_owner_scoped() {
        let path = std::env::temp_dir().join(format!("reminder-proof-{}", Uuid::new_v4()));
        let state = ClinicState::for_tests(path.clone()).unwrap();
        let workspace = ClinicWorkspace {
            organization_id: "fixture-org".to_owned(),
            clinic_name: "Fixture Dental".to_owned(),
            location_name: "Main".to_owned(),
            timezone: "UTC".to_owned(),
            reminders: vec![appointment(&[("sms", "allowed")])],
            ..Default::default()
        };
        state.store.save("fixture-owner", &workspace).unwrap();
        let application = crate::app_with_clinic_state("fixture", "../../dist", state.clone());

        let exported = application
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/clinic/export")
                    .header(header::AUTHORIZATION, "Bearer test:fixture-owner")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(exported.status(), StatusCode::OK);
        assert_eq!(
            exported.headers().get(header::CONTENT_DISPOSITION).unwrap(),
            "attachment; filename=reminder-proof-export.json"
        );
        let export_body = exported.into_body().collect().await.unwrap().to_bytes();
        let export_text = std::str::from_utf8(&export_body).unwrap();
        assert!(export_text.contains("Fixture Dental"));
        assert!(!export_text.contains("encrypted_"));

        let other_clinic = application
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/clinic/export")
                    .header(header::AUTHORIZATION, "Bearer test:another-owner")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(other_clinic.status(), StatusCode::NOT_FOUND);

        let unconfirmed = application
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("DELETE")
                    .uri("/api/v1/clinic")
                    .header(header::AUTHORIZATION, "Bearer test:fixture-owner")
                    .header("x-confirm-delete", "wrong-org")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unconfirmed.status(), StatusCode::CONFLICT);
        assert!(state.store.get("fixture-owner").unwrap().is_some());

        let deleted = application
            .oneshot(
                axum::http::Request::builder()
                    .method("DELETE")
                    .uri("/api/v1/clinic")
                    .header(header::AUTHORIZATION, "Bearer test:fixture-owner")
                    .header("x-confirm-delete", "fixture-org")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(deleted.status(), StatusCode::NO_CONTENT);
        assert!(state.store.get("fixture-owner").unwrap().is_none());
        let _ = fs::remove_dir_all(path);
    }

    #[tokio::test]
    async fn managed_claim_provider_fallback_and_receipt_is_observable() {
        let (fixture, fixture_task) = fixture_gateway().await;
        let path = std::env::temp_dir().join(format!("reminder-proof-{}", Uuid::new_v4()));
        let state =
            ClinicState::for_tests_with_fixtures(path.clone(), fixture.clone(), fixture.clone())
                .unwrap();
        let sms = StoredProvider {
            id: "sms-provider".to_owned(),
            channel: "sms".to_owned(),
            kind: "twilio".to_owned(),
            account_id: "fixture".to_owned(),
            encrypted_secret: state.store.encrypt("sms-secret").unwrap(),
            from: "+15550000001".to_owned(),
            approved_template_id: "sms-template".to_owned(),
            encrypted_webhook_secret: state.store.encrypt("sms-webhook").unwrap(),
        };
        let email = StoredProvider {
            id: "email-provider".to_owned(),
            channel: "email".to_owned(),
            kind: "resend".to_owned(),
            account_id: "fixture".to_owned(),
            encrypted_secret: state.store.encrypt("email-secret").unwrap(),
            from: "reminders@example.test".to_owned(),
            approved_template_id: "email-template".to_owned(),
            encrypted_webhook_secret: state.store.encrypt("email-webhook").unwrap(),
        };
        let workspace = ClinicWorkspace {
            organization_id: "fixture-org".to_owned(),
            clinic_name: "Fixture Dental".to_owned(),
            location_name: "Main".to_owned(),
            timezone: "UTC".to_owned(),
            provider_configs: vec![sms, email],
            reminders: vec![appointment(&[("sms", "allowed"), ("email", "allowed")])],
            subscription: Subscription {
                tier: Some("clinic".to_owned()),
                status: Some("active".to_owned()),
                checked_at: Some(now()),
                encrypted_entitlement: Some(
                    state.store.encrypt("fixture-license-token-1234").unwrap(),
                ),
            },
            ..Default::default()
        };
        state.store.save("fixture-owner", &workspace).unwrap();
        let app = crate::app_with_clinic_state("fixture", "../../dist", state.clone());
        let dispatched = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/clinic/reminders/dispatch")
                    .header(header::AUTHORIZATION, "Bearer test:fixture-owner")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("idempotency-key", "fixture-dispatch-42")
                    .body(axum::body::Body::from(r#"{"reminder_id":"r1"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(dispatched.status(), StatusCode::OK);
        let stored = state.store.get("fixture-owner").unwrap().unwrap();
        assert_eq!(stored.reminders[0].status, "provider-accepted");
        assert!(stored.reminders[0]
            .timeline
            .iter()
            .any(
                |event| event.channel.as_deref() == Some("sms") && event.kind == "provider-failed"
            ));
        assert!(stored.reminders[0]
            .timeline
            .iter()
            .any(|event| event.channel.as_deref() == Some("email")
                && event.provider_reference.as_deref() == Some("email-fixture-42")));

        let receipt = record_receipt(
            &state,
            "email-provider",
            ReceiptInput {
                provider_reference: "email-fixture-42".to_owned(),
                provider_event_id: "fixture-receipt-42".to_owned(),
                outcome: "delivered".to_owned(),
                occurred_at: now(),
            },
        )
        .await
        .unwrap();
        assert_eq!(receipt, StatusCode::NO_CONTENT);
        let delivered = state.store.get("fixture-owner").unwrap().unwrap();
        assert_eq!(delivered.reminders[0].status, "delivered");

        fixture_task.abort();
        let _ = fs::remove_dir_all(path);
    }

    #[tokio::test]
    async fn managed_claim_billing_checkout_and_return_activates_subscription() {
        let (fixture, fixture_task) = fixture_gateway().await;
        let path = std::env::temp_dir().join(format!("reminder-proof-{}", Uuid::new_v4()));
        let state =
            ClinicState::for_tests_with_fixtures(path.clone(), fixture.clone(), fixture.clone())
                .unwrap();
        let workspace = ClinicWorkspace {
            organization_id: "fixture-org".to_owned(),
            clinic_name: "Fixture Dental".to_owned(),
            location_name: "Main".to_owned(),
            timezone: "UTC".to_owned(),
            ..Default::default()
        };
        state.store.save("fixture-owner", &workspace).unwrap();

        for tier in ["clinic", "practice", "network"] {
            let checkout = billing_checkout(
                State(state.clone()),
                {
                    let mut headers = HeaderMap::new();
                    headers.insert(
                        header::AUTHORIZATION,
                        HeaderValue::from_static("Bearer test:fixture-owner"),
                    );
                    headers
                },
                Json(BillingQuery {
                    tier: tier.to_owned(),
                }),
            )
            .await
            .unwrap()
            .0;
            assert!(checkout.checkout_url.starts_with(&fixture));
            assert_eq!(checkout.tier, tier);
            assert!(checkout.checkout_url.contains(&format!("tier={tier}")));
            let response = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap()
                .get(&checkout.checkout_url)
                .send()
                .await
                .unwrap();
            assert_eq!(response.status(), reqwest::StatusCode::SEE_OTHER);
        }
        let returned = billing_return(
            State(state.clone()),
            {
                let mut headers = HeaderMap::new();
                headers.insert(
                    header::AUTHORIZATION,
                    HeaderValue::from_static("Bearer test:fixture-owner"),
                );
                headers
            },
            Json(BillingReturnInput {
                license: "fixture-license-token-1234".to_owned(),
                tier: Some("practice".to_owned()),
            }),
        )
        .await
        .unwrap()
        .0;
        assert_eq!(returned.subscription.status.as_deref(), Some("active"));
        assert_eq!(returned.subscription.tier.as_deref(), Some("practice"));
        let duplicate = billing_return(
            State(state.clone()),
            {
                let mut headers = HeaderMap::new();
                headers.insert(
                    header::AUTHORIZATION,
                    HeaderValue::from_static("Bearer test:fixture-owner"),
                );
                headers
            },
            Json(BillingReturnInput {
                license: "fixture-license-token-1234".to_owned(),
                tier: Some("practice".to_owned()),
            }),
        )
        .await
        .unwrap()
        .0;
        assert_eq!(duplicate.subscription.status.as_deref(), Some("active"));
        let subscriptions: i64 = state
            .store
            .connection
            .lock()
            .unwrap()
            .query_row("SELECT COUNT(*) FROM subscriptions", [], |row| row.get(0))
            .unwrap();
        assert_eq!(subscriptions, 1);
        let mut cached = state.store.get("fixture-owner").unwrap().unwrap();
        let database = fs::read(path.join("clinic-data.sqlite3")).unwrap();
        assert!(!database
            .windows("fixture-license-token-1234".len())
            .any(|window| window == b"fixture-license-token-1234"));

        fixture_task.abort();
        ensure_active_subscription(&state, &mut cached)
            .await
            .unwrap();
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn m2_claim_reversible_account_migration_round_trips() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(include_str!("../migrations/0001_managed_clinic.sql"))
            .unwrap();
        connection
            .execute_batch(include_str!(
                "../migrations/0002_accounts_subscriptions.up.sql"
            ))
            .unwrap();
        let applied: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM schema_migrations WHERE version=2",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(applied, 1);
        connection
            .execute_batch(include_str!(
                "../migrations/0002_accounts_subscriptions.down.sql"
            ))
            .unwrap();
        let removed: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='organizations'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(removed, 0);
        connection
            .execute_batch(include_str!(
                "../migrations/0002_accounts_subscriptions.up.sql"
            ))
            .unwrap();
        let restored: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='subscriptions'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(restored, 1);
    }

    #[tokio::test]
    async fn m2_claim_tenant_roles_and_onboarding_survive_restart() {
        let path = std::env::temp_dir().join(format!("reminder-proof-m2-{}", Uuid::new_v4()));
        let state = ClinicState::for_tests(path.clone()).unwrap();
        let application = crate::app_with_clinic_state("m2", "../../dist", state.clone());
        let created = application
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/organizations")
                    .header(header::AUTHORIZATION, "Bearer test:owner-a")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({
                            "clinic_name":"North Street Dental",
                            "location_name":"Main desk",
                            "timezone":"Europe/London",
                            "jurisdiction":"uk",
                            "retention_days":90,
                            "owner_name":"Morgan Lee",
                            "staff":[{"user_oid":"viewer-a","display_name":"Sam Rivera","email":"sam@example.test","role":"viewer"}]
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(created.status(), StatusCode::CREATED);
        let body = created.into_body().collect().await.unwrap().to_bytes();
        let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(created["jurisdiction"], "uk");
        assert_eq!(created["retention_days"], 90);
        assert_eq!(created["members"].as_array().unwrap().len(), 2);

        let viewer_read = application
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/me")
                    .header(header::AUTHORIZATION, "Bearer test:viewer-a")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(viewer_read.status(), StatusCode::OK);
        let body = viewer_read.into_body().collect().await.unwrap().to_bytes();
        let viewer: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(viewer["role"], "viewer");

        let forbidden_export = application
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/exports")
                    .header(header::AUTHORIZATION, "Bearer test:viewer-a")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(forbidden_export.status(), StatusCode::FORBIDDEN);
        let other_tenant = application
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/organizations")
                    .header(header::AUTHORIZATION, "Bearer test:owner-b")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = other_tenant.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
            serde_json::json!([])
        );

        drop(state);
        let reopened = ClinicState::for_tests(path.clone()).unwrap();
        let restored = reopened.store.get("owner-a").unwrap().unwrap();
        assert_eq!(restored.clinic_name, "North Street Dental");
        assert_eq!(restored.location_name, "Main desk");
        assert_eq!(restored.jurisdiction, "uk");
        assert_eq!(restored.retention_days, 90);
        let connection = reopened.store.connection.lock().unwrap();
        for table in [
            "users",
            "organizations",
            "locations",
            "memberships",
            "subscriptions",
            "audit_events",
            "notification_preferences",
        ] {
            let count: i64 = connection
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert!(
                count > 0,
                "{table} must contain the durable onboarding record"
            );
        }
        drop(connection);
        let _ = fs::remove_dir_all(path);
    }

    #[tokio::test]
    async fn m2_claim_export_and_seven_day_deletion_are_owner_controlled() {
        let path = std::env::temp_dir().join(format!("reminder-proof-m2-{}", Uuid::new_v4()));
        let state = ClinicState::for_tests(path.clone()).unwrap();
        let workspace = ClinicWorkspace {
            owner_oid: "fixture-owner".to_owned(),
            organization_id: "fixture-org".to_owned(),
            clinic_name: "Fixture Dental".to_owned(),
            location_name: "Main".to_owned(),
            timezone: "UTC".to_owned(),
            members: vec![ClinicMember {
                id: "owner-membership".to_owned(),
                user_oid: Some("fixture-owner".to_owned()),
                display_name: "Fixture Owner".to_owned(),
                email: String::new(),
                role: "owner".to_owned(),
                state: "active".to_owned(),
            }],
            ..Default::default()
        };
        state.store.save("fixture-owner", &workspace).unwrap();
        let application = crate::app_with_clinic_state("m2", "../../dist", state.clone());
        let export = application
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/exports")
                    .header(header::AUTHORIZATION, "Bearer test:fixture-owner")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(export.status(), StatusCode::OK);
        assert_eq!(
            export.headers().get(header::CONTENT_DISPOSITION).unwrap(),
            "attachment; filename=reminder-proof-export.json"
        );
        let jobs: i64 = state
            .store
            .connection
            .lock()
            .unwrap()
            .query_row("SELECT COUNT(*) FROM export_jobs", [], |row| row.get(0))
            .unwrap();
        assert_eq!(jobs, 1);

        let scheduled = application
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/account-deletion")
                    .header(header::AUTHORIZATION, "Bearer test:fixture-owner")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(scheduled.status(), StatusCode::ACCEPTED);
        let saved = state.store.get("fixture-owner").unwrap().unwrap();
        let deletion = saved.deletion.unwrap();
        assert_eq!(deletion.cancel_until - deletion.scheduled_at, 7 * 86_400);

        let cancelled = application
            .oneshot(
                axum::http::Request::builder()
                    .method("DELETE")
                    .uri("/api/v1/account-deletion")
                    .header(header::AUTHORIZATION, "Bearer test:fixture-owner")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(cancelled.status(), StatusCode::OK);
        assert!(state
            .store
            .get("fixture-owner")
            .unwrap()
            .unwrap()
            .deletion
            .is_none());
        let _ = fs::remove_dir_all(path);
    }
}
