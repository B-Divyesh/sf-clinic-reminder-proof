use std::{
    env,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::demo::Problem;

const DEFAULT_TENANT: &str = "35c6fe40-0ec0-46b6-98c6-213ad4de6650";
const DEFAULT_SUBDOMAIN: &str = "sociobotcustomers";
const DEFAULT_CLIENT: &str = "25c704f4-465a-47af-80ab-2c489466b697";

#[derive(Clone)]
pub struct AuthService {
    tenant_id: String,
    client_id: String,
    discovery_url: String,
    client: reqwest::Client,
    cache: Arc<RwLock<Option<CachedKeys>>>,
    #[cfg(test)]
    allow_test_tokens: bool,
}

#[derive(Clone, Debug)]
pub struct Identity {
    pub oid: String,
}

struct CachedKeys {
    issuer: String,
    keys: Vec<Jwk>,
    loaded: Instant,
}

#[derive(Deserialize)]
struct Discovery {
    issuer: String,
    jwks_uri: String,
}

#[derive(Clone, Deserialize)]
struct Jwk {
    kid: String,
    kty: String,
    n: String,
    e: String,
}

#[derive(Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    oid: String,
    tid: String,
    aud: serde_json::Value,
    iss: String,
    exp: usize,
    nbf: Option<usize>,
    name: Option<String>,
}

impl AuthService {
    pub fn from_env() -> Self {
        let tenant_id = env::var("ENTRA_TENANT_ID").unwrap_or_else(|_| DEFAULT_TENANT.to_owned());
        let subdomain =
            env::var("ENTRA_TENANT_SUBDOMAIN").unwrap_or_else(|_| DEFAULT_SUBDOMAIN.to_owned());
        let client_id = env::var("ENTRA_CLIENT_ID").unwrap_or_else(|_| DEFAULT_CLIENT.to_owned());
        Self {
            discovery_url: format!("https://{subdomain}.ciamlogin.com/{tenant_id}/v2.0/.well-known/openid-configuration"),
            tenant_id,
            client_id,
            client: reqwest::Client::builder().timeout(Duration::from_secs(8)).build().expect("valid auth client"),
            cache: Arc::new(RwLock::new(None)),
            #[cfg(test)]
            allow_test_tokens: false,
        }
    }

    #[cfg(test)]
    pub fn for_tests() -> Self {
        let mut service = Self::from_env();
        service.allow_test_tokens = true;
        service
    }

    pub async fn identity(&self, headers: &HeaderMap) -> Result<Identity, AuthError> {
        let token = headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .filter(|value| !value.is_empty())
            .ok_or(AuthError::Missing)?;

        #[cfg(test)]
        if self.allow_test_tokens {
            if let Some(oid) = token.strip_prefix("test:").filter(|oid| !oid.is_empty()) {
                return Ok(Identity {
                    oid: oid.to_owned(),
                });
            }
        }

        let header = decode_header(token).map_err(|_| AuthError::Invalid)?;
        if header.alg != Algorithm::RS256 {
            return Err(AuthError::Invalid);
        }
        let kid = header.kid.ok_or(AuthError::Invalid)?;
        let (issuer, keys) = self.keys().await?;
        let key = keys
            .iter()
            .find(|item| item.kid == kid && item.kty == "RSA")
            .ok_or(AuthError::Invalid)?;
        let decoding_key =
            DecodingKey::from_rsa_components(&key.n, &key.e).map_err(|_| AuthError::Invalid)?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.client_id]);
        validation.set_issuer(&[&issuer]);
        validation.set_required_spec_claims(&["aud", "exp", "iss", "nbf", "oid", "tid"]);
        let claims = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|_| AuthError::Invalid)?
            .claims;
        if claims.tid != self.tenant_id || claims.iss != issuer {
            return Err(AuthError::Invalid);
        }
        Ok(Identity { oid: claims.oid })
    }

    async fn keys(&self) -> Result<(String, Vec<Jwk>), AuthError> {
        if let Some(cached) = self.cache.read().await.as_ref() {
            if cached.loaded.elapsed() < Duration::from_secs(3600) {
                return Ok((cached.issuer.clone(), cached.keys.clone()));
            }
        }
        let discovery = self
            .client
            .get(&self.discovery_url)
            .send()
            .await
            .map_err(|_| AuthError::Unavailable)?
            .error_for_status()
            .map_err(|_| AuthError::Unavailable)?
            .json::<Discovery>()
            .await
            .map_err(|_| AuthError::Unavailable)?;
        let jwks = self
            .client
            .get(&discovery.jwks_uri)
            .send()
            .await
            .map_err(|_| AuthError::Unavailable)?
            .error_for_status()
            .map_err(|_| AuthError::Unavailable)?
            .json::<Jwks>()
            .await
            .map_err(|_| AuthError::Unavailable)?;
        *self.cache.write().await = Some(CachedKeys {
            issuer: discovery.issuer.clone(),
            keys: jwks.keys.clone(),
            loaded: Instant::now(),
        });
        Ok((discovery.issuer, jwks.keys))
    }

    pub fn public_config(&self) -> AuthConfig {
        AuthConfig {
            tenant_id: self.tenant_id.clone(),
            client_id: self.client_id.clone(),
            authority: self
                .discovery_url
                .trim_end_matches("v2.0/.well-known/openid-configuration")
                .to_owned(),
        }
    }
}

#[derive(Serialize)]
pub struct AuthConfig {
    pub tenant_id: String,
    pub client_id: String,
    pub authority: String,
}

#[derive(Debug)]
pub enum AuthError {
    Missing,
    Invalid,
    Unavailable,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::Missing => (
                StatusCode::UNAUTHORIZED,
                "bearer_required",
                "Sign in with your clinic account, then try again.",
            ),
            Self::Invalid => (
                StatusCode::UNAUTHORIZED,
                "bearer_invalid",
                "Your sign-in token is not valid for Reminder Proof. Sign in again.",
            ),
            Self::Unavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "identity_unavailable",
                "Identity validation is temporarily unavailable. Try again shortly.",
            ),
        };
        let mut response = (
            status,
            Json(Problem {
                code,
                message,
                request_id: "available-in-response-header",
            }),
        )
            .into_response();
        response
            .headers_mut()
            .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        response
    }
}
