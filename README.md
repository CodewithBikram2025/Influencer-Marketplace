# 🌟 Influencer Marketplace — Soroban Smart Contract

> A decentralized influencer marketplace built on the **Stellar** blockchain using **Soroban** smart contracts. Connects brands with influencers in a trustless, transparent, and on-chain environment.

---

## 📖 Project Description

**Influencer Marketplace** is a Soroban smart contract that eliminates the middleman between brands and content creators. Instead of relying on centralized platforms (with their opaque algorithms, hidden fees, and payment delays), this contract lets brands post campaigns, influencers pitch proposals, and payments flow directly on-chain — all governed by verifiable, open-source code on Stellar.

Whether you're a DTC brand launching a product or a micro-influencer looking to monetize your audience, this contract provides the foundational layer for a fully decentralized creator economy.

---

## ⚙️ What It Does

The contract manages the **full lifecycle** of a brand–influencer collaboration:

```
Brand creates Campaign
        │
        ▼
Influencer submits Proposal
        │
        ▼
Brand accepts Proposal  ──► Campaign moves to InProgress
        │
        ▼
Brand releases Payment  ──► Event emitted on-chain
        │
        ▼
Proposal marked Paid / Campaign Completed
```

All state — profiles, campaigns, proposals, payment records — lives on the **Stellar ledger**. No backend, no database, no trust required.

---

## ✨ Features

### 👤 Influencer Profiles
- Register a profile with handle, niche, follower count, and per-post rate
- Update or deactivate your profile at any time
- Auth-gated: only the profile owner can modify their own data

### 📣 Campaign Management
- Brands create campaigns with a title, description, and total budget (in stroops)
- Campaigns have typed statuses: `Open → InProgress → Completed | Cancelled`
- Budget tracking prevents overspending across multiple collaborations
- Brands can cancel open campaigns before any proposals are accepted

### 📝 Proposal Workflow
- Influencers submit proposals with a pitch, promised deliverables, and requested payment
- Budget guard: proposal ask is validated against remaining campaign budget
- Multiple proposals per campaign supported — brands pick the best fit
- Proposal statuses: `Pending → Accepted → Paid | Rejected`

### 💸 Payment Release
- Brand triggers `release_payment` after deliverables are confirmed
- On-chain disbursement tracking: `spent` counter updated per campaign
- Payment events emitted via `env.events()` for off-chain wallet and indexer integration
- Designed to plug into Stellar Asset Contract (SAC) for real XLM/USDC transfers

### 🔐 Authorization & Safety
- Every state-mutating function requires `Address::require_auth()` — no impersonation possible
- Ownership checks on all brand-side operations
- Budget overflow protection with explicit assertions
- `no_std` runtime — minimal attack surface, WASM-optimized binary

### 🧪 Test Coverage
- Full end-to-end test: register → campaign → proposal → accept → pay
- Cancel flow test
- Uses Soroban's built-in `testutils` with `mock_all_auths`

---

## 🗂️ Project Structure

```
influencer-marketplace/
├── Cargo.toml          # Soroban SDK dependency, WASM release profile
└── src/
    └── lib.rs          # Full contract: structs, enums, impl, tests
```

---

## 🚀 Getting Started

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install Stellar CLI
cargo install --locked stellar-cli --features opt
```

### Build

```bash
cd influencer-marketplace
stellar contract build
# Output: target/wasm32-unknown-unknown/release/influencer_marketplace.wasm
```

### Test

```bash
cargo test
```

### Deploy to Testnet

```bash
# Generate a keypair
stellar keys generate --global alice --network testnet

# Deploy
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/influencer_marketplace.wasm \
  --source alice \
  --network testnet
```

---

## 🔗 Deployed Contract

| Network | Contract ID |
|---|---|
| Testnet | `CCFHODV2EHXQNWPIYAAUJUNBJ33SHSMXZ5EI3IL5CK5EQYSRUYG5V7E6` |

You can inspect the contract live on the [Stellar Expert Explorer](https://stellar.expert/explorer/testnet/contract/CCFHODV2EHXQNWPIYAAUJUNBJ33SHSMXZ5EI3IL5CK5EQYSRUYG5V7E6).

To invoke a function against the deployed contract:

```bash
stellar contract invoke \
  --id CCFHODV2EHXQNWPIYAAUJUNBJ33SHSMXZ5EI3IL5CK5EQYSRUYG5V7E6 \
  --source alice \
  --network testnet \
  -- \
  get_campaign --campaign_id 1
```

---

## 📡 Contract Interface (Summary)

| Function | Caller | Description |
|---|---|---|
| `register_influencer` | Influencer | Create / update profile |
| `deactivate_influencer` | Influencer | Soft-delete profile |
| `get_influencer` | Anyone | Read a profile |
| `create_campaign` | Brand | Open a new campaign |
| `cancel_campaign` | Brand | Cancel an open campaign |
| `get_campaign` | Anyone | Read a campaign |
| `submit_proposal` | Influencer | Pitch for a campaign |
| `accept_proposal` | Brand | Accept an influencer's proposal |
| `release_payment` | Brand | Disburse payment & emit event |
| `get_proposals` | Anyone | List all proposals for a campaign |

---

## 🛣️ Roadmap

- [ ] Integrate Stellar Asset Contract (SAC) for real XLM / USDC escrow
- [ ] Escrow: hold budget on-chain until deliverables confirmed
- [ ] On-chain dispute resolution (arbitrator address)
- [ ] Reputation scoring stored per `Address`
- [ ] Multi-deliverable milestone payments
- [ ] Frontend dApp (Next.js + Freighter wallet)

---

## 🧰 Tech Stack

| Layer | Technology |
|---|---|
| Blockchain | [Stellar](https://stellar.org) |
| Smart Contract Runtime | [Soroban](https://soroban.stellar.org) |
| Language | Rust (`no_std`) |
| SDK | `soroban-sdk` v20 |
| CLI Tooling | `stellar-cli` |

---

## 📄 License

MIT — free to use, fork, and build upon.
