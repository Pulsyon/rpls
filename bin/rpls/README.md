# rpls

rpls CLI entry point for running the PulseChain execution client.

## Done

- Uses the upstream `Cli` with `PulseChainSpecParser`, so visible built-in chains are `pulsechain`, `pulsechain-testnet-v4`, `mainnet`, and `dev`.
- Accepts `pulsechain-devnet` as an unlisted compatibility alias for testnet-v4.
- Installs the Pulse network, executor, and consensus builders into `EthereumNode::components()`.
- Validates the embedded mainnet sacrifice allocation at startup.
- Injects go-pulse default bootnodes for PulseChain mainnet and testnet-v4 when the user did not provide explicit bootnodes.
- Injects go-pulse Pulse DNS discovery trees for PulseChain mainnet and testnet-v4 when DNS discovery is enabled.
- Applies go-pulse-compatible fork ID filters for PulseChain mainnet and testnet-v4 networking.
- Applies go-pulse's `--rpc.txfeecap` default of `1000000` ether for PulseChain mainnet and testnet-v4 when the user did not provide an explicit cap.
- Uses an `rpls` OS app-data directory for default node storage when the user did not provide `--datadir`.
- Uses an `rpls` OS cache directory and `rpls.log` for default file logs when the user did not provide log-file settings.
- Provides `--rpls.minimal-pruning` to apply aggressive pruning defaults while preserving explicit pruning flags.
- Preserves user-provided bootnodes.

## Not Here

- Protocol constants live in `crates/pulsechain/chainspec` and `crates/pulsechain/hardforks`.
- PrimordialPulse state mutation and consensus wrapper logic live in `crates/pulsechain/node`.
- Live peer handshake compatibility has not been proven yet.

## Verification

- Unit tests cover CLI help chain visibility, default bootnode injection, Pulse DNS discovery links, default RPC transaction fee cap handling, default storage/log paths, and minimal pruning behavior.
- Workspace validation is `cargo test --all`.
