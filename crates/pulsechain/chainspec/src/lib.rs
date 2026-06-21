//! PulseChain chain specification constants.
//!
//! This crate mirrors the network constants from go-pulse for PulseChain
//! mainnet and Testnet V4: chain IDs, genesis compatibility, terminal total
//! difficulty, PrimordialPulse block numbers, Shanghai timestamps, inherited
//! Ethereum block forks, and default bootnodes.

pub mod bootnodes;

use alloy_primitives::{Address, B256, U256, address, b256, uint};
use pulsechain_hardforks::{
    PRIMORDIAL_PULSE_BLOCK, PULSECHAIN_MAINNET_CHAIN_ID, PULSECHAIN_SHANGHAI_TIMESTAMP,
    PULSECHAIN_TERMINAL_TOTAL_DIFFICULTY, PULSECHAIN_TESTNET_V4_CHAIN_ID,
    PULSECHAIN_TESTNET_V4_PRIMORDIAL_PULSE_BLOCK, PULSECHAIN_TESTNET_V4_SHANGHAI_TIMESTAMP,
};

pub use bootnodes::{
    PULSECHAIN_BOOTNODES, PULSECHAIN_DNS_DISCOVERY_URL, PULSECHAIN_TESTNET_V4_BOOTNODES,
    PULSECHAIN_TESTNET_V4_DNS_DISCOVERY_URL, PulseNetworkBuilder,
    apply_default_pulsechain_bootnodes, pulse_dns_discovery_url_for_chain_id,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockFork {
    pub name: &'static str,
    pub block: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreasuryCredit {
    pub address: Address,
    pub amount: U256,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PulseChainSpec {
    pub chain_id: u64,
    pub network_id: u64,
    pub genesis_hash: B256,
    pub terminal_total_difficulty: U256,
    pub primordial_pulse_block: u64,
    pub shanghai_timestamp: u64,
    pub treasury: Option<TreasuryCredit>,
    pub ethereum_block_forks: &'static [BlockFork],
    pub bootnodes: &'static [&'static str],
}

pub const ETHEREUM_MAINNET_GENESIS_HASH: B256 =
    b256!("d4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3");

pub const ETHEREUM_MAINNET_BLOCK_FORKS: &[BlockFork] = &[
    BlockFork {
        name: "Homestead",
        block: 1_150_000,
    },
    BlockFork {
        name: "DAO",
        block: 1_920_000,
    },
    BlockFork {
        name: "Tangerine",
        block: 2_463_000,
    },
    BlockFork {
        name: "SpuriousDragon",
        block: 2_675_000,
    },
    BlockFork {
        name: "Byzantium",
        block: 4_370_000,
    },
    BlockFork {
        name: "Constantinople",
        block: 7_280_000,
    },
    BlockFork {
        name: "Petersburg",
        block: 7_280_000,
    },
    BlockFork {
        name: "Istanbul",
        block: 9_069_000,
    },
    BlockFork {
        name: "MuirGlacier",
        block: 9_200_000,
    },
    BlockFork {
        name: "Berlin",
        block: 12_244_000,
    },
    BlockFork {
        name: "London",
        block: 12_965_000,
    },
    BlockFork {
        name: "ArrowGlacier",
        block: 13_773_000,
    },
    BlockFork {
        name: "GrayGlacier",
        block: 15_050_000,
    },
];

pub const PULSECHAIN_TESTNET_V4_TREASURY: TreasuryCredit = TreasuryCredit {
    address: address!("A592ED65885bcbCeb30442F4902a0D1Cf3AcB8fC"),
    amount: uint!(0x314DC6448D9338C15B0A00000000_U256),
};

pub const PULSECHAIN_MAINNET: PulseChainSpec = PulseChainSpec {
    chain_id: PULSECHAIN_MAINNET_CHAIN_ID,
    network_id: PULSECHAIN_MAINNET_CHAIN_ID,
    genesis_hash: ETHEREUM_MAINNET_GENESIS_HASH,
    terminal_total_difficulty: PULSECHAIN_TERMINAL_TOTAL_DIFFICULTY,
    primordial_pulse_block: PRIMORDIAL_PULSE_BLOCK,
    shanghai_timestamp: PULSECHAIN_SHANGHAI_TIMESTAMP,
    treasury: None,
    ethereum_block_forks: ETHEREUM_MAINNET_BLOCK_FORKS,
    bootnodes: PULSECHAIN_BOOTNODES,
};

pub const PULSECHAIN_TESTNET_V4: PulseChainSpec = PulseChainSpec {
    chain_id: PULSECHAIN_TESTNET_V4_CHAIN_ID,
    network_id: PULSECHAIN_TESTNET_V4_CHAIN_ID,
    genesis_hash: ETHEREUM_MAINNET_GENESIS_HASH,
    terminal_total_difficulty: PULSECHAIN_TERMINAL_TOTAL_DIFFICULTY,
    primordial_pulse_block: PULSECHAIN_TESTNET_V4_PRIMORDIAL_PULSE_BLOCK,
    shanghai_timestamp: PULSECHAIN_TESTNET_V4_SHANGHAI_TIMESTAMP,
    treasury: Some(PULSECHAIN_TESTNET_V4_TREASURY),
    ethereum_block_forks: ETHEREUM_MAINNET_BLOCK_FORKS,
    bootnodes: PULSECHAIN_TESTNET_V4_BOOTNODES,
};

pub const fn pulsechain_spec_for_chain_id(chain_id: u64) -> Option<PulseChainSpec> {
    match chain_id {
        PULSECHAIN_MAINNET_CHAIN_ID => Some(PULSECHAIN_MAINNET),
        PULSECHAIN_TESTNET_V4_CHAIN_ID => Some(PULSECHAIN_TESTNET_V4),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mainnet_spec_matches_go_pulse_constants() {
        assert_eq!(PULSECHAIN_MAINNET.chain_id, 369);
        assert_eq!(PULSECHAIN_MAINNET.network_id, 369);
        assert_eq!(PULSECHAIN_MAINNET.primordial_pulse_block, 17_233_000);
        assert_eq!(
            PULSECHAIN_MAINNET.terminal_total_difficulty.to_string(),
            "58750003716598352947541"
        );
        assert_eq!(PULSECHAIN_MAINNET.shanghai_timestamp, 1_683_786_515);
        assert_eq!(PULSECHAIN_MAINNET.treasury, None);
        assert_eq!(
            PULSECHAIN_MAINNET.genesis_hash,
            ETHEREUM_MAINNET_GENESIS_HASH
        );
    }

    #[test]
    fn testnet_v4_spec_matches_go_pulse_constants() {
        assert_eq!(PULSECHAIN_TESTNET_V4.chain_id, 943);
        assert_eq!(PULSECHAIN_TESTNET_V4.network_id, 943);
        assert_eq!(PULSECHAIN_TESTNET_V4.primordial_pulse_block, 16_492_700);
        assert_eq!(
            PULSECHAIN_TESTNET_V4.terminal_total_difficulty.to_string(),
            "58750003716598352947541"
        );
        assert_eq!(PULSECHAIN_TESTNET_V4.shanghai_timestamp, 1_682_700_369);
        assert_eq!(
            PULSECHAIN_TESTNET_V4.treasury,
            Some(PULSECHAIN_TESTNET_V4_TREASURY)
        );
        assert_eq!(
            PULSECHAIN_TESTNET_V4_TREASURY.amount.to_string(),
            "1000000000000000000000000000000000"
        );
        assert_eq!(
            PULSECHAIN_TESTNET_V4.genesis_hash,
            ETHEREUM_MAINNET_GENESIS_HASH
        );
        assert_eq!(PULSECHAIN_TESTNET_V4.bootnodes.len(), 8);
    }

    #[test]
    fn ethereum_history_forks_are_preserved_before_pulse() {
        assert!(
            PULSECHAIN_MAINNET
                .ethereum_block_forks
                .iter()
                .any(|fork| fork.name == "London" && fork.block == 12_965_000)
        );
        assert!(
            PULSECHAIN_MAINNET
                .ethereum_block_forks
                .iter()
                .any(|fork| fork.name == "GrayGlacier" && fork.block == 15_050_000)
        );
    }
}
