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
| `token` | `CATUGAK6QHMVJ5NUVXDHT3GO3K4KW224QMHZIPM5UHPGT65ED65S26EA` | [View](https://stellar.expert/explorer/testnet/contract/CATUGAK6QHMVJ5NUVXDHT3GO3K4KW224QMHZIPM5UHPGT65ED65S26EA) |
| `access-control` | `CD4JE7L4DEZD4XGMUF54ORJLQ2D7TKL5PBO2X535XTJOBA2DFLIWX5EM` | [View](https://stellar.expert/explorer/testnet/contract/CD4JE7L4DEZD4XGMUF54ORJLQ2D7TKL5PBO2X535XTJOBA2DFLIWX5EM) |
| `upgradeable` | `CB2VSNSMBEOYZN2GJRZYTW6PYQAEMNFPCFJKW3YMQEDZKGXOLLKH3QQP` | [View](https://stellar.expert/explorer/testnet/contract/CB2VSNSMBEOYZN2GJRZYTW6PYQAEMNFPCFJKW3YMQEDZKGXOLLKH3QQP) |
| `multisig` | `CCGKDANXX43YGJDI7LLPQ5BB4OEWG54TIWJWYJSNUMK3I4FDDSHYU2AI` | [View](https://stellar.expert/explorer/testnet/contract/CCGKDANXX43YGJDI7LLPQ5BB4OEWG54TIWJWYJSNUMK3I4FDDSHYU2AI) |
| `event-rich` | `CBHSJRE3FJD7DZPNHQF66LGBQXPYCR425LLXPMUIX2IVHK6EKGMCE26K` | [View](https://stellar.expert/explorer/testnet/contract/CBHSJRE3FJD7DZPNHQF66LGBQXPYCR425LLXPMUIX2IVHK6EKGMCE26K) |
| `escrow` | `CCOHN5UT565GGRQ63VB5DOUBXI56OCM7IB3RTIL2UIA264FECSNAOW4S` | [View](https://stellar.expert/explorer/testnet/contract/CCOHN5UT565GGRQ63VB5DOUBXI56OCM7IB3RTIL2UIA264FECSNAOW4S) |
| `vesting` | `CD2BICKSY5C2TZL5HWOWMH43CGBSVITYTDST4LVE653CID6ITKUVW7IT` | [View](https://stellar.expert/explorer/testnet/contract/CD2BICKSY5C2TZL5HWOWMH43CGBSVITYTDST4LVE653CID6ITKUVW7IT) |
| `oracle` | `CCU6IPNT2HQJOIBZ32GLWX7NRMKAGK22T7JCGPRYSZ6QMM57LHXX6UCK` | [View](https://stellar.expert/explorer/testnet/contract/CCU6IPNT2HQJOIBZ32GLWX7NRMKAGK22T7JCGPRYSZ6QMM57LHXX6UCK) |
| `dao-voting` | `CCU7KUN3VTWBQKQSXJXIWHTN6AG42BIUTP6H3EE656GYMEKBBLJJWLUB` | [View](https://stellar.expert/explorer/testnet/contract/CCU7KUN3VTWBQKQSXJXIWHTN6AG42BIUTP6H3EE656GYMEKBBLJJWLUB) |

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
- The `soroban-devkit-core` integration tests read `deployments.json` to resolve IDs at test time
- Testnet state resets periodically — check [status.stellar.org](https://status.stellar.org) if a contract ID stops responding
