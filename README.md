# pulse-reth

PulseChain execution-client work on top of pinned Reth crates.

This repository is structured as a separate Rust workspace on top of pinned Reth crates. It does not patch Reth core. The node swaps in PulseChain chain specs, executor state transition logic, and a Pulse-aware Reth consensus wrapper for the PrimordialPulse boundary.

## Build

```sh
cargo fmt
cargo test --all
cargo run -p pulse-reth
```

## Workspace

- `bin/pulse-reth`: Reth CLI entry point with Pulse chain parsing, executor/consensus installation, artifact validation, and default bootnode injection.
- `crates/pulsechain/chainspec`: Pulse mainnet/testnet-v4 constants, Ethereum historical forks, genesis hash, optional treasury config, and bootnodes.
- `crates/pulsechain/hardforks`: PrimordialPulse phase, transaction chain ID transition, Shanghai timing, and compatibility predicates.
- `crates/pulsechain/consensus`: Pulse difficulty, TTD, and header-transition helper rules.
- `crates/pulsechain/evm`: sacrifice allocation parser, embedded allocation artifacts, and deposit contract artifacts.
- `crates/pulsechain/node`: Reth node integration, Pulse consensus wrapper, EVM env chain ID override, and PrimordialPulse executor state mutation.
- `crates/pulsechain/rpc`: mainnet/testnet-v4 RPC identity helpers.
- `crates/pulsechain/tests`: cross-crate validation tests.

## Verified Pins

- Reth: `ab2b11f40eed3623219c49022061a11a0b5e2c0c`
- go-pulse source used for validation: `a224d91967a31c2c3080a8f75784d8de13c80b7b`

## Sync Modes

- Mode A, fast MVP trusted checkpoint: allowed by the node gate, but not implemented as a running Reth service yet.
- Mode B, full validation: installs the Pulse executor and Pulse consensus wrapper over Reth's Ethereum node components.

## Built-In Chains

- `pulsechain`: PulseChain mainnet, matching go-pulse `--pulsechain`.
- `pulsechain-testnet-v4`: PulseChain Testnet V4, matching go-pulse `--pulsechain-testnet-v4`. The parser also accepts `pulsechain-devnet` as an unlisted compatibility alias.
- `mainnet` and `dev`: delegated to Reth's Ethereum chain parser.

## Remaining Validation Work

- Golden block/state-root/receipt-root comparison against go-pulse around PrimordialPulse.
- Fork ID / peer handshake parity validation against go-pulse peers.
- Trace/debug compatibility tests.

See [docs/implementation-notes.md](docs/implementation-notes.md) for source-backed protocol notes.
