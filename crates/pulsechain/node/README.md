# pulsechain-node

rpls node integration for PulseChain.

## Done

- Provides `PulseChainSpecParser` for `pulsechain`, `pulsechain-testnet-v4`, hidden `pulsechain-devnet`, and delegated Ethereum chains.
- Builds `ChainSpec` values from Pulse mainnet and testnet-v4 metadata.
- Installs `PulseExecutorBuilder`, wrapping the upstream Ethereum EVM configuration.
- Overrides EVM transaction chain ID to Ethereum mainnet before PrimordialPulse and PulseChain at/after PrimordialPulse.
- Applies PrimordialPulse state mutation exactly at the configured fork block.
- Applies sacrifice credits, testnet-v4 treasury credit, and deposit contract replacement.
- Installs `PulseBeaconConsensus`, wrapping the upstream beacon consensus.
- Allows the otherwise-invalid POS-to-POW header transition at PrimordialPulse and rejects it outside the fork block.
- Applies go-pulse Ethash-style POW header checks for nonzero-difficulty headers, including PrimordialPulse TTD-offset difficulty, future timestamp, gas, London base-fee presence/absence, DAO fork extra-data, and Shanghai/Cancun field rejection.
- Delegates normal body, pre-execution, post-execution, and POS header checks to upstream validation helpers.

## Not Complete

- Live peer handshake compatibility has not been proven yet.
- Golden block/state-root/receipt-root fixtures around PrimordialPulse are not built yet.

## Verification

- Unit tests cover chain parsing, chain spec identity, executor trigger boundaries, transaction chain ID overrides, PrimordialPulse state mutation, deposit replacement, sacrifice allocation, testnet-v4 treasury behavior, Pulse consensus boundary behavior, and POW header DAO/base-fee parity.
