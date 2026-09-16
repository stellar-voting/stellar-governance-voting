use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("proposal not found: {0}")]
    ProposalNotFound(u64),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("voting closed")]
    VotingClosed,
    #[error("already voted")]
    AlreadyVoted,
    #[error("not eligible")]
    NotEligible,
    #[error("not authorized")]
    NotAuthorized,
    #[allow(dead_code)]
    #[error("internal: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorResponse { error: &'static str, code: u16, message: String }

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self {
            ApiError::ProposalNotFound(id) => (StatusCode::NOT_FOUND, 404, format!("Proposal {id} not found")),
            ApiError::InvalidInput(m) => (StatusCode::BAD_REQUEST, 400, m.clone()),
            ApiError::VotingClosed => (StatusCode::BAD_REQUEST, 400, "Voting has closed".into()),
            ApiError::AlreadyVoted => (StatusCode::BAD_REQUEST, 400, "Already voted".into()),
            ApiError::NotEligible => (StatusCode::FORBIDDEN, 403, "Not an eligible voter".into()),
            ApiError::NotAuthorized => (StatusCode::FORBIDDEN, 403, "Not authorized".into()),
            ApiError::Internal(m) => { tracing::error!(%m); (StatusCode::INTERNAL_SERVER_ERROR, 500, "Internal error".into()) }
        };
        let label = match status { StatusCode::NOT_FOUND => "not_found", StatusCode::BAD_REQUEST => "bad_request", StatusCode::FORBIDDEN => "forbidden", _ => "internal_error" };
        (status, Json(ErrorResponse { error: label, code, message: msg })).into_response()
    }
}
