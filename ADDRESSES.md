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
| `multisig` | `CCJQWDZ7TDPVUJMBPXCMBMVZ4WTGXVJZZ4DZTAJ3BCG2KQJFDX5B7J4C` | [View](https://stellar.expert/explorer/testnet/contract/CCJQWDZ7TDPVUJMBPXCMBMVZ4WTGXVJZZ4DZTAJ3BCG2KQJFDX5B7J4C) |
| `event-rich` | `CBHSJRE3FJD7DZPNHQF66LGBQXPYCR425LLXPMUIX2IVHK6EKGMCE26K` | [View](https://stellar.expert/explorer/testnet/contract/CBHSJRE3FJD7DZPNHQF66LGBQXPYCR425LLXPMUIX2IVHK6EKGMCE26K) |
| `escrow` | `CAHTJ7KOOIHITNV2HOCZXXGLS4ZXD64RZNOKQALLQ3ROIRBM6ZM27W2M` | [View](https://stellar.expert/explorer/testnet/contract/CAHTJ7KOOIHITNV2HOCZXXGLS4ZXD64RZNOKQALLQ3ROIRBM6ZM27W2M) |
| `vesting` | `CDH42CTIXQ3OFEFHQTTBHR3IJ4HPEUNC2REM6DXH3K2QL23YKZY4K5W5` | [View](https://stellar.expert/explorer/testnet/contract/CDH42CTIXQ3OFEFHQTTBHR3IJ4HPEUNC2REM6DXH3K2QL23YKZY4K5W5) |
| `oracle` | `CD6LFXHN6HJ432OIUJNGJ7QO3Q6I3DNSGUGCAP2RL4GR27PTMTZZJQBM` | [View](https://stellar.expert/explorer/testnet/contract/CD6LFXHN6HJ432OIUJNGJ7QO3Q6I3DNSGUGCAP2RL4GR27PTMTZZJQBM) |
| `dao-voting` | `CBZOOLSCJFAHHOKM575MBHXAPBI3IWXLJYZV5L3DNX5P4NAL2JNHNLTE` | [View](https://stellar.expert/explorer/testnet/contract/CBZOOLSCJFAHHOKM575MBHXAPBI3IWXLJYZV5L3DNX5P4NAL2JNHNLTE) |

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
- The `soroban-devkit-core` integration tests read `deployments.json` to resolve IDs at test time
- Testnet state resets periodically — check [status.stellar.org](https://status.stellar.org) if a contract ID stops responding
