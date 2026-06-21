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
- Delegates normal body, pre-execution, post-execution, and POS header checks to upstream validation helpers.

## Not Complete

- Full Ethash-style verification for the PrimordialPulse POW-shaped header is only partially represented. go-pulse delegates this header to `ethone.VerifyHeader`, including difficulty calculation and DAO extra-data checks.
- Fork ID and peer handshake parity have not been proven or customized.
- Golden block/state-root/receipt-root fixtures around PrimordialPulse are not built yet.
- Fast trusted-checkpoint mode is represented as a mode, but not implemented as a running sync service.

## Verification

- Unit tests cover chain parsing, chain spec identity, executor trigger boundaries, transaction chain ID overrides, PrimordialPulse state mutation, deposit replacement, sacrifice allocation, testnet-v4 treasury behavior, and Pulse consensus boundary behavior.
