# rpls

PulseChain execution-client work on top of pinned upstream execution crates.

This repository is structured as a separate Rust workspace on top of pinned upstream execution crates. It does not patch upstream core. The node swaps in PulseChain chain specs, executor state transition logic, and a Pulse-aware consensus wrapper for the PrimordialPulse boundary.

## Build

```sh
cargo fmt
cargo test --all
cargo run -p rpls
```

## Workspace

- `bin/rpls`: rpls CLI entry point with Pulse chain parsing, executor/consensus/network installation, artifact validation, default storage/log paths, default bootnode injection, Pulse DNS discovery bootstrap, fork ID filtering, and optional minimal pruning defaults.
- `crates/pulsechain/chainspec`: Pulse mainnet/testnet-v4 constants, Ethereum historical forks, genesis hash, optional treasury config, bootnodes, Pulse DNS discovery URLs, and go-pulse-compatible fork ID filters.
- `crates/pulsechain/hardforks`: PrimordialPulse phase, transaction chain ID transition, Shanghai timing, and compatibility predicates.
- `crates/pulsechain/consensus`: Pulse difficulty, TTD, and header-transition helper rules.
- `crates/pulsechain/evm`: sacrifice allocation parser, embedded allocation artifacts, and deposit contract artifacts.
- `crates/pulsechain/node`: rpls node integration, Pulse consensus wrapper, EVM env chain ID override, and PrimordialPulse executor state mutation.
- `crates/pulsechain/tests`: cross-crate validation tests.

## Verified Pins

- Upstream execution pin: `ab2b11f40eed3623219c49022061a11a0b5e2c0c`
- go-pulse source used for validation: `a224d91967a31c2c3080a8f75784d8de13c80b7b`

## Built-In Chains

- `pulsechain`: PulseChain mainnet, matching go-pulse `--pulsechain`.
- `pulsechain-testnet-v4`: PulseChain Testnet V4, matching go-pulse `--pulsechain-testnet-v4`. The parser also accepts `pulsechain-devnet` as an unlisted compatibility alias.
- `mainnet` and `dev`: delegated to the upstream Ethereum chain parser.

## Remaining Validation Work

- Golden block/state-root/receipt-root comparison against go-pulse around PrimordialPulse.
- Live peer handshake compatibility validation against go-pulse peers.
- Trace/debug compatibility tests.

See [docs/implementation-notes.md](docs/implementation-notes.md) for source-backed protocol notes.
