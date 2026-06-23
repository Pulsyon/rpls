# rpls

`rpls` is a Rust PulseChain execution client built on top of pinned upstream
execution-client crates. It follows the `reth` node architecture and adds the
PulseChain-specific chain configuration, networking, consensus boundary rules,
and PrimordialPulse state transition required to run PulseChain.

The project is not a fork that patches upstream `reth` in-place. It is a
separate Rust workspace that composes upstream execution crates with PulseChain
adapters. This keeps the PulseChain-specific logic isolated in this repository
while still allowing the project to track upstream execution-client work.

## go-pulse Compatibility

`rpls` uses `go-pulse` as the protocol source of truth for PulseChain-specific
behavior. The current compatibility work is validated against:

- `go-pulse`: [`pulsechaincom/go-pulse`](https://gitlab.com/pulsechaincom/go-pulse)
  at `a224d91967a31c2c3080a8f75784d8de13c80b7b`
- upstream execution crates: `ab2b11f40eed3623219c49022061a11a0b5e2c0c`

The Rust implementation mirrors go-pulse behavior for the parts that are
currently implemented, including:

- PulseChain mainnet and testnet-v4 chain identities.
- Ethereum historical fork schedule before PulseChain-specific behavior.
- PulseChain bootnodes and DNS discovery URLs.
- go-pulse-compatible fork ID filters.
- Ethereum chain ID before PrimordialPulse and PulseChain chain ID at and after
  PrimordialPulse.
- PulseChain Shanghai activation behavior.
- PrimordialPulse difficulty and POS-to-POW-to-POS header transition rules.
- PrimordialPulse state mutation: sacrifice credits, optional treasury credit,
  and deposit contract replacement.
- Ethash-style PoW header checks needed for PulseChain historical headers,
  including DAO fork extra-data behavior.

`rpls` is still under active parity validation. See
[Current Limitations](#current-limitations) before treating it as production
infrastructure.

## Workspace Layout

- `bin/rpls`: CLI entry point for running the node. It installs the PulseChain
  chain parser, executor, consensus wrapper, networking hooks, default
  bootnodes, default storage/log paths, and optional minimal pruning defaults.
- `crates/pulsechain/chainspec`: PulseChain constants, genesis identity,
  inherited Ethereum fork schedule, bootnodes, DNS discovery URLs, and fork ID
  filters.
- `crates/pulsechain/hardforks`: PrimordialPulse phase helpers, transaction
  chain ID transition, Shanghai timing, terminal total difficulty, and
  compatibility predicates.
- `crates/pulsechain/consensus`: Dependency-light consensus helper rules for
  PrimordialPulse difficulty and POS-to-POW transition behavior.
- `crates/pulsechain/evm`: Embedded PulseChain EVM artifacts, sacrifice
  allocation parsing, and deposit contract replacement data.
- `crates/pulsechain/node`: Node integration layer. This wraps upstream
  Ethereum node components with the PulseChain consensus wrapper, EVM
  configuration, transaction chain ID override, and PrimordialPulse executor
  mutation.
- `crates/pulsechain/tests`: Cross-crate go-pulse parity tests.
- `tools`: Nix flake and `cargo-deny` configuration.

## Requirements

For a normal Rust build:

- Rust toolchain from `toolchain.toml`.
- A C/C++ toolchain for native dependencies.
- OpenSSL and `pkg-config`.

For Nix-based development:

- Nix with flakes enabled.
- The flake in `tools/` uses `rust-overlay` and the workspace
  `toolchain.toml`.

## Build

Format and test the workspace:

```sh
cargo fmt
cargo test --all
```

Build the debug binary:

```sh
cargo build -p rpls
```

Build the release binary:

```sh
cargo build -p rpls --release
```

Run `cargo-deny` through Nix:

```sh
nix run ./tools#deny
```

## Nix Build And Run

The flake intentionally does not package the crate as a fixed Nix derivation.
Instead, it provides wrappers that run Cargo with the pinned Rust toolchain.
This preserves normal Cargo cache behavior.

Run the release node wrapper:

```sh
nix run ./tools -- node --chain pulsechain
```

Run the debug node wrapper:

```sh
nix run ./tools#debug -- node --chain pulsechain
```

Build release through the wrapper:

```sh
nix run ./tools#build-release
```

Build debug through the wrapper:

```sh
nix run ./tools#build-debug
```

If you run the flake from outside the repository, set `RPLS_WORKSPACE_ROOT`:

```sh
RPLS_WORKSPACE_ROOT=/path/to/reth-pulse nix run /path/to/reth-pulse/tools -- node --chain pulsechain
```

## Running A Node

Run PulseChain mainnet:

```sh
cargo run -p rpls --release -- node --chain pulsechain
```

Run PulseChain testnet-v4:

```sh
cargo run -p rpls --release -- node --chain pulsechain-testnet-v4
```

Run with an explicit datadir:

```sh
cargo run -p rpls --release -- node \
  --chain pulsechain \
  --datadir /path/to/rpls-data
```

Run with HTTP RPC enabled:

```sh
cargo run -p rpls --release -- node \
  --chain pulsechain \
  --http
```

Run with the rpls minimal-pruning preset:

```sh
cargo run -p rpls --release -- node \
  --chain pulsechain \
  --rpls.minimal-pruning
```

A typical local run looks like:

```sh
rpls node \
  --datadir /Volumes/ASTAR/rpls \
  --chain pulsechain \
  --rpls.minimal-pruning \
  --http
```

## Built-In Chains

- `pulsechain`: PulseChain mainnet, matching go-pulse `--pulsechain`.
- `pulsechain-testnet-v4`: PulseChain Testnet V4, matching go-pulse
  `--pulsechain-testnet-v4`.
- `pulsechain-devnet`: hidden compatibility alias for testnet-v4.
- `mainnet` and `dev`: delegated to the upstream Ethereum chain parser.

## Defaults

When running PulseChain chains, `rpls` applies PulseChain-specific defaults:

- Injects go-pulse bootnodes if the user did not provide explicit bootnodes.
- Enables Pulse DNS discovery for supported PulseChain networks.
- Applies go-pulse-compatible fork ID filters.
- Uses an `rpls` OS app-data directory by default when no `--datadir` is set.
- Uses an `rpls` OS cache/log directory and `rpls.log` by default when file logs
  are enabled.
- Applies the go-pulse RPC transaction fee cap default for PulseChain chains
  unless the user provides an explicit value.

## Current Limitations

The project is not yet claiming full production parity with go-pulse. The main
remaining validation work is:

- Golden block import fixtures around PrimordialPulse, including state root,
  receipt root, logs bloom, gas used, transaction sender recovery, and receipts.
- Live peer handshake compatibility validation against go-pulse peers.
- Trace/debug RPC compatibility fixtures.
- Fast trusted-checkpoint sync mode wiring. Snapshot download support is tracked
  separately from the full sync-mode flow.

Open GitHub issues track this remaining work.
