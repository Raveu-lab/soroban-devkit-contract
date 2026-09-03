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
│   ├── event-rich/             # Event emission test fixture
│   ├── escrow/                 # Time-locked escrow with dispute resolution
│   └── vesting/                # Linear token vesting with a cliff
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
  "contracts/escrow",
  "contracts/vesting",
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

Every contract in this repo follows the same internal layout. Not all files are required — only create the ones that apply:

```
contracts/<name>/
├── src/
│   ├── lib.rs          # Contract entry point — #[contract] struct + #[contractimpl]
│   ├── storage.rs      # Ledger key definitions, read/write helpers, TTL bumps
│   ├── events.rs       # Typed event structs + emit() helpers
│   ├── errors.rs       # ContractError enum
│   └── types.rs        # Shared data types (optional — only when needed)
├── Cargo.toml
└── README.md           # Contract-specific usage notes (optional)
```

**Rule:** `lib.rs` contains only the public contract interface. All storage access goes through `storage.rs`. All event emission goes through `events.rs`. This makes each concern independently testable and readable.

---

## Contract Breakdown

### `token`

A complete SEP-41 compliant fungible token.

**Key design decisions:**
- Admin authority stored in instance storage (not hardcoded)
- All state-changing functions emit a corresponding event
- Allowances use a composite key `(owner, spender)` with TTL management
- `clawback_enabled` is fixed at `initialize()` — like Stellar classic assets, it cannot be toggled afterward. `clawback()` panics if it was never enabled.
- `burn()` requires auth from the token holder (self-service); `clawback()` requires auth from the admin and works on any address — the two exist for genuinely different purposes despite both reducing a balance without a corresponding recipient.

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
  │
  └─ cross-contract call to proposal.token's transfer(contract_address, to, amount)
```

**Cross-contract calls:** `token_client.rs` declares a minimal `TokenInterface` trait via `#[contractclient]` rather than depending on `soroban-token`'s crate directly — `execute()` works against any deployed contract exposing the standard `transfer(from, to, amount)` signature, not just this repo's own token contract. The multisig contract authorizes the transfer as itself via `env.current_contract_address()`, so it must hold the token balance it's proposing to send.

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

### `escrow`

A time-locked escrow contract with dispute resolution. One contract instance holds many escrows, each identified by an incrementing ID — the same registry pattern as `multisig`'s proposals.

**State (per escrow):**
```
Escrow(id) → { depositor, recipient, arbiter, token, amount, release_time, status }
EscrowCount → u32
```

**Lifecycle:**
```
deposit(depositor, recipient, arbiter, token, amount, release_time) → escrow_id
  │  pulls `amount` of `token` from depositor into the contract's own custody
  │
  ├─ release(caller, id)   ← depositor: any time. anyone: once release_time has passed.
  ├─ refund(caller, id)    ← recipient only (voluntary give-back)
  │
  └─ dispute(caller, id)   ← depositor or recipient, while Active
       │
       └─ release(arbiter, id) / refund(arbiter, id)   ← only the arbiter, once Disputed
```

**Key design decisions:**
- Like `multisig`, cross-contract token transfers go through a minimal `#[contractclient]`-declared `TokenInterface` (`token_client.rs`) rather than depending on `soroban-token`'s crate — works against any SEP-41-shaped token, not just this repo's own.
- Once a dispute is raised, the normal depositor/timelock/recipient rules for `release`/`refund` no longer apply — only the arbiter can resolve it, in either direction.
- `release` and `refund` are only valid from `Active` (or `Disputed`, for the arbiter path) — both are terminal once `Released` or `Refunded`.

---

### `vesting`

Linear token vesting with a cliff. One contract instance holds many schedules, each identified by an incrementing ID — the same registry pattern as `escrow`/`multisig`.

**State (per schedule):**
```
VestingSchedule(id) → { depositor, beneficiary, token, total_amount,
                         claimed_amount, start_time, cliff_duration,
                         vesting_duration, revoked }
ScheduleCount → u32
```

**Vesting curve** (`vested_at`, pure function, no storage access):
```
elapsed = now - start_time
elapsed < cliff_duration        → 0
elapsed >= vesting_duration     → total_amount
otherwise                       → total_amount * elapsed / vesting_duration
```
Vesting accrues linearly from `start_time` the whole time — the cliff only gates *when* it becomes claimable, not when it starts accruing. Once the cliff passes, whatever has already accrued becomes claimable in one shot.

**Lifecycle:**
```
create_vesting(depositor, beneficiary, token, total_amount,
               start_time, cliff_duration, vesting_duration) → schedule_id
  │  pulls `total_amount` of `token` from depositor into the contract's own custody
  │
  ├─ claim(beneficiary, id)   → pays vested_at(now) - claimed_amount
  │
  └─ revoke(depositor, id)    → pays the beneficiary whatever vested by now,
                                  refunds the unvested remainder to depositor,
                                  sets claimed_amount = total_amount (blocks
                                  further claims without a separate check)
```

**Key design decisions:**
- Like `escrow`/`multisig`, cross-contract token transfers go through a minimal `#[contractclient]`-declared `TokenInterface` — works against any SEP-41-shaped token.
- `revoke` settles the schedule in one transaction: it pays out earned-but-unclaimed tokens to the beneficiary (they keep what they've earned) and refunds only the unvested portion to the depositor — total distributed always equals `total_amount`.
- `#[allow(clippy::too_many_arguments)]` on `create_vesting`: 7 real parameters plus `env` is a genuine business requirement here, not something worth hiding behind an options struct just to satisfy the lint.

---

## Build

Build all contracts to WASM:

```bash
cargo build --target wasm32v1-none --release
```

WASM artifacts are output to:
```
target/wasm32v1-none/release/<contract_name>.wasm
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

Testnet deployments are tracked in `deployments.json`. When a contract is deployed or redeployed, the file is updated and committed.

All 7 contracts are live on testnet:

| Contract | Testnet ID |
|----------|-----------|
| `token` | `CB5YCY5CYLNO3PTH3OXQKKT6XFXTSNIOYSC5B65XE4ZZE6MVIWGD2LNH` |
| `access-control` | `CBFYOBMQF4Z625UVAG4C53KNJ7JVXNFRNBKMRQUCSY2YMORE5FI65QU6` |
| `upgradeable` | `CB2VSNSMBEOYZN2GJRZYTW6PYQAEMNFPCFJKW3YMQEDZKGXOLLKH3QQP` |
| `multisig` | `CCJQWDZ7TDPVUJMBPXCMBMVZ4WTGXVJZZ4DZTAJ3BCG2KQJFDX5B7J4C` |
| `event-rich` | `CBHSJRE3FJD7DZPNHQF66LGBQXPYCR425LLXPMUIX2IVHK6EKGMCE26K` |
| `escrow` | `CAHTJ7KOOIHITNV2HOCZXXGLS4ZXD64RZNOKQALLQ3ROIRBM6ZM27W2M` |
| `vesting` | `CDH42CTIXQ3OFEFHQTTBHR3IJ4HPEUNC2REM6DXH3K2QL23YKZY4K5W5` |

The `soroban-devkit-core` integration tests read `deployments.json` to resolve contract IDs at test time.

---

## Adding a New Contract

1. Create `contracts/<name>/` with the standard layout above
2. Add the crate to the workspace `members` list in the root `Cargo.toml`
3. Implement the contract following the module pattern (`lib.rs`, `storage.rs`, `events.rs`, `errors.rs`)
4. Write unit and scenario tests
5. Update this document and the root `README.md`
