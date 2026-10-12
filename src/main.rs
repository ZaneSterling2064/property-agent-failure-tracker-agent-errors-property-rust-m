mod infrai;
mod property_agent;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use infrai::{InfraiClient, InfraiError};
use property_agent::{decide, AgentDecision, AgentError, AgentInput};
use serde::Serialize;
use std::net::SocketAddr;
use thiserror::Error;

#[derive(Clone)]
struct ServiceState {
    infrai: InfraiClient,
}

#[derive(Debug, Error)]
enum ServiceError {
    #[error(transparent)]
    Agent(#[from] AgentError),
    #[error(transparent)]
    Tracking(#[from] InfraiError),
}

#[derive(Serialize)]
struct DecisionResponse {
    request_id: String,
    decision: AgentDecision,
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::Agent(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Tracking(InfraiError::Rejected { status, .. }) if status.is_client_error() => {
                *status
            }
            Self::Tracking(InfraiError::MissingApiKey) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Tracking(_) => StatusCode::BAD_GATEWAY,
        };
        (
            status,
            Json(serde_json::json!({ "error": self.to_string() })),
        )
            .into_response()
    }
}

async fn run_agent(
    State(state): State<ServiceState>,
    Json(input): Json<AgentInput>,
) -> Result<Json<DecisionResponse>, ServiceError> {
    let request_id = input.maintenance.request_id.clone();
    let property_id = input.maintenance.property_id.clone();
    match decide(&input) {
        Ok(decision) => Ok(Json(DecisionResponse {
            request_id,
            decision,
        })),
        Err(error) => {
            state
                .infrai
                .capture_agent_error(&request_id, &property_id, &error.to_string())
                .await?;
            Err(error.into())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = ServiceState {
        infrai: InfraiClient::from_env()?,
    };
    let app = Router::new()
        .route("/agent/run", post(run_agent))
        .with_state(state);
    let address = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!("property agent listening on http://{address}");
    axum::serve(listener, app).await?;
    Ok(())
}
