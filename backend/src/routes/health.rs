use axum::extract::State;
use axum::Json;
use crate::models::{HealthResponse, StatusResponse};
use crate::state::AppState;

pub async fn health(State(s): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok", version: env!("CARGO_PKG_VERSION"), uptime_seconds: s.started_at.elapsed().as_secs() })
}

pub async fn status(State(s): State<AppState>) -> Json<StatusResponse> {
    let c = &s.config;
    Json(StatusResponse {
        status: "ok", version: env!("CARGO_PKG_VERSION"), uptime_seconds: s.started_at.elapsed().as_secs(),
        network: c.network_name.clone(), contract_id: c.contract_id.clone(),
        proposal_count: s.store.proposal_count(), voter_count: s.store.voter_count(),
    })
}
