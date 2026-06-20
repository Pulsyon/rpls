//! PulseChain RPC identity helpers.
//!
//! The binary still uses Reth's RPC server, but these helpers capture the
//! PulseChain network identities expected by `eth_chainId` and `net_version`
//! for tests and future RPC customization.

use pulsechain_hardforks::{PULSECHAIN_MAINNET_CHAIN_ID, PULSECHAIN_TESTNET_V4_CHAIN_ID};

pub fn eth_chain_id_hex() -> String {
    eth_chain_id_hex_for(PULSECHAIN_MAINNET_CHAIN_ID)
}

pub fn eth_chain_id_hex_for(chain_id: u64) -> String {
    format!("0x{chain_id:x}")
}

pub fn net_version() -> &'static str {
    "369"
}

pub fn net_version_for(chain_id: u64) -> String {
    chain_id.to_string()
}

pub fn testnet_v4_eth_chain_id_hex() -> String {
    eth_chain_id_hex_for(PULSECHAIN_TESTNET_V4_CHAIN_ID)
}

pub fn testnet_v4_net_version() -> String {
    net_version_for(PULSECHAIN_TESTNET_V4_CHAIN_ID)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pulse_rpc_identity_matches_mainnet() {
        assert_eq!(eth_chain_id_hex(), "0x171");
        assert_eq!(net_version(), "369");
    }

    #[test]
    fn pulse_rpc_identity_matches_testnet_v4() {
        assert_eq!(testnet_v4_eth_chain_id_hex(), "0x3af");
        assert_eq!(testnet_v4_net_version(), "943");
    }
}
