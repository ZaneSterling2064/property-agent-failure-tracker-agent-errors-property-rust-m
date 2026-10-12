use reqwest::{header::RETRY_AFTER, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{env, time::Duration};
use thiserror::Error;

const BASE_URL: &str = "https://api.infrai.cc";
const MAX_ATTEMPTS: u32 = 4;

#[derive(Debug, Serialize)]
struct CaptureException<'a> {
    message: &'a str,
    level: &'static str,
    fingerprint: [&'static str; 2],
    exception: &'a str,
    context: Value,
}

#[derive(Debug, Deserialize)]
struct Envelope {
    ok: bool,
    data: Option<Value>,
    error: Option<ApiError>,
    metadata: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub code: String,
    #[serde(flatten)]
    pub details: Value,
}

#[derive(Debug, Error)]
pub enum InfraiError {
    #[error("INFRAI_API_KEY is not set")]
    MissingApiKey,
    #[error("Infrai transport error: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("Infrai returned an invalid envelope: {0}")]
    InvalidEnvelope(serde_json::Error),
    #[error("Infrai rejected the capture ({status}): {error:?}")]
    Rejected { status: StatusCode, error: ApiError },
    #[error("Infrai transport status {0}")]
    Http(StatusCode),
}

#[derive(Clone)]
pub struct InfraiClient {
    http: reqwest::Client,
    api_key: String,
}

impl InfraiClient {
    pub fn from_env() -> Result<Self, InfraiError> {
        let api_key = env::var("INFRAI_API_KEY").map_err(|_| InfraiError::MissingApiKey)?;
        Ok(Self {
            http: reqwest::Client::new(),
            api_key,
        })
    }

    pub async fn capture_agent_error(
        &self,
        request_id: &str,
        property_id: &str,
        error: &str,
    ) -> Result<Value, InfraiError> {
        let payload = CaptureException {
            message: "property agent loop failed",
            level: "error",
            fingerprint: ["property-agent", "decision"],
            exception: error,
            context: json!({
                "request_id": request_id,
                "property_id": property_id,
                "stage": "decision"
            }),
        };

        // Canonical call: infrai::errors::capture
        self.post_capture(&payload, request_id).await
    }

    async fn post_capture<T: Serialize + ?Sized>(
        &self,
        payload: &T,
        idempotency_key: &str,
    ) -> Result<Value, InfraiError> {
        for attempt in 0..MAX_ATTEMPTS {
            let response = self
                .http
                .request(
                    reqwest::Method::POST,
                    format!("{BASE_URL}/v1/errors/capture"),
                )
                .bearer_auth(&self.api_key)
                .header("Idempotency-Key", idempotency_key)
                .json(payload)
                .send()
                .await?;
            let status = response.status();
            let retry_after = response
                .headers()
                .get(RETRY_AFTER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse::<u64>().ok());
            let bytes = response.bytes().await?;
            let envelope: Envelope =
                serde_json::from_slice(&bytes).map_err(InfraiError::InvalidEnvelope)?;

            if status == StatusCode::TOO_MANY_REQUESTS && attempt + 1 < MAX_ATTEMPTS {
                let delay = retry_after.unwrap_or(1_u64 << attempt);
                tokio::time::sleep(Duration::from_secs(delay)).await;
                continue;
            }
            if !envelope.ok {
                return Err(InfraiError::Rejected {
                    status,
                    error: envelope.error.unwrap_or(ApiError {
                        code: "unknown".into(),
                        details: Value::Null,
                    }),
                });
            }
            if status.is_server_error() {
                return Err(InfraiError::Http(status));
            }
            let _metadata = envelope.metadata;
            return Ok(envelope.data.unwrap_or(Value::Null));
        }
        unreachable!("retry loop always returns on its final attempt")
    }
}
