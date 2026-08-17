# pulsechain-storage

PulseChain block-body storage adapters for rpls.

## Done

- Provides `PulseStorage`, the node storage type used by `PulseNode`.
- Mirrors upstream Ethereum body storage for ommers and withdrawals writes.
- Preserves withdrawal bodies on read for PulseChain's Shanghai gap: before PrimordialPulse, blocks use Ethereum mainnet Shanghai time; after PrimordialPulse, they use PulseChain Shanghai time.
- Reconstructs `Some(withdrawals)` when a stored header already has a withdrawals root, even if the PulseChain chain spec timestamp has not reached Pulse Shanghai yet.
- Implements the reth provider `ChainStorage` adapter so the normal provider factory can use this storage type.

## Not Here

- Header validation for withdrawals roots lives in `crates/pulsechain/node`.
- EVM Shanghai activation and upstream finalization of gap withdrawal balance credits live in `crates/pulsechain/node`.
- Fork constants and pure Shanghai predicates live in `crates/pulsechain/hardforks`.
- Chain metadata and configured Pulse Shanghai timestamps live in `crates/pulsechain/chainspec`.

## Verification

- Unit tests cover the storage-side Shanghai activation rule around the gap between Ethereum Shanghai and Pulse Shanghai.
- Run with `cargo test -p pulsechain-storage --lib`.
