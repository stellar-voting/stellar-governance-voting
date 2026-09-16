use axum::extract::{Path, State};
use axum::Json;
use crate::error::ApiError;
use crate::models::*;
use crate::state::AppState;

pub async fn list_proposals(State(s): State<AppState>) -> Json<Vec<ProposalDto>> {
    Json(s.store.get_all_proposals())
}

pub async fn get_proposal(State(s): State<AppState>, Path(id): Path<u64>) -> Result<Json<ProposalDto>, ApiError> {
    Ok(Json(s.store.get_proposal(id)?))
}

pub async fn proposal_count(State(s): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "count": s.store.proposal_count() }))
}

pub async fn create_proposal(State(s): State<AppState>, Json(req): Json<CreateProposalRequest>) -> Result<Json<OperationResult>, ApiError> {
    let id = s.store.create_proposal(&req.title, &req.description, &req.proposer, req.voting_duration)?;
    tracing::info!(proposal_id = id, "Proposal created");
    Ok(Json(OperationResult { success: true, message: format!("Proposal {id} created") }))
}

pub async fn cast_vote(State(s): State<AppState>, Json(req): Json<VoteRequest>) -> Result<Json<OperationResult>, ApiError> {
    s.store.vote(&req.voter, req.proposal_id, req.option)?;
    tracing::info!(proposal = req.proposal_id, voter = %req.voter, "Vote cast");
    Ok(Json(OperationResult { success: true, message: "Vote recorded".into() }))
}

pub async fn get_results(State(s): State<AppState>, Path(id): Path<u64>) -> Result<Json<ResultsDto>, ApiError> {
    let p = s.store.get_proposal(id)?;
    Ok(Json(ResultsDto { yes: p.yes_votes, no: p.no_votes, abstain: p.abstain_votes }))
}

pub async fn close_proposal(State(s): State<AppState>, Path(id): Path<u64>) -> Result<Json<OperationResult>, ApiError> {
    s.store.close_proposal(id)?;
    Ok(Json(OperationResult { success: true, message: format!("Proposal {id} closed") }))
}

pub async fn cancel_proposal(State(s): State<AppState>, Path(id): Path<u64>) -> Result<Json<OperationResult>, ApiError> {
    s.store.cancel_proposal(id)?;
    Ok(Json(OperationResult { success: true, message: format!("Proposal {id} cancelled") }))
}

pub async fn voter_count(State(s): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "count": s.store.voter_count() }))
}

pub async fn register_voter(State(s): State<AppState>, Json(req): Json<VoterRequest>) -> Json<OperationResult> {
    s.store.register_voter(&req.voter);
    Json(OperationResult { success: true, message: "Voter registered".into() })
}

pub async fn revoke_voter(State(s): State<AppState>, Json(req): Json<VoterRequest>) -> Json<OperationResult> {
    s.store.revoke_voter(&req.voter);
    Json(OperationResult { success: true, message: "Voter revoked".into() })
}

pub async fn check_voter(State(s): State<AppState>, Path(addr): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "address": addr, "is_eligible": s.store.is_voter(&addr) }))
}

pub async fn has_voted(State(s): State<AppState>, Path((addr, pid)): Path<(String, u64)>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "voter": addr, "proposal_id": pid, "has_voted": s.store.has_voted(pid, &addr) }))
}

pub async fn get_admin(State(s): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let admin = s.store.get_admin()?;
    Ok(Json(serde_json::json!({ "admin": admin })))
}

pub async fn pause(State(s): State<AppState>) -> Json<OperationResult> {
    s.store.pause(); Json(OperationResult { success: true, message: "Paused".into() })
}

pub async fn unpause(State(s): State<AppState>) -> Json<OperationResult> {
    s.store.unpause(); Json(OperationResult { success: true, message: "Resumed".into() })
}

pub async fn is_paused(State(s): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "paused": s.store.is_paused() }))
}
