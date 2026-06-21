# PulseChain Implementation Notes

Source of truth: local `go-pulse-repo` at commit `a224d91967a31c2c3080a8f75784d8de13c80b7b`.

Upstream execution pin: `ab2b11f40eed3623219c49022061a11a0b5e2c0c`.

## Verified Rules

- Mainnet chain ID is `369`; Ethereum historical fork block numbers are inherited before PulseChain-specific behavior. See `go-pulse-repo/params/pulse.go:24`.
- TTD offset is `131_072`, and Pulse terminal total difficulty is `58_750_003_716_598_352_947_541`. See `go-pulse-repo/params/pulse.go:15`.
- PrimordialPulse mainnet block is `17_233_000`. See `go-pulse-repo/params/pulse.go:43`.
- PulseChain Shanghai time is `1683786515`. See `go-pulse-repo/params/pulse.go:44`.
- Before PrimordialPulse, `IsShanghai` compares against Ethereum mainnet Shanghai time `1681338455`; after that, it uses the configured Pulse Shanghai time. See `go-pulse-repo/params/config.go:664`.
- `IsPrimordialPulseBlock` is exact equality, and `PrimordialPulseAhead` is a strict future-block test. See `go-pulse-repo/params/config.go:741`.
- Config compatibility intentionally allows a mismatching chain ID while PrimordialPulse is still ahead. See `go-pulse-repo/params/config.go:915`.
- Config compatibility intentionally allows a mismatching Shanghai time while on or before PrimordialPulse. See `go-pulse-repo/params/config.go:953`.
- Runtime rules use Ethereum mainnet chain ID `1` for pre-fork transaction signing/recovery, then Pulse chain ID after the fork. See `go-pulse-repo/params/config.go:1218` and `go-pulse-repo/core/types/transaction_signing.go:45`.
- PrimordialPulse fork-state transition is not transaction-driven. `PrimordialPulseFork` applies sacrifice credits and then replaces the deposit contract. See `go-pulse-repo/pulse/pulse.go:11`.
- Sacrifice allocations are embedded compressed artifacts. Mainnet uses `sacrifice_credits_mainnet.bin`; records are length-prefixed, first 20 bytes are address, remaining bytes are big-endian credit amount. See `go-pulse-repo/pulse/sacrifice_credits.go:15` and `go-pulse-repo/pulse/sacrifice_credits.go:37`.
- Mainnet allocation artifact verified in this workspace:
  - SHA-256: `6a8b1890c13c65b2b08e8eb4af7d4707ac73b7bd5e5332c23992381493ba79e1`
  - records: `292217`
  - total: `135089982762636446921514827401775`
  - known recipient `0x000000005dCEE11e13fb536Fa40d65450F53c5a8`: `64000000000000000000`
- Deposit replacement disables Ethereum deposit contract `0x00000000219ab540356cBB839Cbe05303d7705Fa` and deploys Pulse deposit contract `0x3693693693693693693693693693693693693693` with 31 storage entries. See `go-pulse-repo/pulse/deposit_contract.go:10` and `go-pulse-repo/pulse/deposit_contract.go:54`.
- PrimordialPulse block difficulty is the TTD offset. See `go-pulse-repo/consensus/ethash/consensus.go:314`.
- Ethash finalization applies `PrimordialPulseFork` exactly on the fork block, then normal block/uncle rewards. See `go-pulse-repo/consensus/ethash/consensus.go:511`.
- Beacon consensus allows the normally-invalid transition from parent difficulty `0` to header difficulty `>0` only at PrimordialPulse. See `go-pulse-repo/consensus/beacon/consensus.go:119`.
- Batched beacon header verification has explicit PulseChain cases for `POS[eth] => POW fork block[pls]` and `POS[eth] => POW fork block[pls] => POS[pls]`. See `go-pulse-repo/consensus/beacon/consensus.go:148`.
- Bootnodes and Pulse DNS discovery are defined in `go-pulse-repo/params/bootnodes.go:31` and `go-pulse-repo/params/bootnodes.go:118`.
- Pulse genesis uses Ethereum mainnet genesis state/header fields with Pulse config. See `go-pulse-repo/core/genesis.go:627`.

## Current Rust State

- `crates/pulsechain/hardforks` implements verified phase, transaction chain ID, Shanghai, TTD, compatibility, and fork predicates for mainnet and testnet-v4.
- `crates/pulsechain/chainspec` captures verified mainnet/testnet-v4 constants, inherited Ethereum fork schedule, genesis hash compatibility, optional treasury config, bootnodes, Pulse DNS discovery URLs, and rpls network bootstrap adapters.
- `crates/pulsechain/evm` embeds and validates the official mainnet and testnet-v4 sacrifice allocation artifacts and deposit contract artifacts.
- `crates/pulsechain/consensus` encodes the PrimordialPulse difficulty, terminal PoW, and POS-to-POW transition helper rules.
- `crates/pulsechain/node` wraps the upstream EVM configuration, applies the PrimordialPulse state mutation at the configured fork block, overrides the EVM transaction chain ID to Ethereum mainnet before PrimordialPulse and PulseChain at/after the fork, and installs a Pulse consensus wrapper over the upstream beacon consensus.
- `crates/pulsechain/rpc` contains Pulse mainnet and testnet-v4 identity helpers.
- `bin/rpls` wires the chainspec bootnode and DNS discovery adapters into rpls before networking starts, and defaults node storage to the OS app-data directory named `rpls`.

## Required Next Hooks

1. Confirm whether the network/fork-id calculation from the Pulse `ChainSpec` matches go-pulse peers. If not, customize the network/fork-id hook.
2. Build golden fixtures from `go-pulse` for blocks `17_232_999`, `17_233_000`, `17_233_001`, and at least 100 post-fork blocks, then compare headers, state roots, receipt roots, and transaction recovery.
3. Add trace/debug compatibility fixtures once block import parity is proven.
