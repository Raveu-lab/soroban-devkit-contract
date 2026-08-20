# Contributing to soroban-devkit-contracts

This project is open source and welcomes contributions from the Stellar developer community.

This is the **best entry point for new contributors** — writing or extending a Soroban contract in Rust is one of the most accessible and impactful ways to contribute to the Stellar ecosystem.

---

## Prerequisites

- Rust (stable) — install via [rustup.rs](https://rustup.rs)
- Soroban CLI — `cargo install --locked soroban-cli`
- Docker — for running a local Stellar node (optional, testnet works without it)

---

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

---

## Coding Convention — Test-Driven Development (TDD)

This project follows **strict TDD**. Every contribution must follow this cycle:

```
1. Write a failing test that describes the behaviour you want
2. Write the minimal implementation to make the test pass
3. Refactor — clean up without changing behaviour
4. Repeat
```

**No implementation code is accepted without a corresponding test.**

### What this looks like in practice

Write the test first inside `#[cfg(test)]` in `src/lib.rs`:

```rust
#[test]
fn test_transfer_from_reduces_allowance() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TokenContract, ());
    let client = TokenContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let spender = Address::generate(&env);

    client.initialize(&admin, &String::from_str(&env, "Token"), &String::from_str(&env, "TKN"), &7);
    client.mint(&alice, &1_000_000);
    client.approve(&alice, &spender, &500_000, &1000);

    client.transfer_from(&spender, &alice, &spender, &200_000);

    // Allowance should be reduced
    assert_eq!(client.allowance(&alice, &spender), 300_000);
}
```

Run it — it should fail:

```bash
cargo test
# FAILED — transfer_from is not implemented
```

Then implement `transfer_from`. Run again — it should pass.

---

## SOLID Principles

- **Single Responsibility** — `lib.rs` is the public interface only. Storage logic in `storage.rs`. Event emission in `events.rs`. Error definitions in `errors.rs`
- **Small, focused functions** — each function does one thing. `set_balance`, `get_allowance`, `emit_transfer` — not one large function that does all three
- **Function names describe what they do** — `emit_role_granted`, `require_signer`, `set_proposal` — not `update` or `process`

---

## Code Standards

- Every public contract function must have a test
- Tests live in `src/lib.rs` under `#[cfg(test)]`
- No `unsafe` code
- No `unwrap()` in production code paths — use proper error handling with `ContractError`
- All events must go through `events.rs` — never call `env.events().publish` directly in `lib.rs`
- All storage access must go through `storage.rs` — never call `env.storage()` directly in `lib.rs`

---

## Picking Up an Issue

1. Browse [open issues](https://github.com/Raveu-lab/soroban-devkit-contract/issues)
2. Issues tagged `good first issue` are beginner-friendly
3. Comment to claim before starting work

---

## Good First Issues

- Implement a `TODO` marked with an issue number in `lib.rs`
- Add a missing test for an existing function
- Add a new event to an existing contract
- Fix a Clippy warning

---

## Adding a New Contract

Follow the standard layout:

```
contracts/<name>/
├── src/
│   ├── lib.rs       # Contract entry point and public interface only
│   ├── storage.rs   # Ledger key definitions and read/write helpers
│   ├── events.rs    # Event emission helpers
│   ├── errors.rs    # ContractError enum
│   └── types.rs     # Shared data types (if needed)
└── Cargo.toml
```

1. **Write tests first** — define the expected behaviour before implementing
2. Create the directory and files above
3. Add the crate to `members` in root `Cargo.toml`
4. Implement until all tests pass
5. Open a PR referencing the proposal issue

---

## Pull Request Guidelines

- Tests must be written before or alongside implementation — not after
- All tests must pass: `cargo test`
- Run `cargo fmt` and `cargo clippy -- -D warnings` before opening a PR
- Reference the issue: `feat: implement transfer_from in token contract (#3)`
- Each PR should touch one contract only
