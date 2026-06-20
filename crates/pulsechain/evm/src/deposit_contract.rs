use alloy_primitives::{Address, B256, address};
use sha2::{Digest, Sha256};

pub const ETHEREUM_DEPOSIT_CONTRACT: Address = address!("00000000219ab540356cBB839Cbe05303d7705Fa");
pub const PULSECHAIN_DEPOSIT_CONTRACT: Address =
    address!("3693693693693693693693693693693693693693");

pub const STORAGE_ENTRY_COUNT: usize = 31;

pub const NIL_CONTRACT_BYTECODE: &[u8] = include_bytes!("../assets/nil_contract.bin");
pub const PULSE_DEPOSIT_CONTRACT_BYTECODE: &[u8] =
    include_bytes!("../assets/pulse_deposit_contract.bin");

pub const NIL_CONTRACT_BYTECODE_SHA256: &str =
    "a88e927ab6af90f4d3f7a870ed9a82c29a80731fffb111a55b43f0b9131e7c9f";
pub const PULSE_DEPOSIT_CONTRACT_BYTECODE_SHA256: &str =
    "e67e69d553b7635490c9bccf202e2ade931b7f76851dcc635477919ebbfbe65b";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageEntry {
    pub key: &'static str,
    pub value: &'static str,
}

pub const DEPOSIT_CONTRACT_STORAGE: [StorageEntry; STORAGE_ENTRY_COUNT] = [
    StorageEntry {
        key: "0x22",
        value: "0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b",
    },
    StorageEntry {
        key: "0x23",
        value: "0xdb56114e00fdd4c1f85c892bf35ac9a89289aaecb1ebd0a96cde606a748b5d71",
    },
    StorageEntry {
        key: "0x24",
        value: "0xc78009fdf07fc56a11f122370658a353aaa542ed63e44c4bc15ff4cd105ab33c",
    },
    StorageEntry {
        key: "0x25",
        value: "0x536d98837f2dd165a55d5eeae91485954472d56f246df256bf3cae19352a123c",
    },
    StorageEntry {
        key: "0x26",
        value: "0x9efde052aa15429fae05bad4d0b1d7c64da64d03d7a1854a588c2cb8430c0d30",
    },
    StorageEntry {
        key: "0x27",
        value: "0xd88ddfeed400a8755596b21942c1497e114c302e6118290f91e6772976041fa1",
    },
    StorageEntry {
        key: "0x28",
        value: "0x87eb0ddba57e35f6d286673802a4af5975e22506c7cf4c64bb6be5ee11527f2c",
    },
    StorageEntry {
        key: "0x29",
        value: "0x26846476fd5fc54a5d43385167c95144f2643f533cc85bb9d16b782f8d7db193",
    },
    StorageEntry {
        key: "0x2a",
        value: "0x506d86582d252405b840018792cad2bf1259f1ef5aa5f887e13cb2f0094f51e1",
    },
    StorageEntry {
        key: "0x2b",
        value: "0xffff0ad7e659772f9534c195c815efc4014ef1e1daed4404c06385d11192e92b",
    },
    StorageEntry {
        key: "0x2c",
        value: "0x6cf04127db05441cd833107a52be852868890e4317e6a02ab47683aa75964220",
    },
    StorageEntry {
        key: "0x2d",
        value: "0xb7d05f875f140027ef5118a2247bbb84ce8f2f0f1123623085daf7960c329f5f",
    },
    StorageEntry {
        key: "0x2e",
        value: "0xdf6af5f5bbdb6be9ef8aa618e4bf8073960867171e29676f8b284dea6a08a85e",
    },
    StorageEntry {
        key: "0x2f",
        value: "0xb58d900f5e182e3c50ef74969ea16c7726c549757cc23523c369587da7293784",
    },
    StorageEntry {
        key: "0x30",
        value: "0xd49a7502ffcfb0340b1d7885688500ca308161a7f96b62df9d083b71fcc8f2bb",
    },
    StorageEntry {
        key: "0x31",
        value: "0x8fe6b1689256c0d385f42f5bbe2027a22c1996e110ba97c171d3e5948de92beb",
    },
    StorageEntry {
        key: "0x32",
        value: "0x8d0d63c39ebade8509e0ae3c9c3876fb5fa112be18f905ecacfecb92057603ab",
    },
    StorageEntry {
        key: "0x33",
        value: "0x95eec8b2e541cad4e91de38385f2e046619f54496c2382cb6cacd5b98c26f5a4",
    },
    StorageEntry {
        key: "0x34",
        value: "0xf893e908917775b62bff23294dbbe3a1cd8e6cc1c35b4801887b646a6f81f17f",
    },
    StorageEntry {
        key: "0x35",
        value: "0xcddba7b592e3133393c16194fac7431abf2f5485ed711db282183c819e08ebaa",
    },
    StorageEntry {
        key: "0x36",
        value: "0x8a8d7fe3af8caa085a7639a832001457dfb9128a8061142ad0335629ff23ff9c",
    },
    StorageEntry {
        key: "0x37",
        value: "0xfeb3c337d7a51a6fbf00b9e34c52e1c9195c969bd4e7a0bfd51d5c5bed9c1167",
    },
    StorageEntry {
        key: "0x38",
        value: "0xe71f0aa83cc32edfbefa9f4d3e0174ca85182eec9f3a09f6a6c0df6377a510d7",
    },
    StorageEntry {
        key: "0x39",
        value: "0x31206fa80a50bb6abe29085058f16212212a60eec8f049fecb92d8c8e0a84bc0",
    },
    StorageEntry {
        key: "0x3a",
        value: "0x21352bfecbeddde993839f614c3dac0a3ee37543f9b412b16199dc158e23b544",
    },
    StorageEntry {
        key: "0x3b",
        value: "0x619e312724bb6d7c3153ed9de791d764a366b389af13c58bf8a8d90481a46765",
    },
    StorageEntry {
        key: "0x3c",
        value: "0x7cdd2986268250628d0c10e385c58c6191e6fbe05191bcc04f133f2cea72c1c4",
    },
    StorageEntry {
        key: "0x3d",
        value: "0x848930bd7ba8cac54661072113fb278869e07bb8587f91392933374d017bcbe1",
    },
    StorageEntry {
        key: "0x3e",
        value: "0x8869ff2c22b28cc10510d9853292803328be4fb0e80495e8bb8d271f5b889636",
    },
    StorageEntry {
        key: "0x3f",
        value: "0xb5fe28e79f1b850f8658246ce9b6a1e7b49fc06db7143e8fe0b4f2b0c5523a5c",
    },
    StorageEntry {
        key: "0x40",
        value: "0x985e929f70af28d0bdd1a90a808f977f597c7c778c489e98d3bd8910d31ac0f7",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DepositContractReplacement {
    pub old_address: Address,
    pub new_address: Address,
    pub storage_entries: usize,
}

pub const REPLACEMENT: DepositContractReplacement = DepositContractReplacement {
    old_address: ETHEREUM_DEPOSIT_CONTRACT,
    new_address: PULSECHAIN_DEPOSIT_CONTRACT,
    storage_entries: STORAGE_ENTRY_COUNT,
};

pub fn go_pulse_hex_to_hash(hex_value: &str) -> B256 {
    let hex_value = hex_value
        .strip_prefix("0x")
        .or_else(|| hex_value.strip_prefix("0X"))
        .unwrap_or(hex_value);
    let decoded = hex::decode(hex_value).expect("go-pulse deposit storage hex is valid");
    assert!(
        decoded.len() <= 32,
        "go-pulse deposit storage hex exceeds 32 bytes"
    );

    let mut padded = [0u8; 32];
    padded[32 - decoded.len()..].copy_from_slice(&decoded);
    B256::from(padded)
}

pub fn bytecode_sha256(bytecode: &[u8]) -> String {
    hex::encode(Sha256::digest(bytecode))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytecode_matches_go_pulse() {
        assert_eq!(
            bytecode_sha256(NIL_CONTRACT_BYTECODE),
            NIL_CONTRACT_BYTECODE_SHA256
        );
        assert_eq!(
            bytecode_sha256(PULSE_DEPOSIT_CONTRACT_BYTECODE),
            PULSE_DEPOSIT_CONTRACT_BYTECODE_SHA256
        );
    }

    #[test]
    fn storage_table_matches_go_pulse_shape() {
        assert_eq!(DEPOSIT_CONTRACT_STORAGE.len(), STORAGE_ENTRY_COUNT);
        assert_eq!(
            go_pulse_hex_to_hash(DEPOSIT_CONTRACT_STORAGE[0].key),
            B256::from([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0x22
            ])
        );
        assert_eq!(DEPOSIT_CONTRACT_STORAGE.last().unwrap().key, "0x40");
    }
}
