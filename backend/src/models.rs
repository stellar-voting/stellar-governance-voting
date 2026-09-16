use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus { Active, Closed, Cancelled }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoteOption { Yes, No, Abstain }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalDto {
    pub id: u64, pub title: String, pub description: String,
    pub proposer: String, pub start_time: u64, pub end_time: u64,
    pub status: ProposalStatus, pub yes_votes: u64, pub no_votes: u64, pub abstain_votes: u64,
    pub ledger: u32,
}

#[derive(Debug, Deserialize)]
pub struct CreateProposalRequest {
    pub title: String, pub description: String, pub proposer: String, pub voting_duration: u64,
}

#[derive(Debug, Deserialize)]
pub struct VoteRequest {
    pub voter: String, pub proposal_id: u64, pub option: VoteOption,
}

#[derive(Debug, Deserialize)]
pub struct VoterRequest { pub voter: String }

#[derive(Debug, Serialize)]
pub struct HealthResponse { pub status: &'static str, pub version: &'static str, pub uptime_seconds: u64 }

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub status: &'static str, pub version: &'static str, pub uptime_seconds: u64,
    pub network: String, pub contract_id: String,
    pub proposal_count: u64, pub voter_count: u64,
}

#[derive(Debug, Serialize)]
pub struct OperationResult { pub success: bool, pub message: String }

#[derive(Debug, Serialize)]
pub struct ResultsDto { pub yes: u64, pub no: u64, pub abstain: u64 }
