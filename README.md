# soroban-devkit-contracts

> Reference and example Soroban smart contracts for the Soroban DevKit ecosystem.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/Raveu-lab/soroban-devkit-contract/actions/workflows/ci.yml/badge.svg)](https://github.com/Raveu-lab/soroban-devkit-contract/actions)

---

## What is this?

`soroban-devkit-contracts` is the contracts repository for the Soroban DevKit. It serves two purposes:

1. **Reference contracts** — well-documented, production-quality Soroban contracts that demonstrate best practices for common patterns (tokens, access control, upgradeable contracts, multi-sig, etc.)
2. **Test fixtures** — the contracts used by `soroban-devkit-core` and `soroban-devkit-cli` for integration testing and live demos

Every contract in this repo is:
- Written in Rust using the Soroban SDK
- Fully tested with the Soroban test harness
- Annotated with inline documentation explaining every design decision
- Deployed to testnet with a known contract ID for use in examples

---

## Why it exists

Most Soroban examples in the wild are minimal stubs — just enough to demonstrate a single concept. When developers try to build something real, they hit a wall: how do you structure an upgradeable contract? How do you emit events correctly? How do you handle access control without a standard library?

This repo answers those questions with complete, readable, real-world contracts that you can use as a reference or fork as a starting point.

---

## Prerequisites

- Rust (stable) — install via [rustup](https://rustup.rs)
- Soroban CLI — `cargo install --locked soroban-cli`
- Docker (for running a local Stellar node) — [docker.com](https://docker.com)

---

## Quick Start

Clone and build all contracts:

```bash
git clone https://github.com/Raveu-lab/soroban-devkit-contract
cd soroban-devkit-contract
cargo build --target wasm32v1-none --release
```

Run all tests:

```bash
cargo test
```

---

## Contracts

### `token`

A SEP-41 fungible token contract. Implements the state-changing `transfer`, `approve`, `transfer_from`, and `mint` (each emitting a structured event), plus the read-only `balance` and `allowance` views. `burn` and `clawback` are on the roadmap — the `clawback_enabled` storage flag is scaffolded, but neither function is callable yet.

```
contracts/token/
├── src/
│   ├── lib.rs        # Contract entry point and public interface
│   ├── storage.rs    # Ledger key definitions and TTL management
│   ├── events.rs     # Typed event emission helpers
│   ├── errors.rs     # ContractError enum
│   └── types.rs      # Shared data types
└── Cargo.toml
```

---

### `access-control`

A role-based access control contract that can be used as a dependency by other contracts. Supports multiple roles, role assignment, and role revocation.

```
contracts/access-control/
├── src/
│   ├── lib.rs
│   ├── storage.rs
│   ├── events.rs
│   └── errors.rs
└── Cargo.toml
```

---

### `upgradeable`

Demonstrates the correct pattern for deploying an upgradeable Soroban contract using WASM hash replacement. Includes a migration function and version tracking.

```
contracts/upgradeable/
├── src/
│   ├── lib.rs
│   ├── storage.rs
│   ├── events.rs
│   ├── errors.rs
│   └── migration.rs
└── Cargo.toml
```

---

### `multisig`

A multi-signature contract that holds funds and requires M-of-N signers to approve any transfer. A good reference for threshold authorization patterns.

```
contracts/multisig/
├── src/
│   ├── lib.rs
│   ├── storage.rs
│   ├── events.rs
│   ├── errors.rs
│   ├── types.rs
│   └── token_client.rs
└── Cargo.toml
```

---

### `event-rich`

A minimal contract designed specifically to emit a wide variety of event types and structures. This is the primary test fixture for `soroban-devkit-core`'s `EventDecoder` and `ContractMonitor`.

```
contracts/event-rich/
├── src/
│   └── lib.rs
└── Cargo.toml
```

---



## Project Structure

```
soroban-devkit-contracts/
├── contracts/
│   ├── token/
│   ├── access-control/
│   ├── upgradeable/
│   ├── multisig/
│   └── event-rich/
├── deployments.json
├── Cargo.toml          # Workspace manifest
├── CONTRIBUTING.md
└── README.md
```

---

## Deployments

All contracts are deployed to **Stellar testnet**. Contract IDs are tracked in [`deployments.json`](deployments.json) and [`ADDRESSES.md`](ADDRESSES.md).

| Contract | Testnet ID |
|----------|-----------|
| `token` | `CCNGTMOQNIF5VFJCHCF6S2CGW473IN76RPAX72YOTGDXC6VDZ4XINN45` |
| `access-control` | `CBFYOBMQF4Z625UVAG4C53KNJ7JVXNFRNBKMRQUCSY2YMORE5FI65QU6` |
| `upgradeable` | `CB2VSNSMBEOYZN2GJRZYTW6PYQAEMNFPCFJKW3YMQEDZKGXOLLKH3QQP` |
| `multisig` | `CCJQWDZ7TDPVUJMBPXCMBMVZ4WTGXVJZZ4DZTAJ3BCG2KQJFDX5B7J4C` |
| `event-rich` | `CBHSJRE3FJD7DZPNHQF66LGBQXPYCR425LLXPMUIX2IVHK6EKGMCE26K` |

---

This is the **best entry point for new contributors** — writing a Rust contract or adding tests to an existing one is one of the most accessible ways to contribute to the ecosystem.

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup instructions and how to pick up an issue.

**Good first issues** are tagged [`good first issue`](https://github.com/Raveu-lab/soroban-devkit-contract/issues?q=label%3A%22good+first+issue%22) on GitHub.

---

## Roadmap

- [ ] `token`: `burn` and `clawback` functions (storage flag for clawback already scaffolded)
- [ ] `escrow` contract — time-locked escrow with dispute resolution
- [ ] `vesting` contract — linear token vesting with cliff
- [ ] `oracle` contract — a simple price feed interface
- [ ] `dao-voting` contract — on-chain proposal and voting
- [ ] Property-based fuzz tests for all contracts using `cargo-fuzz`
- [ ] Formal verification annotations using Komet

---

## License

MIT — see [LICENSE](LICENSE).

Built for the Stellar ecosystem.
