//! PulseChain hardfork constants and fork-selection predicates.
//!
//! This crate intentionally contains pure, dependency-light rules copied from
//! `go-pulse` so higher layers can use the same behavior for chain specs,
//! transaction chain IDs, Shanghai activation, compatibility checks, and the
//! PrimordialPulse boundary.

use alloy_primitives::U256;

pub const ETHEREUM_MAINNET_CHAIN_ID: u64 = 1;
pub const PULSECHAIN_MAINNET_CHAIN_ID: u64 = 369;
pub const PULSECHAIN_TESTNET_V4_CHAIN_ID: u64 = 943;
pub const PRIMORDIAL_PULSE_BLOCK: u64 = 17_233_000;
pub const PULSECHAIN_TESTNET_V4_PRIMORDIAL_PULSE_BLOCK: u64 = 16_492_700;
pub const PULSECHAIN_TTD_OFFSET: u64 = 131_072;
pub const PULSECHAIN_SHANGHAI_TIMESTAMP: u64 = 1_683_786_515;
pub const PULSECHAIN_TESTNET_V4_SHANGHAI_TIMESTAMP: u64 = 1_682_700_369;
pub const ETHEREUM_MAINNET_LONDON_BLOCK: u64 = 12_965_000;
pub const ETHEREUM_MAINNET_SHANGHAI_TIMESTAMP: u64 = 1_681_338_455;

pub const PULSECHAIN_TERMINAL_TOTAL_DIFFICULTY: U256 =
    U256::from_limbs([0xd815d562d3d1a955, 0x0000000000000c70, 0, 0]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PulsePhase {
    BeforePrimordialPulse,
    PrimordialPulse,
    AfterPrimordialPulse,
}

pub const fn pulse_phase(block_number: u64) -> PulsePhase {
    pulse_phase_at(block_number, PRIMORDIAL_PULSE_BLOCK)
}

pub const fn pulse_phase_at(block_number: u64, primordial_pulse_block: u64) -> PulsePhase {
    if block_number < primordial_pulse_block {
        PulsePhase::BeforePrimordialPulse
    } else if block_number == primordial_pulse_block {
        PulsePhase::PrimordialPulse
    } else {
        PulsePhase::AfterPrimordialPulse
    }
}

pub const fn is_before_primordial_pulse(block_number: u64) -> bool {
    is_before_primordial_pulse_at(block_number, PRIMORDIAL_PULSE_BLOCK)
}

pub const fn is_before_primordial_pulse_at(block_number: u64, primordial_pulse_block: u64) -> bool {
    block_number < primordial_pulse_block
}

pub const fn is_primordial_pulse_block(block_number: u64) -> bool {
    is_primordial_pulse_block_at(block_number, PRIMORDIAL_PULSE_BLOCK)
}

pub const fn is_primordial_pulse_block_at(block_number: u64, primordial_pulse_block: u64) -> bool {
    block_number == primordial_pulse_block
}

pub const fn is_after_primordial_pulse(block_number: u64) -> bool {
    is_after_primordial_pulse_at(block_number, PRIMORDIAL_PULSE_BLOCK)
}

pub const fn is_after_primordial_pulse_at(block_number: u64, primordial_pulse_block: u64) -> bool {
    block_number > primordial_pulse_block
}

pub const fn effective_chain_id(block_number: u64) -> u64 {
    effective_chain_id_at(
        block_number,
        PRIMORDIAL_PULSE_BLOCK,
        PULSECHAIN_MAINNET_CHAIN_ID,
    )
}

pub const fn effective_chain_id_at(
    block_number: u64,
    primordial_pulse_block: u64,
    pulsechain_chain_id: u64,
) -> u64 {
    if is_before_primordial_pulse_at(block_number, primordial_pulse_block) {
        ETHEREUM_MAINNET_CHAIN_ID
    } else {
        pulsechain_chain_id
    }
}

pub const fn transaction_chain_id(block_number: u64) -> u64 {
    effective_chain_id(block_number)
}

pub const fn transaction_chain_id_at(
    block_number: u64,
    primordial_pulse_block: u64,
    pulsechain_chain_id: u64,
) -> u64 {
    effective_chain_id_at(block_number, primordial_pulse_block, pulsechain_chain_id)
}

pub const fn primordial_pulse_ahead(block_number: u64) -> bool {
    primordial_pulse_ahead_at(block_number, PRIMORDIAL_PULSE_BLOCK)
}

pub const fn primordial_pulse_ahead_at(block_number: u64, primordial_pulse_block: u64) -> bool {
    block_number < primordial_pulse_block
}

pub const fn allow_chain_id_mismatch_at(block_number: u64, primordial_pulse_block: u64) -> bool {
    primordial_pulse_ahead_at(block_number, primordial_pulse_block)
}

pub const fn allow_shanghai_time_mismatch_at(
    block_number: u64,
    primordial_pulse_block: u64,
) -> bool {
    block_number <= primordial_pulse_block
}

pub const fn is_shanghai_active(block_number: u64, timestamp: u64) -> bool {
    is_shanghai_active_at(
        block_number,
        timestamp,
        PRIMORDIAL_PULSE_BLOCK,
        PULSECHAIN_SHANGHAI_TIMESTAMP,
    )
}

pub const fn is_shanghai_active_at(
    block_number: u64,
    timestamp: u64,
    primordial_pulse_block: u64,
    pulsechain_shanghai_timestamp: u64,
) -> bool {
    if block_number < ETHEREUM_MAINNET_LONDON_BLOCK {
        return false;
    }

    if is_before_primordial_pulse_at(block_number, primordial_pulse_block) {
        timestamp >= ETHEREUM_MAINNET_SHANGHAI_TIMESTAMP
    } else {
        timestamp >= pulsechain_shanghai_timestamp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_boundaries_match_go_pulse() {
        assert_eq!(pulse_phase(17_232_999), PulsePhase::BeforePrimordialPulse);
        assert_eq!(pulse_phase(17_233_000), PulsePhase::PrimordialPulse);
        assert_eq!(pulse_phase(17_233_001), PulsePhase::AfterPrimordialPulse);
    }

    #[test]
    fn chain_id_transitions_at_primordial_pulse() {
        assert_eq!(effective_chain_id(17_232_999), 1);
        assert_eq!(effective_chain_id(17_233_000), 369);
        assert_eq!(effective_chain_id(17_233_001), 369);
        assert_eq!(transaction_chain_id(17_232_999), 1);
        assert_eq!(transaction_chain_id(17_233_000), 369);
    }

    #[test]
    fn shanghai_uses_ethereum_time_before_pulse_and_pulse_time_after() {
        assert!(!is_shanghai_active(
            ETHEREUM_MAINNET_LONDON_BLOCK - 1,
            ETHEREUM_MAINNET_SHANGHAI_TIMESTAMP
        ));
        assert!(is_shanghai_active(
            17_232_999,
            ETHEREUM_MAINNET_SHANGHAI_TIMESTAMP
        ));
        assert!(!is_shanghai_active(
            17_233_000,
            ETHEREUM_MAINNET_SHANGHAI_TIMESTAMP
        ));
        assert!(is_shanghai_active(
            17_233_000,
            PULSECHAIN_SHANGHAI_TIMESTAMP
        ));
    }

    #[test]
    fn terminal_total_difficulty_matches_go_pulse_decimal() {
        assert_eq!(
            PULSECHAIN_TERMINAL_TOTAL_DIFFICULTY.to_string(),
            "58750003716598352947541"
        );
    }

    #[test]
    fn testnet_v4_constants_match_go_pulse() {
        assert_eq!(PULSECHAIN_TESTNET_V4_CHAIN_ID, 943);
        assert_eq!(PULSECHAIN_TESTNET_V4_PRIMORDIAL_PULSE_BLOCK, 16_492_700);
        assert_eq!(PULSECHAIN_TESTNET_V4_SHANGHAI_TIMESTAMP, 1_682_700_369);
        assert!(!is_primordial_pulse_block_at(
            16_492_699,
            PULSECHAIN_TESTNET_V4_PRIMORDIAL_PULSE_BLOCK
        ));
        assert!(is_primordial_pulse_block_at(
            16_492_700,
            PULSECHAIN_TESTNET_V4_PRIMORDIAL_PULSE_BLOCK
        ));
        assert!(!is_primordial_pulse_block_at(
            16_492_701,
            PULSECHAIN_TESTNET_V4_PRIMORDIAL_PULSE_BLOCK
        ));
        assert_eq!(
            effective_chain_id_at(
                16_492_700,
                PULSECHAIN_TESTNET_V4_PRIMORDIAL_PULSE_BLOCK,
                PULSECHAIN_TESTNET_V4_CHAIN_ID
            ),
            943
        );
        assert!(is_shanghai_active_at(
            16_492_700,
            PULSECHAIN_TESTNET_V4_SHANGHAI_TIMESTAMP,
            PULSECHAIN_TESTNET_V4_PRIMORDIAL_PULSE_BLOCK,
            PULSECHAIN_TESTNET_V4_SHANGHAI_TIMESTAMP
        ));
    }

    #[test]
    fn compatibility_predicates_match_go_pulse_exceptions() {
        assert!(primordial_pulse_ahead(17_232_999));
        assert!(!primordial_pulse_ahead(17_233_000));

        assert!(allow_chain_id_mismatch_at(
            17_232_999,
            PRIMORDIAL_PULSE_BLOCK
        ));
        assert!(!allow_chain_id_mismatch_at(
            17_233_000,
            PRIMORDIAL_PULSE_BLOCK
        ));

        assert!(allow_shanghai_time_mismatch_at(
            17_232_999,
            PRIMORDIAL_PULSE_BLOCK
        ));
        assert!(allow_shanghai_time_mismatch_at(
            17_233_000,
            PRIMORDIAL_PULSE_BLOCK
        ));
        assert!(!allow_shanghai_time_mismatch_at(
            17_233_001,
            PRIMORDIAL_PULSE_BLOCK
        ));
    }
}
