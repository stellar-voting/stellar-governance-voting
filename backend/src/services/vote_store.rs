use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use crate::error::ApiError;
use crate::models::{ProposalDto, ProposalStatus, VoteOption};

#[derive(Debug, Clone)]
struct StoredProposal {
    id: u64, title: String, description: String, proposer: String,
    start_time: u64, end_time: u64, status: ProposalStatus,
    yes_votes: u64, no_votes: u64, abstain_votes: u64, ledger: u32,
}

pub struct VoteStore {
    proposals: RwLock<Vec<StoredProposal>>,
    voters: RwLock<HashSet<String>>,
    voted: RwLock<HashSet<(u64, String)>>,
    admin: RwLock<Option<String>>,
    paused: RwLock<bool>,
    now: RwLock<u64>,
}

impl VoteStore {
    pub fn new() -> Self {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        Self {
            proposals: RwLock::new(Vec::new()),
            voters: RwLock::new(HashSet::new()),
            voted: RwLock::new(HashSet::new()),
            admin: RwLock::new(None),
            paused: RwLock::new(false),
            now: RwLock::new(now),
        }
    }

    pub fn initialize(&self, admin: String) -> Result<(), ApiError> {
        let mut cur = self.admin.write().unwrap();
        if cur.is_some() { return Err(ApiError::InvalidInput("Already initialized".into())); }
        *cur = Some(admin); Ok(())
    }
    pub fn is_initialized(&self) -> bool { self.admin.read().unwrap().is_some() }
    pub fn get_admin(&self) -> Result<String, ApiError> { self.admin.read().unwrap().clone().ok_or(ApiError::NotAuthorized) }
    pub fn proposal_count(&self) -> u64 { self.proposals.read().unwrap().len() as u64 }
    pub fn voter_count(&self) -> u64 { self.voters.read().unwrap().len() as u64 }
    pub fn is_paused(&self) -> bool { *self.paused.read().unwrap() }
    pub fn pause(&self) { *self.paused.write().unwrap() = true; }
    pub fn unpause(&self) { *self.paused.write().unwrap() = false; }

    pub fn register_voter(&self, addr: &str) { self.voters.write().unwrap().insert(addr.into()); }
    pub fn revoke_voter(&self, addr: &str) { self.voters.write().unwrap().remove(addr); }
    pub fn is_voter(&self, addr: &str) -> bool { self.voters.read().unwrap().contains(addr) }

    pub fn create_proposal(&self, title: &str, desc: &str, proposer: &str, duration: u64) -> Result<u64, ApiError> {
        if *self.paused.read().unwrap() { return Err(ApiError::InvalidInput("Paused".into())); }
        if title.is_empty() { return Err(ApiError::InvalidInput("Title required".into())); }
        if duration < 60 || duration > 365*24*60*60 { return Err(ApiError::InvalidInput("Invalid duration".into())); }
        let now = *self.now.read().unwrap();
        let mut props = self.proposals.write().unwrap();
        let id = props.len() as u64;
        props.push(StoredProposal {
            id, title: title.into(), description: desc.into(), proposer: proposer.into(),
            start_time: now, end_time: now + duration, status: ProposalStatus::Active,
            yes_votes: 0, no_votes: 0, abstain_votes: 0, ledger: 1,
        });
        Ok(id)
    }

    pub fn get_proposal(&self, id: u64) -> Result<ProposalDto, ApiError> {
        self.proposals.read().unwrap().get(id as usize)
            .map(|p| p.to_dto()).ok_or(ApiError::ProposalNotFound(id))
    }

    pub fn get_all_proposals(&self) -> Vec<ProposalDto> {
        self.proposals.read().unwrap().iter().map(|p| p.to_dto()).collect()
    }

    pub fn vote(&self, voter: &str, proposal_id: u64, option: VoteOption) -> Result<(), ApiError> {
        if !self.is_voter(voter) { return Err(ApiError::NotEligible); }
        let voted_key = (proposal_id, voter.to_string());
        if self.voted.read().unwrap().contains(&voted_key) { return Err(ApiError::AlreadyVoted); }
        let mut props = self.proposals.write().unwrap();
        let prop = props.get_mut(proposal_id as usize).ok_or(ApiError::ProposalNotFound(proposal_id))?;
        if prop.status != ProposalStatus::Active { return Err(ApiError::VotingClosed); }
        let now = *self.now.read().unwrap();
        if now >= prop.end_time { return Err(ApiError::VotingClosed); }
        match option {
            VoteOption::Yes => prop.yes_votes += 1,
            VoteOption::No => prop.no_votes += 1,
            VoteOption::Abstain => prop.abstain_votes += 1,
        }
        self.voted.write().unwrap().insert(voted_key);
        Ok(())
    }

    pub fn has_voted(&self, proposal_id: u64, voter: &str) -> bool {
        self.voted.read().unwrap().contains(&(proposal_id, voter.to_string()))
    }

    pub fn close_proposal(&self, id: u64) -> Result<(), ApiError> {
        let mut props = self.proposals.write().unwrap();
        let prop = props.get_mut(id as usize).ok_or(ApiError::ProposalNotFound(id))?;
        if prop.status != ProposalStatus::Active { return Ok(()); }
        let now = *self.now.read().unwrap();
        if now < prop.end_time { return Err(ApiError::InvalidInput("Voting not ended".into())); }
        prop.status = ProposalStatus::Closed;
        Ok(())
    }

    pub fn cancel_proposal(&self, id: u64) -> Result<(), ApiError> {
        let mut props = self.proposals.write().unwrap();
        let prop = props.get_mut(id as usize).ok_or(ApiError::ProposalNotFound(id))?;
        prop.status = ProposalStatus::Cancelled;
        Ok(())
    }

    pub fn seed_demo_data(&self) {
        if self.is_initialized() { return; }
        let _ = self.initialize("GDQP2PQPM3XKKD4FQ7LG7N4XQ4FQ7LG7N4XQ4FQ7LG7N4XQ4FQ7".into());
        
        let voters = ["GA2C5RFPE6GCKMY3US5GAB29QZYE7DYXTZPGHLDJ5YBJO7VOZKAUBP2X",
                       "GDFX4P3OZO5NHNXHKDI5G4QXTZKQO4NHPKWXDWQZQFILXKZKABZ6WZ2N",
                       "GCXV2VWQFJZWCY2PHEJF5XMQ5RZQ3PKXEXZQKOOYZOJXFFAAYQJYTSK"];
        for v in voters { self.register_voter(v); }

        let _ = self.create_proposal("Adopt SIP-20: Smart Contract Upgrade", "Vote to approve the Soroban protocol upgrade scheduled for Q1 2027.", "GDQP2PQPM3XKKD4FQ7LG7N4XQ4FQ7LG7N4XQ4FQ7LG7N4XQ4FQ7", 86400);
        let _ = self.create_proposal("Treasury: Allocate 50K XLM to Dev Grants", "Allocate 50,000 XLM from the community treasury to developer grants for Q4 2026.", "GDQP2PQPM3XKKD4FQ7LG7N4XQ4FQ7LG7N4XQ4FQ7LG7N4XQ4FQ7", 172800);
        let _ = self.create_proposal("Add USDC as supported asset", "Enable USDC as a supported asset for governance staking rewards.", "GDQP2PQPM3XKKD4FQ7LG7N4XQ4FQ7LG7N4XQ4FQ7LG7N4XQ4FQ7", 3600);

        // Cast some votes on proposal 0
        let _ = self.vote(voters[0], 0, VoteOption::Yes);
        let _ = self.vote(voters[1], 0, VoteOption::Yes);
        let _ = self.vote(voters[2], 0, VoteOption::No);

        // Cast votes on proposal 1
        let _ = self.vote(voters[0], 1, VoteOption::Yes);
        let _ = self.vote(voters[1], 1, VoteOption::Abstain);

        tracing::info!("Seeded 3 proposals, 3 voters, 5 votes");
    }
}

impl StoredProposal {
    fn to_dto(&self) -> ProposalDto {
        ProposalDto {
            id: self.id, title: self.title.clone(), description: self.description.clone(),
            proposer: self.proposer.clone(), start_time: self.start_time, end_time: self.end_time,
            status: self.status, yes_votes: self.yes_votes, no_votes: self.no_votes,
            abstain_votes: self.abstain_votes, ledger: self.ledger,
        }
    }
}

impl Default for VoteStore { fn default() -> Self { Self::new() } }
