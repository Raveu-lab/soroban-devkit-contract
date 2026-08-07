# Architecture — soroban-devkit-contracts

## Overview

`soroban-devkit-contracts` is a Rust workspace containing multiple Soroban smart contracts. Each contract is an independent crate under the `contracts/` directory. They share no runtime code — the workspace exists purely for unified build, test, and release tooling.

Contracts in this repo serve two roles:
1. **Reference implementations** — production-quality patterns for the ecosystem
2. **Test fixtures** — deployed on testnet for integration testing by `soroban-devkit-core` and `soroban-devkit-cli`

---

## Repository Structure

```
soroban-devkit-contracts/
├── contracts/
│   ├── token/                  # SEP-41 fungible token
│   ├── access-control/         # Role-based access control
│   ├── upgradeable/            # WASM-upgradeable contract pattern
│   ├── multisig/               # M-of-N multi-signature wallet
│   └── event-rich/             # Event emission test fixture
├── deployments.json            # Testnet contract IDs
├── Cargo.toml                  # Workspace manifest
├── ARCHITECTURE.md
└── README.md
```

---

## Workspace Layout

`Cargo.toml` at the root defines the workspace and shared dependency versions:

```toml
[workspace]
members = [
  "contracts/token",
  "contracts/access-control",
  "contracts/upgradeable",
  "contracts/multisig",
  "contracts/event-rich",
]
resolver = "2"

[workspace.dependencies]
soroban-sdk = { version = "21.0.0", features = ["testutils"] }
```

Each contract's `Cargo.toml` references workspace dependencies:

```toml
[dependencies]
soroban-sdk = { workspace = true }
```

This ensures all contracts stay on the same SDK version and simplifies upgrades.

---

## Contract Architecture Pattern

Every contract in this repo follows the same internal layout:

```
contracts/<name>/
├── src/
│   ├── lib.rs          # Contract entry point — #[contract] struct + #[contractimpl]
│   ├── storage.rs      # Ledger key definitions, read/write helpers, TTL bumps
│   ├── events.rs       # Typed event structs + emit() helpers
│   ├── errors.rs       # ContractError enum
│   └── types.rs        # Shared data types (structs used in storage and args)
├── Cargo.toml
└── README.md           # Contract-specific usage notes
```

**Rule:** `lib.rs` contains only the public contract interface. All storage access goes through `storage.rs`. All event emission goes through `events.rs`. This makes each concern independently testable and readable.

---

## Contract Breakdown

### `token`

A complete SEP-41 compliant fungible token.

**Key design decisions:**
- Admin authority stored in a persistent ledger entry (not hardcoded)
- All state-changing functions emit a corresponding event
- Allowances use a composite key `(owner, spender)` with TTL management
- `clawback` is gated behind a separate `clawback_enabled` flag set at initialization

**Storage keys:**
```
Balance(Address)          → i128
Allowance(Address, Address) → i128
Admin                     → Address
Metadata                  → TokenMetadata { name, symbol, decimals }
ClawbackEnabled           → bool
```

---

### `access-control`

A standalone role-based access control contract intended to be composed with other contracts via cross-contract calls.

**Key design decisions:**
- Roles are represented as `Symbol` values — open-ended, not an enum
- Role assignments stored as `RoleMember(role, address) → bool`
- Supports role admin — each role has a designated admin role that can grant/revoke it
- Emits `RoleGranted` and `RoleRevoked` events on every change

---

### `upgradeable`

Demonstrates the correct pattern for Soroban WASM upgrades.

**Flow:**
```
deploy v1 contract
  │
  └─ upgrade(new_wasm_hash)        ← admin-gated
       │
       └─ env.deployer().update_current_contract_wasm(hash)
            │
            └─ migrate()           ← called once after upgrade to transform state
```

**Key design decisions:**
- `version: u32` stored in persistent storage, incremented on each migration
- Migration function is a no-op by default — contributors implement the actual state transformation
- Upgrade is behind an admin check using the `access-control` contract pattern

---

### `multisig`

An M-of-N threshold signature wallet.

**State:**
```
Signers        → Vec<Address>
Threshold      → u32
Proposals      → Map<u32, Proposal>
Approvals      → Map<(u32, Address), bool>
ProposalCount  → u32
```

**Proposal lifecycle:**
```
propose(transfer_args)  → proposal_id
  │
approve(proposal_id)    ← each signer calls this
  │
execute(proposal_id)    ← callable once approvals >= threshold
```

---

### `event-rich`

A minimal contract with no real business logic. Its sole purpose is to emit events covering every XDR `ScVal` type so that `EventDecoder` in `soroban-devkit-core` can be tested against real on-chain data.

**Functions:**
- `emit_primitive_types()` — emits u32, i32, u64, i64, bool, void
- `emit_big_numbers()` — emits u128, i128
- `emit_strings()` — emits Symbol, String, Bytes
- `emit_collections()` — emits Vec, Map
- `emit_address(addr: Address)` — emits an Address type

---

## Build

Build all contracts to WASM:

```bash
cargo build --target wasm32-unknown-unknown --release
```

WASM artifacts are output to:
```
target/wasm32-unknown-unknown/release/<contract_name>.wasm
```

---

## Testing Strategy

Each contract has two levels of tests, both in `src/lib.rs` under `#[cfg(test)]`:

**Unit tests** — test individual functions in isolation using `soroban_sdk::testutils`:
```rust
#[test]
fn test_transfer_updates_balance() {
    let env = Env::default();
    // ...
}
```

**Scenario tests** — test multi-step user flows end to end:
```rust
#[test]
fn test_full_approval_and_transfer_from_flow() {
    // mint → approve → transfer_from → check balances
}
```

Run all tests:
```bash
cargo test
```

---

## Deployments

Testnet deployments are tracked in `deployments.json`. When a contract is redeployed (e.g. after an upgrade), the file is updated and committed.

```json
{
  "testnet": {
    "token":          "C...",
    "access-control": "C...",
    "upgradeable":    "C...",
    "multisig":       "C...",
    "event-rich":     "C..."
  }
}
```

The `soroban-devkit-core` integration tests read this file to know which contract IDs to test against.

---

## Adding a New Contract

1. Create `contracts/<name>/` with the standard layout above
2. Add the crate to the workspace `members` list in the root `Cargo.toml`
3. Implement the contract following the module pattern (`lib.rs`, `storage.rs`, `events.rs`, `errors.rs`)
4. Write unit and scenario tests
5. Deploy to testnet and add the contract ID to `deployments.json`
6. Update this document and the root `README.md`
