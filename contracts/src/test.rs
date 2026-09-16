#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger}, Address, Env, String,
};

fn setup() -> (Env, GovernanceContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(GovernanceContract, ());
    let client = GovernanceContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

fn make_proposal(env: &Env, client: &GovernanceContractClient, admin: &Address, title: &str, dur: u64) -> u64 {
    client.create_proposal(
        &String::from_str(env, title),
        &String::from_str(env, "A test proposal for governance voting"),
        admin,
        &dur,
    )
}

#[test]
fn initialize_sets_admin() {
    let (_env, client, admin) = setup();
    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.is_paused(), false);
    assert_eq!(client.get_proposal_count(), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn initialize_twice_panics() {
    let (_env, client, admin) = setup();
    client.initialize(&admin);
}

#[test]
fn register_and_revoke_voter() {
    let (env, client, _admin) = setup();
    let voter = Address::generate(&env);
    assert!(!client.is_eligible_voter(&voter));
    client.register_voter(&voter);
    assert!(client.is_eligible_voter(&voter));
    assert_eq!(client.get_voter_count(), 1);
    client.revoke_voter(&voter);
    assert!(!client.is_eligible_voter(&voter));
}

#[test]
fn create_proposal_happy_path() {
    let (env, client, admin) = setup();
    let id = make_proposal(&env, &client, &admin, "Proposal 1", 3600);
    assert_eq!(id, 0);
    assert_eq!(client.get_proposal_count(), 1);
    let p = client.get_proposal(&0);
    assert_eq!(p.title, String::from_str(&env, "Proposal 1"));
    assert_eq!(p.status, ProposalStatus::Active);
    assert_eq!(p.yes_votes, 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn create_proposal_empty_title() {
    let (env, client, admin) = setup();
    client.create_proposal(&String::from_str(&env, ""), &String::from_str(&env, "desc"), &admin, &3600);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn create_proposal_invalid_duration() {
    let (env, client, admin) = setup();
    client.create_proposal(&String::from_str(&env, "Title"), &String::from_str(&env, "desc"), &admin, &10);
}

#[test]
fn vote_happy_path() {
    let (env, client, admin) = setup();
    let voter = Address::generate(&env);
    client.register_voter(&voter);
    make_proposal(&env, &client, &admin, "Test", 3600);

    let result = client.vote(&voter, &0, &VoteOption::Yes);
    assert!(result);
    assert!(client.has_voted(&0, &voter));

    let (yes, no, abstain) = client.get_results(&0);
    assert_eq!(yes, 1);
    assert_eq!(no, 0);
    assert_eq!(abstain, 0);
}

#[test]
fn vote_all_options() {
    let (env, client, admin) = setup();
    let v1 = Address::generate(&env);
    let v2 = Address::generate(&env);
    let v3 = Address::generate(&env);
    client.register_voter(&v1);
    client.register_voter(&v2);
    client.register_voter(&v3);
    make_proposal(&env, &client, &admin, "Test", 3600);

    client.vote(&v1, &0, &VoteOption::Yes);
    client.vote(&v2, &0, &VoteOption::No);
    client.vote(&v3, &0, &VoteOption::Abstain);

    let (yes, no, abstain) = client.get_results(&0);
    assert_eq!(yes, 1);
    assert_eq!(no, 1);
    assert_eq!(abstain, 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn double_vote_rejected() {
    let (env, client, admin) = setup();
    let voter = Address::generate(&env);
    client.register_voter(&voter);
    make_proposal(&env, &client, &admin, "Test", 3600);
    client.vote(&voter, &0, &VoteOption::Yes);
    client.vote(&voter, &0, &VoteOption::No);
}

#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn non_eligible_voter_rejected() {
    let (env, client, admin) = setup();
    let voter = Address::generate(&env);
    make_proposal(&env, &client, &admin, "Test", 3600);
    client.vote(&voter, &0, &VoteOption::Yes);
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn vote_after_closed_rejected() {
    let (env, client, admin) = setup();
    let voter = Address::generate(&env);
    client.register_voter(&voter);
    make_proposal(&env, &client, &admin, "Test", 3600);
    env.ledger().with_mut(|l| l.timestamp += 4000);
    client.vote(&voter, &0, &VoteOption::Yes);
}

#[test]
fn close_proposal_after_voting() {
    let (env, client, admin) = setup();
    let voter = Address::generate(&env);
    client.register_voter(&voter);
    make_proposal(&env, &client, &admin, "Test", 3600);
    client.vote(&voter, &0, &VoteOption::Yes);

    env.ledger().with_mut(|l| l.timestamp += 4000);
    let status = client.close_proposal(&0);
    assert_eq!(status, ProposalStatus::Closed);

    let p = client.get_proposal(&0);
    assert_eq!(p.status, ProposalStatus::Closed);
    assert_eq!(p.yes_votes, 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn close_before_ended_rejected() {
    let (env, client, admin) = setup();
    make_proposal(&env, &client, &admin, "Test", 3600);
    client.close_proposal(&0);
}

#[test]
fn cancel_proposal_admin() {
    let (env, client, admin) = setup();
    make_proposal(&env, &client, &admin, "Test", 3600);
    client.cancel_proposal(&0);
    let p = client.get_proposal(&0);
    assert_eq!(p.status, ProposalStatus::Cancelled);
}

#[test]
fn pause_unpause_cycle() {
    let (_env, client, _admin) = setup();
    client.pause();
    assert!(client.is_paused());
    client.unpause();
    assert!(!client.is_paused());
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn create_proposal_blocked_when_paused() {
    let (env, client, admin) = setup();
    client.pause();
    make_proposal(&env, &client, &admin, "Test", 3600);
}

#[test]
fn multiple_proposals() {
    let (env, client, admin) = setup();
    let id0 = make_proposal(&env, &client, &admin, "P1", 3600);
    let id1 = make_proposal(&env, &client, &admin, "P2", 7200);
    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
    assert_eq!(client.get_proposal_count(), 2);
}

#[test]
fn get_results_reflects_votes() {
    let (env, client, admin) = setup();
    let v1 = Address::generate(&env);
    let v2 = Address::generate(&env);
    client.register_voter(&v1);
    client.register_voter(&v2);
    make_proposal(&env, &client, &admin, "Test", 3600);

    client.vote(&v1, &0, &VoteOption::Yes);
    client.vote(&v2, &0, &VoteOption::No);

    let (yes, no, abstain) = client.get_results(&0);
    assert_eq!(yes, 1);
    assert_eq!(no, 1);
    assert_eq!(abstain, 0);
}
