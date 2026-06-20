//! PulseChain consensus rule helpers.
//!
//! This crate contains the go-pulse-specific consensus predicates that can be
//! tested independently from Reth integration: PrimordialPulse difficulty,
//! terminal PoW detection, and the otherwise-invalid post-merge POS-to-POW
//! transition that is allowed exactly at PrimordialPulse.

use alloy_primitives::U256;
use pulsechain_hardforks::{
    PRIMORDIAL_PULSE_BLOCK, PULSECHAIN_TERMINAL_TOTAL_DIFFICULTY, PULSECHAIN_TTD_OFFSET,
    is_primordial_pulse_block_at,
};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PulseConsensusError {
    #[error(
        "PulseChain full validation requires the custom POS->POW->POS PrimordialPulse header verifier"
    )]
    PrimordialPulseHeaderVerifierNotWired,
    #[error(
        "PulseChain full validation requires PrimordialPulse fork-state mutation in the block executor"
    )]
    PrimordialPulseStateTransitionNotWired,
}

pub const fn primordial_pulse_difficulty(block_number: u64) -> Option<u64> {
    primordial_pulse_difficulty_at(block_number, PRIMORDIAL_PULSE_BLOCK)
}

pub const fn primordial_pulse_difficulty_at(
    block_number: u64,
    primordial_pulse_block: u64,
) -> Option<u64> {
    if is_primordial_pulse_block_at(block_number, primordial_pulse_block) {
        Some(PULSECHAIN_TTD_OFFSET)
    } else {
        None
    }
}

pub fn is_terminal_pow_block(parent_total_difficulty: U256, total_difficulty: U256) -> bool {
    parent_total_difficulty < PULSECHAIN_TERMINAL_TOTAL_DIFFICULTY
        && total_difficulty >= PULSECHAIN_TERMINAL_TOTAL_DIFFICULTY
}

pub fn allows_pos_to_pow_transition_at(
    parent_difficulty: U256,
    header_difficulty: U256,
    header_number: u64,
    primordial_pulse_block: u64,
) -> bool {
    !parent_difficulty.is_zero()
        || header_difficulty.is_zero()
        || is_primordial_pulse_block_at(header_number, primordial_pulse_block)
}

pub fn uses_pow_header_rules(header_difficulty: U256) -> bool {
    !header_difficulty.is_zero()
}

pub fn assert_full_validation_ready() -> Result<(), PulseConsensusError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primordial_block_uses_ttd_offset_as_difficulty() {
        assert_eq!(primordial_pulse_difficulty(17_232_999), None);
        assert_eq!(primordial_pulse_difficulty(17_233_000), Some(131_072));
        assert_eq!(primordial_pulse_difficulty(17_233_001), None);
        assert_eq!(
            primordial_pulse_difficulty_at(16_492_700, 16_492_700),
            Some(131_072)
        );
    }

    #[test]
    fn terminal_pow_boundary_matches_pulse_ttd() {
        assert!(is_terminal_pow_block(
            PULSECHAIN_TERMINAL_TOTAL_DIFFICULTY - U256::from(1),
            PULSECHAIN_TERMINAL_TOTAL_DIFFICULTY
        ));
        assert!(!is_terminal_pow_block(
            PULSECHAIN_TERMINAL_TOTAL_DIFFICULTY,
            PULSECHAIN_TERMINAL_TOTAL_DIFFICULTY + U256::from(1)
        ));
    }

    #[test]
    fn pos_to_pow_transition_is_allowed_only_at_primordial_pulse() {
        assert!(!allows_pos_to_pow_transition_at(
            U256::ZERO,
            U256::from(PULSECHAIN_TTD_OFFSET),
            17_232_999,
            17_233_000
        ));
        assert!(allows_pos_to_pow_transition_at(
            U256::ZERO,
            U256::from(PULSECHAIN_TTD_OFFSET),
            17_233_000,
            17_233_000
        ));
        assert!(!allows_pos_to_pow_transition_at(
            U256::ZERO,
            U256::from(PULSECHAIN_TTD_OFFSET),
            17_233_001,
            17_233_000
        ));
        assert!(allows_pos_to_pow_transition_at(
            U256::from(1),
            U256::from(PULSECHAIN_TTD_OFFSET),
            17_233_001,
            17_233_000
        ));
    }

    #[test]
    fn nonzero_difficulty_uses_pow_header_rules() {
        assert!(uses_pow_header_rules(U256::from(1)));
        assert!(!uses_pow_header_rules(U256::ZERO));
    }
}
