# pulse-reth

Reth CLI entry point for running the PulseChain execution client.

## Done

- Uses Reth's `Cli` with `PulseChainSpecParser`, so visible built-in chains are `pulsechain`, `pulsechain-testnet-v4`, `mainnet`, and `dev`.
- Accepts `pulsechain-devnet` as an unlisted compatibility alias for testnet-v4.
- Installs the Pulse executor and Pulse consensus builder into `EthereumNode::components()`.
- Validates the embedded mainnet sacrifice allocation at startup.
- Injects go-pulse default bootnodes for PulseChain mainnet and testnet-v4 when the user did not provide explicit bootnodes.
- Applies go-pulse's `--rpc.txfeecap` default of `1000000` ether for PulseChain mainnet and testnet-v4 when the user did not provide an explicit cap.
- Preserves user-provided bootnodes.

## Not Here

- Protocol constants live in `crates/pulsechain/chainspec` and `crates/pulsechain/hardforks`.
- PrimordialPulse state mutation and consensus wrapper logic live in `crates/pulsechain/node`.
- Fork ID and Pulse DNS discovery parity are not implemented yet.

## Verification

- Unit tests cover CLI help chain visibility, default bootnode injection, and default RPC transaction fee cap handling.
- Workspace validation is `cargo test --all`.
