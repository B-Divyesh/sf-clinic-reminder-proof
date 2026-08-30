use std::{
    env,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use axum::{
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::demo::problem_response;

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
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as usize;
        if !claims_match_contract(
            &claims,
            &self.tenant_id,
            &self.client_id,
            &issuer,
            current_time,
        ) {
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

fn claims_match_contract(
    claims: &Claims,
    tenant_id: &str,
    client_id: &str,
    issuer: &str,
    current_time: usize,
) -> bool {
    let audience_matches = match &claims.aud {
        serde_json::Value::String(value) => value == client_id,
        serde_json::Value::Array(values) => {
            values.iter().any(|value| value.as_str() == Some(client_id))
        }
        _ => false,
    };
    claims.tid == tenant_id
        && claims.iss == issuer
        && audience_matches
        && claims.exp > current_time
        && claims
            .nbf
            .is_some_and(|not_before| not_before <= current_time)
        && !claims.oid.trim().is_empty()
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
        let mut response = problem_response(status, code, message);
        response
            .headers_mut()
            .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_claims() -> Claims {
        Claims {
            oid: "11111111-1111-1111-1111-111111111111".to_owned(),
            tid: DEFAULT_TENANT.to_owned(),
            aud: serde_json::Value::String(DEFAULT_CLIENT.to_owned()),
            iss: format!("https://{DEFAULT_SUBDOMAIN}.ciamlogin.com/{DEFAULT_TENANT}/v2.0"),
            exp: 2_000,
            nbf: Some(900),
            name: Some("Fixture Owner".to_owned()),
        }
    }

    #[test]
    fn m2_claim_ciam_contract_rejects_wrong_registered_claims() {
        let issuer = format!("https://{DEFAULT_SUBDOMAIN}.ciamlogin.com/{DEFAULT_TENANT}/v2.0");
        let valid = valid_claims();
        assert!(claims_match_contract(
            &valid,
            DEFAULT_TENANT,
            DEFAULT_CLIENT,
            &issuer,
            1_000
        ));

        let mut wrong_issuer = valid_claims();
        wrong_issuer.iss = "https://issuer.invalid/v2.0".to_owned();
        assert!(!claims_match_contract(
            &wrong_issuer,
            DEFAULT_TENANT,
            DEFAULT_CLIENT,
            &issuer,
            1_000
        ));

        let mut wrong_audience = valid_claims();
        wrong_audience.aud = serde_json::Value::String("another-client".to_owned());
        assert!(!claims_match_contract(
            &wrong_audience,
            DEFAULT_TENANT,
            DEFAULT_CLIENT,
            &issuer,
            1_000
        ));

        let mut wrong_tenant = valid_claims();
        wrong_tenant.tid = "another-tenant".to_owned();
        assert!(!claims_match_contract(
            &wrong_tenant,
            DEFAULT_TENANT,
            DEFAULT_CLIENT,
            &issuer,
            1_000
        ));

        let mut expired = valid_claims();
        expired.exp = 999;
        assert!(!claims_match_contract(
            &expired,
            DEFAULT_TENANT,
            DEFAULT_CLIENT,
            &issuer,
            1_000
        ));

        let mut early = valid_claims();
        early.nbf = Some(1_001);
        assert!(!claims_match_contract(
            &early,
            DEFAULT_TENANT,
            DEFAULT_CLIENT,
            &issuer,
            1_000
        ));
    }

    #[test]
    fn ciam_public_config_uses_the_shared_sociobot_tenant() {
        let config = AuthService::from_env().public_config();
        assert_eq!(config.tenant_id, DEFAULT_TENANT);
        assert_eq!(config.client_id, DEFAULT_CLIENT);
        assert_eq!(
            config.authority,
            format!("https://{DEFAULT_SUBDOMAIN}.ciamlogin.com/{DEFAULT_TENANT}/")
        );
    }
}
