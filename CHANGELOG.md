# Changelog

All notable changes to `soroban-devkit-contracts` are documented here.

Format loosely follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). This project hasn't cut a tagged release yet, so everything below lives under **Unreleased**. See [ADDRESSES.md](ADDRESSES.md) for the full deployment history (every redeploy, with the real reason and the old address it replaced).

## Unreleased

### Known issues

- Every contract defines a typed error enum (`errors.rs`, a stable `u32` discriminant per variant) but none of them are actually used anywhere in `lib.rs` — confirmed live by grepping every contract's `lib.rs` for its own error type: zero usages, 49 raw `panic!("...")` calls across the 8 contracts that panic at all. Callers get a string message, not a typed, stable error code they can match on.

### Added

- `token`, `access-control`, `upgradeable`, `multisig`, `event-rich` — the initial scaffold.
- `token`: `transfer_from` with real allowance decrement.
- `multisig`: `execute()` actually transfers tokens, instead of just recording state.
- `escrow` — time-locked escrow with dispute resolution (depositor/recipient/arbiter roles).
- `vesting` — linear vesting with a cliff.
- `token`: `burn` and `clawback`.
- `oracle` — admin-published price feed with staleness checking.
- `dao-voting` — on-chain proposal and voting, one-address-one-vote.

### Fixed

- `token.symbol()` returned a hardcoded `"DKT"` instead of the actually-initialized symbol.
- `upgradeable.migrate()` was not actually idempotent — calling it twice kept bumping the version.
- `access-control.initialize()` had no re-initialization guard, unlike every other contract — anyone could re-call it and hijack the super admin.
- `multisig.initialize()` didn't reject duplicate addresses in the signer list — `count_approvals()` sums one approval per *entry*, not per unique address, so a duplicate let one signer's approval count twice toward the threshold.
- `token`'s balance-moving functions (`mint`/`transfer`/`transfer_from`/`burn`/`clawback`/`approve`) didn't reject non-positive amounts — a negative amount could flip a transfer's arithmetic direction and let a caller mint themselves funds while draining the recipient.
- `multisig.propose()` didn't reject a non-positive amount, for the same reason.
- Persistent-storage TTL wasn't managed anywhere — `oracle`, `access-control`, `dao-voting`, `escrow`, `multisig`, and `vesting` each got this fixed per-contract: a default write's TTL (a few thousand ledgers) is nowhere near the network max (several million), so an untouched entry would eventually get archived from disuse, even though nothing about it changed.
- The same gap existed for **instance** storage, more severely — losing an instance makes the whole contract inoperable, not just one entry. Fixed across `oracle`, `access-control`, `token` (which had *no* TTL management at all before this), `multisig`, `upgradeable`, `dao-voting`, `escrow`, and `vesting`. `upgradeable`'s case was the worst: `upgrade()`/`migrate()` are the only way to ever fix a deployed contract, and both are admin-gated, reading the admin from the instance.
- `token.approve()`'s `expiry_ledger` (an absolute ledger sequence) was passed straight into `extend_ttl()`'s relative-count parameter, so an allowance silently outlived its stated expiration. `approve(0)` — SEP-41's way to revoke an allowance — was also rejected by the blanket positive-amount guard.
- `dao-voting.propose()` had no guard against `voting_duration == 0`, unlike `vesting`'s equivalent field — a proposal with `voting_duration=0` got `deadline = now`, silently un-votable by anyone, forever.
- The bug-report issue template's "Which contract?" dropdown only listed 5 of 9 contracts.

### Changed

- Upgraded to `soroban-sdk` 27.0.5.
