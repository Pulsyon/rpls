//! PulseChain EVM fork-state artifacts.
//!
//! This crate owns the go-pulse data used by the PrimordialPulse state
//! transition: sacrifice credit allocation artifacts, testnet treasury
//! metadata, deposit-contract bytecode, and deposit-contract storage entries.

pub mod deposit_contract;
pub mod sacrifice;
