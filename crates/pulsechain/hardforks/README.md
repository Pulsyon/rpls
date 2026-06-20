# pulsechain-hardforks

Pure PulseChain fork constants and fork-selection predicates.

## Done

- Defines PulseChain mainnet/testnet-v4 chain IDs.
- Defines PrimordialPulse blocks for mainnet and testnet-v4.
- Defines PulseChain TTD offset and terminal total difficulty.
- Implements `BeforePrimordialPulse`, `PrimordialPulse`, and `AfterPrimordialPulse` phase helpers.
- Implements effective transaction chain ID behavior: Ethereum mainnet chain ID before PrimordialPulse, PulseChain chain ID at and after PrimordialPulse.
- Implements Shanghai activation behavior matching go-pulse: Ethereum mainnet Shanghai time before PrimordialPulse, PulseChain Shanghai time afterward.
- Implements compatibility predicates for the chain ID and Shanghai-time exceptions around PrimordialPulse.

## Not Here

- These helpers do not depend on Reth and do not validate headers directly.
- Consensus integration lives in `crates/pulsechain/node`.
- EVM state changes live in `crates/pulsechain/evm` and `crates/pulsechain/node`.

## Verification

- Unit tests cover phase boundaries, transaction chain ID transition, Shanghai timing, terminal total difficulty, and compatibility predicates.
- Cross-crate tests compare the exported constants with go-pulse-derived expectations.
