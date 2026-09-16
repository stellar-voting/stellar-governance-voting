# Architecture

## Overview

Stellar Governance Voting is a decentralized governance protocol on the Stellar
blockchain. An admin creates proposals with configurable voting periods.
Registered voters cast single votes (Yes/No/Abstain) per proposal. Results are
tallied on-chain and proposals can be closed or cancelled.

## System Architecture

```
┌──────────────────────────────────────────────────────────┐
│                   Frontend (React)                        │
│  Dashboard · Proposals · Create · Vote · Admin · Wallet   │
└──────────────────────┬───────────────────────────────────┘
                       │ HTTP
┌──────────────────────┼───────────────────────────────────┐
│                Backend API (Axum)                         │
│  /proposals · /vote · /voters · /admin · /pause          │
│  VoteStore: in-memory proposal/vote management            │
└──────────────────────┬───────────────────────────────────┘
                       │ (production: Soroban RPC)
┌──────────────────────┼───────────────────────────────────┐
│              Stellar Testnet (Soroban)                    │
│  GovernanceContract: create_proposal · vote · close      │
│  register_voter · cancel_proposal · pause/unpause        │
└──────────────────────────────────────────────────────────┘
```

## Smart Contract

**Roles:** Admin (manages voters, proposals, pause), Voter (registered, casts votes)

**Proposal Lifecycle:** Active → Closed (after voting ends) or Cancelled (admin)

**Vote Options:** Yes, No, Abstain (one vote per voter per proposal)

**Key Parameters:**
- Max title: 128 chars, Max description: 1,024 chars
- Min voting duration: 60s, Max: 365 days

**Contract Functions:**
| Function | Access | Description |
|---|---|---|
| `initialize(admin)` | Once | Set admin |
| `create_proposal(...)` | Admin | Create governance proposal |
| `vote(voter, id, option)` | Voter | Cast vote |
| `close_proposal(id)` | Public | Close after voting ends |
| `cancel_proposal(id)` | Admin | Cancel proposal |
| `register_voter(voter)` | Admin | Register eligible voter |
| `revoke_voter(voter)` | Admin | Revoke voter eligibility |
| `get_proposal(id)` | Public | Fetch proposal |
| `get_results(id)` | Public | Get vote counts |
| `has_voted(id, voter)` | Public | Check if voted |
| `pause()/unpause()` | Admin | Toggle protocol |

## Backend API

18 REST endpoints: proposals CRUD, voting, voter management, admin controls, health.

## Frontend

6 views: Dashboard (stats + recent), Proposals (table + detail modal), Create (form),
Vote (cast votes on active proposals), Admin (manage voters/proposals, pause), Wallet.

Features: Dark mode (localStorage), mobile responsive, toast notifications.

## Tests
- Contract: 18 unit tests
- Backend: 10 integration tests
- Total: 28 tests
