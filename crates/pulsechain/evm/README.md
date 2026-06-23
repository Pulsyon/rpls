# pulsechain-evm

PulseChain fork artifacts and EVM-state data helpers.

## Done

- Embeds official go-pulse sacrifice allocation artifacts for mainnet and testnet-v4.
- Parses the compressed allocation record format used by go-pulse.
- Validates allocation SHA-256, record count, total amount, and known recipient balances.
- Embeds Pulse deposit contract bytecode, nil bytecode, and the replacement storage table.
- Defines Ethereum and Pulse deposit contract addresses.

## Not Here

- Applying sacrifice credits and deposit replacement into node state happens in `crates/pulsechain/node`.
- Transaction execution and EVM environment overrides are handled by `crates/pulsechain/node`.
- RPC exposure is handled by the `rpls` binary through the upstream node RPC stack.

## Verification

- Unit tests validate artifact hashes, allocation totals, known recipients, deposit bytecode, and storage table shape.
