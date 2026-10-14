# Demo — the safety guards, enforced on-chain

These contracts are deployed on Stellar testnet. This walkthrough calls the
**live deployments** and shows their validation actually rejecting bad input —
not a unit test asserting it would.

Every output block below is captured from a real run. Reproduce it with
[`run.sh`](run.sh).

No key, no funded account, no local build. The script uses
[`sdev simulate`](https://github.com/Raveu-lab/soroban-devkit-cli), which
simulates against the public RPC and needs only public addresses:

```bash
npm install -g @soroban-devkit/cli
./demo/run.sh

# or, without installing, against a local checkout of the CLI:
SDEV_JS=../soroban-devkit-cli/dist/cli.js ./demo/run.sh
```

Addresses come from [`ADDRESSES.md`](../ADDRESSES.md).

---

## `vesting` — a cliff may not outlast the vesting duration

A schedule whose cliff is longer than its own vesting window is not a vesting
schedule. `vested_at` returns 0 for the whole cliff — including the entire
stretch past `vesting_duration` where the curve says fully vested — then jumps
straight to the full amount the instant the cliff clears. The linear portion is
unreachable, so `vesting_duration` silently means nothing.

The contract rejects it at creation, **before** the token transfer pulls the
depositor's funds.

```bash
# cliff_duration = 1000, vesting_duration = 100
sdev simulate --contract CB3DUVIY... --method create_vesting --caller GC3B... \
  --args '[..., {"$u64":"0"}, {"$u64":"1000"}, {"$u64":"100"}]' --network testnet
```

```
✖ Simulation failed
  HostError: Error(Contract, #7)
   0: [Diagnostic Event] contract:CB3DUVIY..., topics:[error, Error(Contract, #7)],
      data:"escalating error to VM trap from failed host function call: fail_with_error"
```

`Error(Contract, #7)` is `VestingError::CliffExceedsDuration` — a typed,
matchable discriminant, not a string. A caller can branch on it.

The same call with the cliff inside the duration is accepted:

```bash
# cliff_duration = 100, vesting_duration = 1000
```

```
✔ Simulation successful

  Return Value     : 1
```

`cliff_duration == vesting_duration` is allowed too — that's a coherent
pure-cliff schedule, and a test pins the boundary.

---

## `escrow` — the arbiter must be a third party

Once an escrow is `Disputed`, only the arbiter can release or refund, and the
`Active` branch becomes unreachable along with the timelock path anyone could
use after `release_time`. So an arbiter who is also a party owns the outcome: a
depositor who names themselves arbiter can dispute their own escrow and refund
themselves at any time, leaving the recipient no recourse.

```bash
# depositor and arbiter are the same address
sdev simulate --contract CB26WWGM... --method deposit --caller GC3B... \
  --args '["GC3B...", "GCTQ...", "GC3B...", ...]' --network testnet
```

```
✖ Simulation failed

  0: [Diagnostic Event] contract:CB26WWGM..., topics:[error, Error(WasmVm, InvalidAction)],
     data:["VM call trapped: UnreachableCodeReached", deposit]
```

With a neutral arbiter it goes through:

```
✔ Simulation successful

  Return Value     : 1
```

`depositor == recipient` stays allowed: locking your own funds until
`release_time` behind a neutral arbiter is a legitimate self-timelock. Only the
arbiter's neutrality is a security property, so that is all this guards.

---

### A note on those two error shapes

`vesting` reports `Error(Contract, #7)`. `escrow` reports only
`VM call trapped: UnreachableCodeReached`.

Both rejected the bad input, but only one told you *why*. `vesting` has been
converted to `panic_with_error!` with a typed `VestingError` enum; `escrow`
still raises raw `panic!("...")` strings, and a string panic carries no
discriminant across the host boundary — so the caller gets an opaque trap.

Converting the remaining contracts is tracked as good-first-issues
([#11](https://github.com/Raveu-lab/soroban-devkit-contract/issues/11)–[#17](https://github.com/Raveu-lab/soroban-devkit-contract/issues/17)),
and this is the concrete difference it makes.
