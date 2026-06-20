use alloy_primitives::address;
use pulsechain_chainspec::{PULSECHAIN_MAINNET, PULSECHAIN_TESTNET_V4};
use pulsechain_evm::sacrifice::{MAINNET_ALLOCATION, find_credit};
use pulsechain_hardforks::{
    effective_chain_id, effective_chain_id_at, is_shanghai_active, is_shanghai_active_at,
};
use pulsechain_rpc::{eth_chain_id_hex, net_version};

#[test]
fn verified_constants_match_go_pulse() {
    assert_eq!(PULSECHAIN_MAINNET.chain_id, 369);
    assert_eq!(PULSECHAIN_MAINNET.primordial_pulse_block, 17_233_000);
    assert_eq!(
        PULSECHAIN_MAINNET.terminal_total_difficulty.to_string(),
        "58750003716598352947541"
    );
    assert_eq!(PULSECHAIN_MAINNET.shanghai_timestamp, 1_683_786_515);
    assert_eq!(eth_chain_id_hex(), "0x171");
    assert_eq!(net_version(), "369");
}

#[test]
fn testnet_v4_constants_match_go_pulse() {
    assert_eq!(PULSECHAIN_TESTNET_V4.chain_id, 943);
    assert_eq!(PULSECHAIN_TESTNET_V4.network_id, 943);
    assert_eq!(PULSECHAIN_TESTNET_V4.primordial_pulse_block, 16_492_700);
    assert_eq!(
        PULSECHAIN_TESTNET_V4.terminal_total_difficulty.to_string(),
        "58750003716598352947541"
    );
    assert_eq!(PULSECHAIN_TESTNET_V4.shanghai_timestamp, 1_682_700_369);
}

#[test]
fn transaction_chain_id_and_shanghai_rules_transition_at_primordial_pulse() {
    assert_eq!(effective_chain_id(17_232_999), 1);
    assert_eq!(effective_chain_id(17_233_000), 369);
    assert!(is_shanghai_active(17_232_999, 1_681_338_455));
    assert!(!is_shanghai_active(17_233_000, 1_681_338_455));
    assert!(is_shanghai_active(17_233_000, 1_683_786_515));

    assert_eq!(
        effective_chain_id_at(
            16_492_699,
            PULSECHAIN_TESTNET_V4.primordial_pulse_block,
            PULSECHAIN_TESTNET_V4.chain_id
        ),
        1
    );
    assert_eq!(
        effective_chain_id_at(
            16_492_700,
            PULSECHAIN_TESTNET_V4.primordial_pulse_block,
            PULSECHAIN_TESTNET_V4.chain_id
        ),
        943
    );
    assert!(!is_shanghai_active_at(
        16_492_700,
        1_681_338_455,
        PULSECHAIN_TESTNET_V4.primordial_pulse_block,
        PULSECHAIN_TESTNET_V4.shanghai_timestamp
    ));
    assert!(is_shanghai_active_at(
        16_492_700,
        PULSECHAIN_TESTNET_V4.shanghai_timestamp,
        PULSECHAIN_TESTNET_V4.primordial_pulse_block,
        PULSECHAIN_TESTNET_V4.shanghai_timestamp
    ));
}

#[test]
fn sacrifice_allocation_artifact_matches_go_pulse() {
    let summary = MAINNET_ALLOCATION.validate().unwrap();
    assert_eq!(summary.record_count, 292_217);
    assert_eq!(
        summary.total.to_string(),
        "135089982762636446921514827401775"
    );

    let amount = find_credit(
        MAINNET_ALLOCATION.bytes,
        address!("000000005dcee11e13fb536fa40d65450f53c5a8"),
    )
    .unwrap();
    assert_eq!(amount.unwrap().to_string(), "64000000000000000000");
}
