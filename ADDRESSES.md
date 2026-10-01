# On-Chain Addresses

All soroban-devkit-contracts deployments are tracked here.
This file is the single source of truth for contract IDs across all networks.

---

## Testnet

Network passphrase: `Test SDF Network ; September 2015`
RPC endpoint: `https://soroban-testnet.stellar.org`
Explorer: [stellar.expert/explorer/testnet](https://stellar.expert/explorer/testnet)

| Contract | Contract ID | Explorer |
|----------|-------------|---------|
| `token` | `CCEQRERHTDQU5Q45JRBRWSFF6QS22TWRURAIOH27FK23ZPOB6YNXRHSX` | [View](https://stellar.expert/explorer/testnet/contract/CCEQRERHTDQU5Q45JRBRWSFF6QS22TWRURAIOH27FK23ZPOB6YNXRHSX) |
| `access-control` | `CCCKSTG5SDF6DJOXJASU2YP5EEWZLIGKBNUCWO665KWGFVPUFEVTIGUB` | [View](https://stellar.expert/explorer/testnet/contract/CCCKSTG5SDF6DJOXJASU2YP5EEWZLIGKBNUCWO665KWGFVPUFEVTIGUB) |
| `upgradeable` | `CCXNDNKLCSR2M2TMNHABOARH5Y4H6M27GVHLK7GOIRMRGCTQT32HWXXC` | [View](https://stellar.expert/explorer/testnet/contract/CCXNDNKLCSR2M2TMNHABOARH5Y4H6M27GVHLK7GOIRMRGCTQT32HWXXC) |
| `multisig` | `CCJZKGFUIHYBKZNGO4AW5HHYPJ4PLVYDJ2B342BLG3V5J23KI3CIAKUO` | [View](https://stellar.expert/explorer/testnet/contract/CCJZKGFUIHYBKZNGO4AW5HHYPJ4PLVYDJ2B342BLG3V5J23KI3CIAKUO) |
| `event-rich` | `CBHSJRE3FJD7DZPNHQF66LGBQXPYCR425LLXPMUIX2IVHK6EKGMCE26K` | [View](https://stellar.expert/explorer/testnet/contract/CBHSJRE3FJD7DZPNHQF66LGBQXPYCR425LLXPMUIX2IVHK6EKGMCE26K) |
| `escrow` | `CDY5RRA5666C6SQYVBAJW7ARZC2JKKBTGVGMTSANVDJLAC6LHO37KWSH` | [View](https://stellar.expert/explorer/testnet/contract/CDY5RRA5666C6SQYVBAJW7ARZC2JKKBTGVGMTSANVDJLAC6LHO37KWSH) |
| `vesting` | `CCKLABPVWH3EV7Y4YMVHFCBWXPJQJRDFNRB6COYB34Z6IEUUBLV6G4XX` | [View](https://stellar.expert/explorer/testnet/contract/CCKLABPVWH3EV7Y4YMVHFCBWXPJQJRDFNRB6COYB34Z6IEUUBLV6G4XX) |
| `oracle` | `CCU6IPNT2HQJOIBZ32GLWX7NRMKAGK22T7JCGPRYSZ6QMM57LHXX6UCK` | [View](https://stellar.expert/explorer/testnet/contract/CCU6IPNT2HQJOIBZ32GLWX7NRMKAGK22T7JCGPRYSZ6QMM57LHXX6UCK) |
| `dao-voting` | `CABVISPJ6JS2G637CNEDTMJ3OCYN4QCCZ7M2DYON7YLKOQ6QHQUEDTFE` | [View](https://stellar.expert/explorer/testnet/contract/CABVISPJ6JS2G637CNEDTMJ3OCYN4QCCZ7M2DYON7YLKOQ6QHQUEDTFE) |

### Deployer Account

| Key | Value |
|-----|-------|
| Public key | `GC3BJ52UL6CBAJBSZRX5DIPHXUW6OGJFVWMQKV3GLBMXHEZ66BWLXZN6` |
| Network | Testnet |
| Explorer | [View account](https://stellar.expert/explorer/testnet/account/GC3BJ52UL6CBAJBSZRX5DIPHXUW6OGJFVWMQKV3GLBMXHEZ66BWLXZN6) |

---

## Mainnet

Not yet deployed.

---

## Notes

- Contract IDs are deterministic per deployment — redeploying produces a new ID
- When a contract is redeployed, update both this file and `deployments.json`
- `token` was redeployed on 2026-09-11 to fix a critical bug: `mint`/`transfer`/`transfer_from`/`burn`/`clawback`/`approve` didn't reject non-positive amounts, so a negative amount could flip a transfer's arithmetic direction and let a caller mint themselves funds while draining the recipient. The old address `CB5YCY5CYLNO3PTH3OXQKKT6XFXTSNIOYSC5B65XE4ZZE6MVIWGD2LNH` should be treated as vulnerable and not used.
- `oracle` was redeployed on 2026-09-17: `set_price`/`get_price` now extend the price entry's persistent-storage TTL, so it no longer gets archived from disuse. The old address `CDX4U7QYTAGLEOOBUEJPTGONEV5HHXIK4BN7BHLOQAVRDGMWGWOX76SH` still works for read/write today but its prices won't have their TTL managed.
- `access-control` was redeployed on 2026-09-18, same TTL fix applied to `set_role`/`get_role`. The old address `CBFYOBMQF4Z625UVAG4C53KNJ7JVXNFRNBKMRQUCSY2YMORE5FI65QU6` still works but doesn't manage TTL on role-membership entries.
- `dao-voting` was redeployed on 2026-09-18, same TTL fix applied to proposals and vote records. The old address `CBZOOLSCJFAHHOKM575MBHXAPBI3IWXLJYZV5L3DNX5P4NAL2JNHNLTE` still works but doesn't manage TTL.
- `escrow` was redeployed on 2026-09-18, same TTL fix applied to escrow entries. The old address `CAHTJ7KOOIHITNV2HOCZXXGLS4ZXD64RZNOKQALLQ3ROIRBM6ZM27W2M` still works but doesn't manage TTL.
- `multisig` was redeployed on 2026-09-20, same TTL fix applied to proposals and approvals. The old address `CCJQWDZ7TDPVUJMBPXCMBMVZ4WTGXVJZZ4DZTAJ3BCG2KQJFDX5B7J4C` still works but doesn't manage TTL.
- `vesting` was redeployed on 2026-09-21, same TTL fix applied to vesting schedules — the last of the six persistent-storage contracts to get it. The old address `CDH42CTIXQ3OFEFHQTTBHR3IJ4HPEUNC2REM6DXH3K2QL23YKZY4K5W5` still works but doesn't manage TTL.
- `oracle` was redeployed *again* on 2026-09-21: `set_admin`/`get_admin` now also extend the whole contract instance's TTL, a more severe version of the persistent-entry fix from 2026-09-17 above — losing instance storage would make every function inoperable, not just one price lookup. The 2026-09-17 address `CD6LFXHN6HJ432OIUJNGJ7QO3Q6I3DNSGUGCAP2RL4GR27PTMTZZJQBM` manages persistent-entry TTL but not instance TTL.
- `access-control`, `token`, `multisig`, `upgradeable`, `dao-voting`, `escrow`, and `vesting` were all redeployed on 2026-10-01, carrying a batch of real fixes from `main` that had accumulated since the 2026-09-21 oracle redeploy above without ever reaching testnet:
  - `access-control`: `set_super_admin`/`get_super_admin` now extend the instance TTL (same severity as oracle's instance fix above — `SuperAdmin` is read on every `grant_role`/`revoke_role`/`set_role_admin` call).
  - `token`: balance entries and the whole instance (`Admin`/`Name`/`Symbol`/`Decimals`/`Clawback`) now get TTL-managed — this contract previously had *no* TTL management at all. Also: allowance `expiration_ledger` is now actually enforced (`get_allowance` returns 0 once it passes; previously the absolute ledger was passed straight into `extend_ttl`'s relative-count parameter, so an allowance silently outlived its stated expiry), and `approve(0)` now revokes an allowance per SEP-41 instead of being rejected by the blanket positive-amount guard.
  - `multisig`: `Signers`/`Threshold`/`Count` (instance storage) now get TTL-managed.
  - `upgradeable`: `Admin`/`Version` (instance storage) now get TTL-managed — the most severe version of this bug in the repo, since `upgrade()`/`migrate()` are the only way to ever fix a deployed contract and both are admin-gated.
  - `dao-voting`: `Count` (instance storage) now gets TTL-managed, and `propose()` now rejects `voting_duration == 0` (previously created a proposal with `deadline = now` that nobody could ever vote on).
  - `escrow`: `Count` (instance storage) now gets TTL-managed.
  - `vesting`: `Count` (instance storage) now gets TTL-managed.

  `access-control` was re-initialized with the deployer as `super_admin`; `upgradeable` with the deployer as `admin`; `token` with the deployer as `admin` (name "DevKit Token", symbol "DKT", decimals 7, clawback disabled); `multisig` with the deployer as the sole signer and threshold 1. `dao-voting`/`escrow`/`vesting` have no `initialize()` — their `Count` starts at 0 on first use. All four initializations and a handful of read calls were verified live against testnet before updating this file. The old addresses listed in the Notes above (and the ones these directly replace in the table) still work for reads but don't carry today's fixes.
- The `soroban-devkit-core` integration tests read `deployments.json` to resolve IDs at test time
- Testnet state resets periodically — check [status.stellar.org](https://status.stellar.org) if a contract ID stops responding
