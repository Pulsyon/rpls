//! PulseChain storage adapters.
//!
//! This crate keeps PulseChain body storage behavior separate from node wiring.

use std::marker::PhantomData;

use pulsechain_hardforks::{
    PULSECHAIN_MAINNET_CHAIN_ID, PULSECHAIN_TESTNET_V4_CHAIN_ID, is_shanghai_active_at,
};
use reth_ethereum::{
    TransactionSigned,
    chainspec::{EthChainSpec, EthereumHardfork, EthereumHardforks},
    evm::revm::primitives::alloy_primitives::BlockNumber,
    primitives::{
        Block as BlockTrait, FullBlockHeader, FullNodePrimitives, FullSignedTx, Header,
        SignedTransaction,
    },
    provider::{
        ChainSpecProvider, DBProvider, ProviderResult,
        db::{
            cursor::DbCursorRO,
            tables,
            transaction::{DbTx, DbTxMut},
        },
        providers::{ChainStorage, DatabaseProvider, NodeTypesForProvider},
    },
    storage::{
        BlockBodyReader, BlockBodyWriter, ChainStorageReader, ChainStorageWriter, EthStorage,
        ReadBodyInput,
    },
};

/// Ethereum body storage with PulseChain's pre-PrimordialPulse Shanghai rule.
#[derive(Debug, Clone, Copy)]
pub struct PulseStorage<T = TransactionSigned, H = Header>(PhantomData<(T, H)>);

impl<T, H> Default for PulseStorage<T, H> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<Provider, T, H> BlockBodyWriter<Provider, alloy_consensus::BlockBody<T, H>>
    for PulseStorage<T, H>
where
    Provider: DBProvider<Tx: DbTxMut>,
    T: SignedTransaction,
    H: FullBlockHeader,
{
    fn write_block_bodies(
        &self,
        provider: &Provider,
        bodies: Vec<(u64, Option<alloy_consensus::BlockBody<T, H>>)>,
    ) -> ProviderResult<()> {
        EthStorage::<T, H>::default().write_block_bodies(provider, bodies)
    }

    fn remove_block_bodies_above(
        &self,
        provider: &Provider,
        block: BlockNumber,
    ) -> ProviderResult<()> {
        EthStorage::<T, H>::default().remove_block_bodies_above(provider, block)
    }
}

impl<Provider, T, H> BlockBodyReader<Provider> for PulseStorage<T, H>
where
    Provider: DBProvider + ChainSpecProvider<ChainSpec: EthChainSpec + EthereumHardforks>,
    T: SignedTransaction,
    H: FullBlockHeader,
{
    type Block = alloy_consensus::Block<T, H>;

    fn read_block_bodies(
        &self,
        provider: &Provider,
        inputs: Vec<ReadBodyInput<'_, Self::Block>>,
    ) -> ProviderResult<Vec<<Self::Block as BlockTrait>::Body>> {
        let chain_spec = provider.chain_spec();
        let mut withdrawals_cursor = provider
            .tx_ref()
            .cursor_read::<tables::BlockWithdrawals>()?;
        let mut bodies = Vec::with_capacity(inputs.len());

        for (header, transactions) in inputs {
            let withdrawals = if pulse_storage_withdrawals_active(
                &chain_spec,
                header.number(),
                header.timestamp(),
                header.withdrawals_root().is_some(),
            ) {
                withdrawals_cursor
                    .seek_exact(header.number())?
                    .map(|(_, w)| w.withdrawals)
                    .unwrap_or_default()
                    .into()
            } else {
                None
            };
            let ommers = if chain_spec.is_paris_active_at_block(header.number()) {
                Vec::new()
            } else {
                provider
                    .tx_ref()
                    .cursor_read::<tables::BlockOmmers<H>>()?
                    .seek_exact(header.number())?
                    .map(|(_, stored_ommers)| stored_ommers.ommers)
                    .unwrap_or_default()
            };

            bodies.push(alloy_consensus::BlockBody {
                transactions,
                ommers,
                withdrawals,
            });
        }

        Ok(bodies)
    }
}

impl<N, T, H> ChainStorage<N> for PulseStorage<T, H>
where
    T: FullSignedTx,
    H: FullBlockHeader,
    N: FullNodePrimitives<
            Block = alloy_consensus::Block<T, H>,
            BlockHeader = H,
            BlockBody = alloy_consensus::BlockBody<T, H>,
            SignedTx = T,
        >,
{
    fn reader<TX, Types>(&self) -> impl ChainStorageReader<DatabaseProvider<TX, Types>, N>
    where
        TX: DbTx + 'static,
        Types: NodeTypesForProvider<Primitives = N>,
    {
        self
    }

    fn writer<TX, Types>(&self) -> impl ChainStorageWriter<DatabaseProvider<TX, Types>, N>
    where
        TX: DbTxMut + DbTx + 'static,
        Types: NodeTypesForProvider<Primitives = N>,
    {
        self
    }
}

/// Returns true when a body read should reconstruct `Some(withdrawals)`.
pub fn pulse_storage_withdrawals_active<ChainSpec>(
    chain_spec: &ChainSpec,
    block_number: u64,
    timestamp: u64,
    header_has_withdrawals_root: bool,
) -> bool
where
    ChainSpec: EthChainSpec + EthereumHardforks,
{
    header_has_withdrawals_root || pulse_shanghai_active_at(chain_spec, block_number, timestamp)
}

fn pulse_shanghai_active_at<ChainSpec>(
    chain_spec: &ChainSpec,
    block_number: u64,
    timestamp: u64,
) -> bool
where
    ChainSpec: EthChainSpec + EthereumHardforks,
{
    if let Some((primordial_pulse_block, shanghai_timestamp)) = pulse_hardfork_context(chain_spec) {
        return is_shanghai_active_at(
            block_number,
            timestamp,
            primordial_pulse_block,
            shanghai_timestamp,
        );
    }

    chain_spec.is_shanghai_active_at_timestamp(timestamp)
}

fn pulse_hardfork_context<ChainSpec>(chain_spec: &ChainSpec) -> Option<(u64, u64)>
where
    ChainSpec: EthChainSpec + EthereumHardforks,
{
    match chain_spec.chain().id() {
        PULSECHAIN_MAINNET_CHAIN_ID | PULSECHAIN_TESTNET_V4_CHAIN_ID => {}
        _ => return None,
    }

    let primordial_pulse_block = chain_spec
        .ethereum_fork_activation(EthereumHardfork::Paris)
        .block_number()
        .unwrap_or(u64::MAX);
    let shanghai_timestamp = chain_spec
        .ethereum_fork_activation(EthereumHardfork::Shanghai)
        .as_timestamp()
        .unwrap_or(u64::MAX);

    Some((primordial_pulse_block, shanghai_timestamp))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulsechain_hardforks::{
        ETHEREUM_MAINNET_SHANGHAI_TIMESTAMP, PRIMORDIAL_PULSE_BLOCK, PULSECHAIN_SHANGHAI_TIMESTAMP,
    };
    use reth_ethereum::{
        chainspec::{Chain, ChainSpec, EthereumHardfork, ForkCondition, MAINNET},
        evm::revm::primitives::U256,
    };

    fn pulsechain_test_spec() -> ChainSpec {
        let mut spec = (**MAINNET).clone();
        spec.chain = Chain::from(PULSECHAIN_MAINNET_CHAIN_ID);
        spec.genesis.config.chain_id = PULSECHAIN_MAINNET_CHAIN_ID;
        spec.genesis.config.shanghai_time = Some(PULSECHAIN_SHANGHAI_TIMESTAMP);
        spec.genesis.config.terminal_total_difficulty = Some(U256::ZERO);
        spec.paris_block_and_final_difficulty = Some((PRIMORDIAL_PULSE_BLOCK, U256::ZERO));
        spec.hardforks.insert(
            EthereumHardfork::Paris,
            ForkCondition::TTD {
                activation_block_number: PRIMORDIAL_PULSE_BLOCK,
                total_difficulty: U256::ZERO,
                fork_block: Some(PRIMORDIAL_PULSE_BLOCK),
            },
        );
        spec.hardforks.insert(
            EthereumHardfork::Shanghai,
            ForkCondition::Timestamp(PULSECHAIN_SHANGHAI_TIMESTAMP),
        );
        spec
    }

    #[test]
    fn pulse_storage_uses_go_pulse_shanghai_rule_for_body_withdrawals() {
        let spec = pulsechain_test_spec();

        assert!(pulse_storage_withdrawals_active(
            &spec,
            PRIMORDIAL_PULSE_BLOCK - 1,
            ETHEREUM_MAINNET_SHANGHAI_TIMESTAMP,
            false
        ));
        assert!(!pulse_storage_withdrawals_active(
            &spec,
            PRIMORDIAL_PULSE_BLOCK - 1,
            ETHEREUM_MAINNET_SHANGHAI_TIMESTAMP - 1,
            false
        ));
        assert!(!pulse_storage_withdrawals_active(
            &spec,
            PRIMORDIAL_PULSE_BLOCK,
            ETHEREUM_MAINNET_SHANGHAI_TIMESTAMP,
            false
        ));
        assert!(pulse_storage_withdrawals_active(
            &spec,
            PRIMORDIAL_PULSE_BLOCK,
            PULSECHAIN_SHANGHAI_TIMESTAMP,
            false
        ));
        assert!(pulse_storage_withdrawals_active(
            &spec,
            PRIMORDIAL_PULSE_BLOCK,
            ETHEREUM_MAINNET_SHANGHAI_TIMESTAMP,
            true
        ));
    }
}
