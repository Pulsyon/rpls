//! PulseChain chain specification constants.
//!
//! This crate mirrors the network constants from go-pulse for PulseChain
//! mainnet and Testnet V4: chain IDs, genesis compatibility, terminal total
//! difficulty, PrimordialPulse block numbers, Shanghai timestamps, inherited
//! Ethereum block forks, and default bootnodes.

use alloy_primitives::{Address, B256, U256, address, b256, uint};
use pulsechain_hardforks::{
    PRIMORDIAL_PULSE_BLOCK, PULSECHAIN_MAINNET_CHAIN_ID, PULSECHAIN_SHANGHAI_TIMESTAMP,
    PULSECHAIN_TERMINAL_TOTAL_DIFFICULTY, PULSECHAIN_TESTNET_V4_CHAIN_ID,
    PULSECHAIN_TESTNET_V4_PRIMORDIAL_PULSE_BLOCK, PULSECHAIN_TESTNET_V4_SHANGHAI_TIMESTAMP,
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

pub const PULSECHAIN_BOOTNODES: &[&str] = &[
    "enode://bdb96e7ff6607414a4be8cdc8458861e9c22a25a0c254c7bb9c9c8423912e998b59e7ba012801538480eb78cec4d6766ab0b379d0b60356de84a7cdaec988c0b@5.9.124.244:30303",
    "enode://d69f8d28804ab34f7d5e20ac8bd4940412602787e2c37fc3600adc60dcd5d0a52e1fe1baccbefb6e278e1ee59fcb099c45db242edeb5e0a4547ff971218a0592@148.251.54.222:30303",
    "enode://1c9e030aa44b95b8239e1c97926787e12770c015b9dbf7a89b1178a5f4fab02462fde3489662119872dad5998e23440f78daae753d7a8f800900d871f08650a4@65.108.236.231:30303",
    "enode://95097eaeda4118297ad0ccb6160e1c9188af7560d25b4724052e0f004a33aaddb0e468103d622c77539b692fd1d9f3c156cb76c9ea402a86e3170d6ae60092e7@135.181.212.228:30303",
    "enode://da30ab2475cda64c2454b659a3ef045884c7d02b97d524d710020fdc2f37192b0aac7992bca8b7afd57474eb477e95567c8e0fe98003b779834f265304376c3c@135.181.229.180:30303",
    "enode://01d93871155cbe270bc60acfebc1aa859aacce002acaac39d633aa8e7c186ee26d19a41a50d8bc094c025a546ae5e1a38dc21ead75b4e7ddf4e917988d2f7c74@46.4.224.159:30303",
    "enode://96367e5e533cde68b6d3e7cc5308901fb1e4b1df51d2a0442df365fcfb8ba27a6e8bcde44b3629579da9e13d819f6059386a1e81ea4c5fd10d14599639c16214@46.4.224.160:30303",
    "enode://aece632270d66ff6bf9e9528e766b5829fb3b7812d48e4934c2768c45976b5f98559ce6d5763dc16d4351b15e776b55e2b983a0c367bdbe6279cfb3242f2587e@95.217.148.233:30303",
    "enode://95e1761e526d77fc732416a31c9c1795863b557ea02880101c01d14d13fdabb9312ce45c4f3037ad88002815f6826a36d86e42a1a7122f9188c64f53c4b68b1e@148.251.185.52:30303",
    "enode://0ad3bc059105b0cbc1d30a330f79b4fd4ef40f37782194daa6d3412a29a69e0190dd246fc019be9157a4bf095b584ab7874beba4c71c02156f602f32ff389f00@138.201.220.52:30303",
];

pub const PULSECHAIN_TESTNET_V4_BOOTNODES: &[&str] = &[
    "enode://3edb6b2b76ef50af30d3b02e098f00546f1a460ff1c82adad2639a57f6742c69516d24d760c0dd4555334adb01e6f3327f1a61056b3d89db4de10060248e8dea@65.21.204.190:30303",
    "enode://2b9af9cc9d09e2d2ef8cb3203f859e69b0175c1d7c41e14acf5162b239a773a966eea98a71999af9424ddb5b27a44759318869f8a4ba954483889aafdd6ea921@157.90.129.118:30303",
    "enode://2181f1b061713260eb806a7824d880088bbf3b47cf60fa7bc610439aedd20c213479df83a6eeaf42b41ad6f3eac6973ddc1d8d903a00094603ad667d5d87161f@37.27.57.158:30303",
    "enode://c1a8bc7b4a7fa66e3eed6732d966f98de6b4e4243353e9c2f4d632126b8da73022b3becf1582e940d3feeaf3243f63304356856053c76a7ea6cc5c50ad21d483@213.133.100.132:30303",
    "enode://7dce6f27d102ae4fac47042b0ed8fadfce0037a5384ae171017b8b6684efe57bb850359e00582a6f8099ac60b41e16efe46afb8772270e5e1cad3f7ed79d0e41@85.10.193.180:30303",
    "enode://94eedc89cebf735374bbae8078fff23744d7b118af6c0f33804d1ccf6cc8fdb9db7f55ccf81455034bc34b43f00fdc7ea5693b86d6c6098fc9603f689d0d1fca@95.217.150.118:30303",
    "enode://5999295986a65151d416dc09635da46896e8cd5e2f0dda0823ed3a0981dc50885407e5a990aa34e165c345e7bebaa837fcf9afaaa5e62d5add1fed6d4c9edbcc@95.217.148.234:30303",
    "enode://86831392545cec45fa30b578717684c4ffcf2e2bf050d4ecfdd5b9a6b2136e10d58f8606bacdd137e6ce68c1081442e39347ed391f166366f4951ab031156e93@138.201.193.233:30303",
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
