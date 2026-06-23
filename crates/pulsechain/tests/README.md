# pulsechain-tests

Cross-crate go-pulse parity tests.

## Done

- Verifies PulseChain constants across crates.
- Verifies transaction chain ID and Shanghai transition behavior at PrimordialPulse.
- Verifies testnet-v4 constants.
- Verifies sacrifice allocation artifact metadata against go-pulse-derived expectations.

## Not Complete

- Golden fixtures for importing blocks `17_232_999`, `17_233_000`, `17_233_001`, and post-fork ranges are not implemented yet.
- State root, receipt root, and transaction recovery comparison against go-pulse are not implemented yet.
- Live peer handshake compatibility tests are not implemented yet.
- Trace/debug compatibility tests are not implemented yet.

## Verification

- Run with `cargo test -p pulsechain-tests`.
- The full workspace path is `cargo test --all`.
