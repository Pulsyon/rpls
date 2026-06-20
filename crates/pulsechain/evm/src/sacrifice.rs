use alloy_primitives::{Address, U256};
use pulsechain_hardforks::PULSECHAIN_TESTNET_V4_CHAIN_ID;
use sha2::{Digest, Sha256};
use thiserror::Error;

pub const MAINNET_ALLOCATION_BYTES: &[u8] =
    include_bytes!("../assets/sacrifice_credits_mainnet.bin");
pub const TESTNET_V4_ALLOCATION_BYTES: &[u8] =
    include_bytes!("../assets/sacrifice_credits_testnet_v4.bin");

pub const MAINNET_ALLOCATION: AllocationArtifact = AllocationArtifact {
    network: "mainnet",
    bytes: MAINNET_ALLOCATION_BYTES,
    expected_sha256: "6a8b1890c13c65b2b08e8eb4af7d4707ac73b7bd5e5332c23992381493ba79e1",
    expected_record_count: 292_217,
    expected_total: "135089982762636446921514827401775",
};

pub const TESTNET_V4_ALLOCATION: AllocationArtifact = AllocationArtifact {
    network: "testnet-v4",
    bytes: TESTNET_V4_ALLOCATION_BYTES,
    expected_sha256: "29b2e85aac54b721bdf1e61a7db287f5d3bf9f6cacf944982801a63fabf050cb",
    expected_record_count: 286_830,
    expected_total: "135089980879196446921514827401775",
};

#[derive(Debug, Clone, Copy)]
pub struct AllocationArtifact {
    pub network: &'static str,
    pub bytes: &'static [u8],
    pub expected_sha256: &'static str,
    pub expected_record_count: usize,
    pub expected_total: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllocationSummary {
    pub record_count: usize,
    pub total: U256,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SacrificeCredit {
    pub address: Address,
    pub amount: U256,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AllocationError {
    #[error("allocation artifact {network} has SHA-256 {actual}, expected {expected}")]
    ChecksumMismatch {
        network: &'static str,
        expected: &'static str,
        actual: String,
    },
    #[error("allocation artifact {network} has {actual} records, expected {expected}")]
    RecordCountMismatch {
        network: &'static str,
        expected: usize,
        actual: usize,
    },
    #[error("allocation artifact {network} total is {actual}, expected {expected}")]
    TotalMismatch {
        network: &'static str,
        expected: &'static str,
        actual: U256,
    },
    #[error("allocation artifact ended inside a record at byte {offset}")]
    TruncatedRecord { offset: usize },
    #[error(
        "allocation artifact contains a record shorter than the 20 byte address at byte {offset}"
    )]
    RecordTooShort { offset: usize },
}

impl AllocationArtifact {
    pub fn validate(&self) -> Result<AllocationSummary, AllocationError> {
        let summary = summarize(self.bytes)?;
        if summary.sha256 != self.expected_sha256 {
            return Err(AllocationError::ChecksumMismatch {
                network: self.network,
                expected: self.expected_sha256,
                actual: summary.sha256,
            });
        }
        if summary.record_count != self.expected_record_count {
            return Err(AllocationError::RecordCountMismatch {
                network: self.network,
                expected: self.expected_record_count,
                actual: summary.record_count,
            });
        }
        if summary.total.to_string() != self.expected_total {
            return Err(AllocationError::TotalMismatch {
                network: self.network,
                expected: self.expected_total,
                actual: summary.total,
            });
        }
        Ok(summary)
    }

    pub fn credits(&self) -> Result<impl Iterator<Item = SacrificeCredit> + '_, AllocationError> {
        self.validate()?;
        Ok(parse_credits(self.bytes)?.into_iter())
    }
}

pub fn allocation_for_chain_id(chain_id: u64) -> &'static AllocationArtifact {
    if chain_id == PULSECHAIN_TESTNET_V4_CHAIN_ID {
        &TESTNET_V4_ALLOCATION
    } else {
        &MAINNET_ALLOCATION
    }
}

pub fn summarize(bytes: &[u8]) -> Result<AllocationSummary, AllocationError> {
    let mut total = U256::ZERO;
    let mut record_count = 0usize;

    for credit in parse_credits(bytes)? {
        total = total.saturating_add(credit.amount);
        record_count += 1;
    }

    let sha256 = hex::encode(Sha256::digest(bytes));
    Ok(AllocationSummary {
        record_count,
        total,
        sha256,
    })
}

pub fn find_credit(bytes: &[u8], address: Address) -> Result<Option<U256>, AllocationError> {
    for credit in parse_credits(bytes)? {
        if credit.address == address {
            return Ok(Some(credit.amount));
        }
    }
    Ok(None)
}

pub fn parse_credits(bytes: &[u8]) -> Result<Vec<SacrificeCredit>, AllocationError> {
    let mut offset = 0usize;
    let mut credits = Vec::new();

    while offset < bytes.len() {
        let record_offset = offset;
        let byte_count = bytes[offset] as usize;
        offset += 1;

        let record_end = offset
            .checked_add(byte_count)
            .filter(|end| *end <= bytes.len())
            .ok_or(AllocationError::TruncatedRecord {
                offset: record_offset,
            })?;
        let record = &bytes[offset..record_end];
        offset = record_end;

        if record.len() < 20 {
            return Err(AllocationError::RecordTooShort {
                offset: record_offset,
            });
        }
        let address = Address::from_slice(&record[..20]);
        let amount = U256::from_be_slice(&record[20..]);
        credits.push(SacrificeCredit { address, amount });
    }

    Ok(credits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::address;

    #[test]
    fn mainnet_artifact_matches_go_pulse() {
        let summary = MAINNET_ALLOCATION.validate().unwrap();
        assert_eq!(summary.record_count, 292_217);
        assert_eq!(
            summary.sha256,
            "6a8b1890c13c65b2b08e8eb4af7d4707ac73b7bd5e5332c23992381493ba79e1"
        );
        assert_eq!(
            summary.total.to_string(),
            "135089982762636446921514827401775"
        );
    }

    #[test]
    fn testnet_v4_artifact_matches_go_pulse() {
        let summary = TESTNET_V4_ALLOCATION.validate().unwrap();
        assert_eq!(summary.record_count, 286_830);
        assert_eq!(
            summary.sha256,
            "29b2e85aac54b721bdf1e61a7db287f5d3bf9f6cacf944982801a63fabf050cb"
        );
        assert_eq!(
            summary.total.to_string(),
            "135089980879196446921514827401775"
        );
    }

    #[test]
    fn known_mainnet_recipient_matches_go_pulse_test() {
        let amount = find_credit(
            MAINNET_ALLOCATION.bytes,
            address!("000000005dcee11e13fb536fa40d65450f53c5a8"),
        )
        .unwrap();
        assert_eq!(amount.unwrap().to_string(), "64000000000000000000");
    }
}
