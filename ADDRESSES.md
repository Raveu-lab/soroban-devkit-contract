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
| `token` | `CB5YCY5CYLNO3PTH3OXQKKT6XFXTSNIOYSC5B65XE4ZZE6MVIWGD2LNH` | [View](https://stellar.expert/explorer/testnet/contract/CB5YCY5CYLNO3PTH3OXQKKT6XFXTSNIOYSC5B65XE4ZZE6MVIWGD2LNH) |
| `access-control` | `CBFYOBMQF4Z625UVAG4C53KNJ7JVXNFRNBKMRQUCSY2YMORE5FI65QU6` | [View](https://stellar.expert/explorer/testnet/contract/CBFYOBMQF4Z625UVAG4C53KNJ7JVXNFRNBKMRQUCSY2YMORE5FI65QU6) |
| `upgradeable` | `CB2VSNSMBEOYZN2GJRZYTW6PYQAEMNFPCFJKW3YMQEDZKGXOLLKH3QQP` | [View](https://stellar.expert/explorer/testnet/contract/CB2VSNSMBEOYZN2GJRZYTW6PYQAEMNFPCFJKW3YMQEDZKGXOLLKH3QQP) |
| `multisig` | `CCJQWDZ7TDPVUJMBPXCMBMVZ4WTGXVJZZ4DZTAJ3BCG2KQJFDX5B7J4C` | [View](https://stellar.expert/explorer/testnet/contract/CCJQWDZ7TDPVUJMBPXCMBMVZ4WTGXVJZZ4DZTAJ3BCG2KQJFDX5B7J4C) |
| `event-rich` | `CBHSJRE3FJD7DZPNHQF66LGBQXPYCR425LLXPMUIX2IVHK6EKGMCE26K` | [View](https://stellar.expert/explorer/testnet/contract/CBHSJRE3FJD7DZPNHQF66LGBQXPYCR425LLXPMUIX2IVHK6EKGMCE26K) |
| `escrow` | `CAHTJ7KOOIHITNV2HOCZXXGLS4ZXD64RZNOKQALLQ3ROIRBM6ZM27W2M` | [View](https://stellar.expert/explorer/testnet/contract/CAHTJ7KOOIHITNV2HOCZXXGLS4ZXD64RZNOKQALLQ3ROIRBM6ZM27W2M) |
| `vesting` | `CDH42CTIXQ3OFEFHQTTBHR3IJ4HPEUNC2REM6DXH3K2QL23YKZY4K5W5` | [View](https://stellar.expert/explorer/testnet/contract/CDH42CTIXQ3OFEFHQTTBHR3IJ4HPEUNC2REM6DXH3K2QL23YKZY4K5W5) |

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
- The `soroban-devkit-core` integration tests read `deployments.json` to resolve IDs at test time
- Testnet state resets periodically — check [status.stellar.org](https://status.stellar.org) if a contract ID stops responding
