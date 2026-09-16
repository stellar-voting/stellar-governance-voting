# 🗳️ Stellar Governance Voting

A decentralized governance voting platform on the Stellar blockchain using
Soroban smart contracts. Built with Rust/Axum backend and React/TypeScript frontend.

![License](https://img.shields.io/badge/license-MIT-blue)
![Network](https://img.shields.io/badge/network-Stellar%20Testnet-7d4cdb)

## What It Does

An admin creates proposals with configurable voting periods. Registered voters
cast Yes/No/Abstain votes. Results are tallied on-chain. Proposals can be closed
after voting ends or cancelled by the admin.

## Key Features
- Proposal creation with title, description, and voting duration
- Single vote per voter per proposal (Yes/No/Abstain)
- Voter registration and revocation by admin
- Proposal lifecycle: Active → Closed/Cancelled
- Pause/unpause circuit breaker
- Dark mode + mobile responsive
- 28 tests (18 contract + 10 backend)

## Quick Start
```bash
cd backend && cargo run    # API at :3000 with demo data
cd frontend && npm install && npm run dev  # App at :5175
```

## Testing
```bash
cd contracts && cargo test   # 18 tests
cd backend && cargo test     # 10 tests
```

## Project Structure
```
contracts/    # Soroban contract (Rust) — 18 tests
backend/      # Axum REST API — 10 tests
frontend/     # React app — 6 views
docs/         # Architecture + deployment
.github/      # CI/CD
```

## License
MIT
