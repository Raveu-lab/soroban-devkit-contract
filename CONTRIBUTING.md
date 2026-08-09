# Contributing to soroban-devkit-contracts

This project is part of the **Stellar Wave Program** on [Drips](https://drips.network). Contributors earn rewards for completing issues during active Wave sprints.

This is the **best entry point for new contributors** — writing or extending a Soroban contract in Rust is one of the most accessible and impactful ways to contribute to the Stellar ecosystem.

## Prerequisites

- Rust (stable) — install via [rustup.rs](https://rustup.rs)
- Soroban CLI — `cargo install --locked soroban-cli`
- Docker — for running a local Stellar node (optional, testnet works without it)

## Setup

```bash
git clone https://github.com/Raveu-lab/soroban-devkit-contract
cd soroban-devkit-contract

# Build all contracts
cargo build --target wasm32-unknown-unknown --release

# Run all tests
cargo test

# Check formatting
cargo fmt --all -- --check

# Lint
cargo clippy --all-targets -- -D warnings
```

## Picking Up an Issue

1. Browse [open issues](https://github.com/Raveu-lab/soroban-devkit-contract/issues)
2. Issues tagged `good first issue` are beginner-friendly
3. Comment to claim before starting work

## Good First Issues

- Add tests to an existing contract
- Implement a `TODO` marked with an issue number
- Add a new event to an existing contract
- Fix a Clippy warning

## Adding a New Contract

Follow the standard layout:

```
contracts/<name>/
├── src/
│   ├── lib.rs       # Contract entry point
│   ├── storage.rs   # Ledger key definitions
│   ├── events.rs    # Event emission helpers
│   ├── errors.rs    # ContractError enum
│   └── types.rs     # Shared data types
└── Cargo.toml
```

1. Create the directory and files above
2. Add the crate to `members` in root `Cargo.toml`
3. Write unit and scenario tests in `src/lib.rs`
4. Deploy to testnet and add the contract ID to `deployments.json`
5. Open a PR referencing the proposal issue

## Pull Request Guidelines

- Run `cargo test`, `cargo fmt`, and `cargo clippy` before opening a PR
- Reference the issue: `feat: implement transfer_from in token contract (#3)`
- Each PR should touch one contract only
- No unsafe code
