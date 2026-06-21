# pulsechain-chainspec

PulseChain chain metadata and static network constants.

## Done

- Defines PulseChain mainnet chain ID `369`.
- Defines PulseChain testnet-v4 chain ID `943`.
- Captures PrimordialPulse block, Shanghai timestamp, terminal total difficulty, and genesis hash for both supported Pulse networks.
- Defines optional chain treasury config; mainnet has none, testnet-v4 matches go-pulse.
- Preserves inherited Ethereum historical fork schedule before PulseChain-specific behavior.
- Provides go-pulse bootnode lists and Pulse DNS discovery URLs for mainnet and testnet-v4.
- Keeps the PulseChain parser-visible names separate from hidden compatibility aliases.

## Not Here

- Reth `ChainSpec` construction is done in `crates/pulsechain/node`.
- Fork predicates and transaction chain ID transition rules live in `crates/pulsechain/hardforks`.
- Bootnode and DNS discovery injection are done by `bin/pulse-reth`.

## Verification

- Unit tests check mainnet and testnet-v4 constants against go-pulse.
- Cross-crate tests verify exported constants with the embedded artifacts.
