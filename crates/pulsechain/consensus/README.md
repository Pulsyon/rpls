# pulsechain-consensus

Dependency-light PulseChain consensus helper rules.

## Done

- Defines PrimordialPulse difficulty as the Pulse TTD offset exactly at the fork block.
- Implements terminal PoW boundary detection against PulseChain terminal total difficulty.
- Implements the go-pulse POS-to-POW transition predicate: a zero-difficulty parent may be followed by a nonzero-difficulty child only at PrimordialPulse.
- Exposes a readiness gate used by the node layer; it now reports ready because the Pulse executor and consensus wrapper are wired.

## Not Here

- Full header validation is implemented by `PulseBeaconConsensus` in `crates/pulsechain/node`.
- Full Ethash-style verification for the PrimordialPulse POW-shaped header is not complete yet.
- Golden block import fixtures are not in this crate.

## Verification

- Unit tests cover PrimordialPulse difficulty, terminal PoW detection, transition allowance, and nonzero-difficulty POW rule selection.
