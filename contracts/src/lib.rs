#![no_std]
//! # Stellar Governance Voting — Soroban Contract
//!
//! A **decentralized governance voting protocol** on the Stellar network.
//! An admin creates proposals with configurable voting periods. Eligible voters
//! cast single votes per proposal. Results are tallied on-chain and can be
//! finalized by the admin or anyone after the voting period ends.

use soroban_sdk::{
    contract, contracterror, contractimpl, contractmeta, contracttype, panic_with_error,
    symbol_short, Address, Env, String,
};

contractmeta!(
    key = "Description",
    val = "Decentralized governance voting protocol on Stellar"
);

const MAX_TITLE_LEN: u32 = 128;
const MAX_DESC_LEN: u32 = 1024;
const MAX_VOTING_DURATION: u64 = 365 * 24 * 60 * 60;
const MIN_VOTING_DURATION: u64 = 60;

/// Status of a proposal.
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProposalStatus {
    Active = 0,
    Closed = 1,
    Cancelled = 2,
}

/// Vote options.
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VoteOption {
    Yes = 0,
    No = 1,
    Abstain = 2,
}

/// A governance proposal.
#[contracttype]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Proposal {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub proposer: Address,
    pub start_time: u64,
    pub end_time: u64,
    pub status: ProposalStatus,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub abstain_votes: u64,
    pub ledger: u32,
}

/// Storage keys.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Paused,
    ProposalCount,
    VoterCount,
    Proposal(u64),
    HasVoted(u64, Address),
    EligibleVoter(Address),
}

#[contracterror]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    Paused = 4,
    InvalidTitle = 5,
    InvalidDescription = 6,
    InvalidDuration = 7,
    ProposalNotFound = 8,
    VotingClosed = 9,
    AlreadyVoted = 10,
    NotEligibleVoter = 11,
    ProposalNotActive = 12,
    VotingNotEnded = 13,
}

#[contract]
pub struct GovernanceContract;

#[contractimpl]
impl GovernanceContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage().instance().set(&DataKey::ProposalCount, &0u64);
        env.storage().instance().set(&DataKey::VoterCount, &0u64);
        env.events().publish((symbol_short!("init"),), admin);
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotInitialized))
    }

    /// Register an eligible voter. Admin-only.
    pub fn register_voter(env: Env, voter: Address) {
        let admin = Self::read_admin(&env);
        admin.require_auth();
        env.storage().persistent().set(&DataKey::EligibleVoter(voter.clone()), &true);
        let count: u64 = env.storage().instance().get(&DataKey::VoterCount).unwrap_or(0);
        env.storage().instance().set(&DataKey::VoterCount, &(count + 1));
        env.events().publish((symbol_short!("regd"), admin), voter);
    }

    /// Revoke voter eligibility. Admin-only.
    pub fn revoke_voter(env: Env, voter: Address) {
        let admin = Self::read_admin(&env);
        admin.require_auth();
        env.storage().persistent().remove(&DataKey::EligibleVoter(voter.clone()));
        env.events().publish((symbol_short!("revoked"), admin), voter);
    }

    /// Check if an address is an eligible voter.
    pub fn is_eligible_voter(env: Env, voter: Address) -> bool {
        env.storage().persistent().get(&DataKey::EligibleVoter(voter)).unwrap_or(false)
    }

    /// Total number of registered voters.
    pub fn get_voter_count(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::VoterCount).unwrap_or(0)
    }

    /// Create a new proposal. Admin-only.
    pub fn create_proposal(
        env: Env,
        title: String,
        description: String,
        proposer: Address,
        voting_duration: u64,
    ) -> u64 {
        let admin = Self::read_admin(&env);
        admin.require_auth();
        Self::require_not_paused(&env);

        if title.len() == 0 || title.len() > MAX_TITLE_LEN {
            panic_with_error!(&env, Error::InvalidTitle);
        }
        if description.len() > MAX_DESC_LEN {
            panic_with_error!(&env, Error::InvalidDescription);
        }
        if voting_duration < MIN_VOTING_DURATION || voting_duration > MAX_VOTING_DURATION {
            panic_with_error!(&env, Error::InvalidDuration);
        }

        let now = env.ledger().timestamp();
        let id: u64 = env.storage().instance().get(&DataKey::ProposalCount).unwrap_or(0);

        let proposal = Proposal {
            id,
            title,
            description,
            proposer,
            start_time: now,
            end_time: now + voting_duration,
            status: ProposalStatus::Active,
            yes_votes: 0,
            no_votes: 0,
            abstain_votes: 0,
            ledger: env.ledger().sequence(),
        };

        env.storage().persistent().set(&DataKey::Proposal(id), &proposal);
        env.storage().instance().set(&DataKey::ProposalCount, &(id + 1));
        env.events().publish((symbol_short!("proposal"), admin), id);
        id
    }

    /// Get a proposal by ID.
    pub fn get_proposal(env: Env, id: u64) -> Proposal {
        env.storage().persistent().get(&DataKey::Proposal(id))
            .unwrap_or_else(|| panic_with_error!(&env, Error::ProposalNotFound))
    }

    /// Get total number of proposals.
    pub fn get_proposal_count(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::ProposalCount).unwrap_or(0)
    }

    /// Cast a vote on a proposal. Returns true on success.
    pub fn vote(env: Env, voter: Address, proposal_id: u64, option: VoteOption) -> bool {
        voter.require_auth();
        Self::require_initialized(&env);

        // Check eligibility.
        if !env.storage().persistent().get(&DataKey::EligibleVoter(voter.clone())).unwrap_or(false) {
            panic_with_error!(&env, Error::NotEligibleVoter);
        }

        // Check not already voted.
        let voted_key = DataKey::HasVoted(proposal_id, voter.clone());
        if env.storage().persistent().has(&voted_key) {
            panic_with_error!(&env, Error::AlreadyVoted);
        }

        // Get and validate proposal.
        let mut proposal: Proposal = env.storage().persistent().get(&DataKey::Proposal(proposal_id))
            .unwrap_or_else(|| panic_with_error!(&env, Error::ProposalNotFound));

        if proposal.status != ProposalStatus::Active {
            panic_with_error!(&env, Error::ProposalNotActive);
        }

        let now = env.ledger().timestamp();
        if now < proposal.start_time || now >= proposal.end_time {
            panic_with_error!(&env, Error::VotingClosed);
        }

        // Record vote.
        match option {
            VoteOption::Yes => proposal.yes_votes += 1,
            VoteOption::No => proposal.no_votes += 1,
            VoteOption::Abstain => proposal.abstain_votes += 1,
        }

        env.storage().persistent().set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage().persistent().set(&voted_key, &true);

        env.events().publish((symbol_short!("vote"), voter), (proposal_id, option as u32));
        true
    }

    /// Check if a voter has voted on a proposal.
    pub fn has_voted(env: Env, proposal_id: u64, voter: Address) -> bool {
        env.storage().persistent().has(&DataKey::HasVoted(proposal_id, voter))
    }

    /// Get vote counts for a proposal.
    pub fn get_results(env: Env, proposal_id: u64) -> (u64, u64, u64) {
        let proposal: Proposal = env.storage().persistent().get(&DataKey::Proposal(proposal_id))
            .unwrap_or_else(|| panic_with_error!(&env, Error::ProposalNotFound));
        (proposal.yes_votes, proposal.no_votes, proposal.abstain_votes)
    }

    /// Close a proposal after voting ends. Anyone can call.
    pub fn close_proposal(env: Env, proposal_id: u64) -> ProposalStatus {
        Self::require_initialized(&env);
        let mut proposal: Proposal = env.storage().persistent().get(&DataKey::Proposal(proposal_id))
            .unwrap_or_else(|| panic_with_error!(&env, Error::ProposalNotFound));

        if proposal.status != ProposalStatus::Active {
            return proposal.status;
        }

        let now = env.ledger().timestamp();
        if now < proposal.end_time {
            panic_with_error!(&env, Error::VotingNotEnded);
        }

        proposal.status = ProposalStatus::Closed;
        env.storage().persistent().set(&DataKey::Proposal(proposal_id), &proposal);

        env.events().publish((symbol_short!("closed"),), proposal_id);
        ProposalStatus::Closed
    }

    /// Cancel a proposal. Admin-only.
    pub fn cancel_proposal(env: Env, proposal_id: u64) {
        let admin = Self::read_admin(&env);
        admin.require_auth();
        let mut proposal: Proposal = env.storage().persistent().get(&DataKey::Proposal(proposal_id))
            .unwrap_or_else(|| panic_with_error!(&env, Error::ProposalNotFound));

        proposal.status = ProposalStatus::Cancelled;
        env.storage().persistent().set(&DataKey::Proposal(proposal_id), &proposal);
        env.events().publish((symbol_short!("cancelled"), admin), proposal_id);
    }

    /// Pause the protocol. Admin-only.
    pub fn pause(env: Env) {
        let admin = Self::read_admin(&env);
        admin.require_auth();
        env.storage().instance().set(&DataKey::Paused, &true);
        env.events().publish((symbol_short!("paused"),), admin);
    }

    pub fn unpause(env: Env) {
        let admin = Self::read_admin(&env);
        admin.require_auth();
        env.storage().instance().set(&DataKey::Paused, &false);
        env.events().publish((symbol_short!("unpaused"),), admin);
    }

    pub fn is_paused(env: Env) -> bool {
        env.storage().instance().get(&DataKey::Paused).unwrap_or(false)
    }

    fn read_admin(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(env, Error::NotInitialized))
    }

    fn require_initialized(env: &Env) {
        if !env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(env, Error::NotInitialized);
        }
    }

    fn require_not_paused(env: &Env) {
        let paused: bool = env.storage().instance().get(&DataKey::Paused).unwrap_or(false);
        if paused {
            panic_with_error!(env, Error::Paused);
        }
    }
}

#[cfg(test)]
mod test;
