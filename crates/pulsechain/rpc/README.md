# pulsechain-rpc

PulseChain RPC identity helpers.

## Done

- Provides PulseChain mainnet network identity values.
- Provides PulseChain testnet-v4 network identity values.
- Keeps RPC identity logic independent from rpls node wiring.

## Not Here

- RPC module installation is not customized in this crate.
- Trace/debug compatibility behavior is not validated yet.
- Network fork ID and peer handshake logic are not handled here.

## Verification

- Unit tests cover mainnet and testnet-v4 RPC identity values.
