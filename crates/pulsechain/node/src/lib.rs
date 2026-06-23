//! PulseChain rpls node integration.
//!
//! This crate adapts the upstream Ethereum node stack for PulseChain by supplying
//! chain parsing, PulseChain chain specs, EVM environment overrides, and the
//! PrimordialPulse block executor state mutation.

use std::sync::Arc;

use alloy_consensus::{EMPTY_OMMER_ROOT_HASH, Transaction};
use alloy_genesis::EthashConfig;
use pulsechain_chainspec::{
    PULSECHAIN_MAINNET, PULSECHAIN_TESTNET_V4, PulseChainSpec, TreasuryCredit,
    pulsechain_spec_for_chain_id,
};
use pulsechain_consensus::PulseConsensusError;
use pulsechain_evm::{
    deposit_contract::{
        DEPOSIT_CONTRACT_STORAGE, ETHEREUM_DEPOSIT_CONTRACT, NIL_CONTRACT_BYTECODE,
        PULSE_DEPOSIT_CONTRACT_BYTECODE, PULSECHAIN_DEPOSIT_CONTRACT, go_pulse_hex_to_hash,
    },
    sacrifice::allocation_for_chain_id,
};
use pulsechain_hardforks::{
    PRIMORDIAL_PULSE_BLOCK, PULSECHAIN_MAINNET_CHAIN_ID, PULSECHAIN_TESTNET_V4_CHAIN_ID,
    PULSECHAIN_TESTNET_V4_PRIMORDIAL_PULSE_BLOCK, PULSECHAIN_TTD_OFFSET, is_shanghai_active_at,
    transaction_chain_id_at,
};
use reth_cli::chainspec::ChainSpecParser;
use reth_ethereum::{
    Block, EthPrimitives, Receipt, TransactionSigned,
    chainspec::{
        Chain, ChainSpec, EthChainSpec, EthereumHardfork, EthereumHardforks, ForkCondition, MAINNET,
    },
    cli::chainspec::EthereumChainSpecParser,
    consensus::{
        Consensus, ConsensusError, EthBeaconConsensus, FullConsensus, HeaderValidator,
        TxGasLimitTooHighErr, validate_block_post_execution,
        validation::{
            MAX_RLP_BLOCK_SIZE, validate_4844_header_standalone, validate_against_parent_4844,
            validate_against_parent_eip1559_base_fee, validate_against_parent_gas_limit,
            validate_against_parent_hash_number, validate_against_parent_timestamp,
            validate_body_against_header, validate_cancun_gas, validate_header_base_fee,
            validate_header_extra_data, validate_header_gas, validate_shanghai_withdrawals,
        },
    },
    evm::{
        EthBlockAssembler, EthEvm, EthEvmConfig, RethReceiptBuilder,
        primitives::{
            Database, EthEvmFactory, Evm, EvmEnv, EvmEnvFor, ExecutionCtxFor, InspectorFor,
            NextBlockEnvAttributes, OnStateHook,
            block::{BlockExecutor, BlockExecutorFactory, BlockExecutorFor, ExecutableTx},
            eth::{EthBlockExecutionCtx, EthBlockExecutor},
            execute::{BlockExecutionError, InternalBlockExecutionError},
            precompiles::PrecompilesMap,
        },
        revm::{
            Database as _, DatabaseCommit,
            context::{CfgEnv, TxEnv, result::ResultAndState},
            db::State,
            primitives::{Address, B256, U256, hardfork::SpecId},
            state::{Account, Bytecode, EvmState, EvmStorage, EvmStorageSlot},
        },
    },
    node::{
        api::{ConfigureEngineEvm, ConfigureEvm, ExecutableTxIterator, FullNodeTypes, NodeTypes},
        builder::{
            BuilderContext,
            components::{ConsensusBuilder, ExecutorBuilder},
        },
    },
    primitives::{
        AlloyBlockHeader, Block as BlockTrait, BlockBody, BlockHeader, GotExpected, Header,
        NodePrimitives, RecoveredBlock, SealedBlock, SealedHeader,
        constants::MAX_TX_GAS_LIMIT_OSAKA, transaction::TxHashRef,
    },
    provider::BlockExecutionResult,
    rpc::types::engine::ExecutionData,
};

const ALLOWED_FUTURE_BLOCK_TIME_SECONDS: u64 = 15;
const DIFFICULTY_BOUND_DIVISOR: u64 = 2048;
const MINIMUM_DIFFICULTY: u64 = 131_072;
const FRONTIER_DURATION_LIMIT_SECONDS: u64 = 13;
const EXP_DIFFICULTY_PERIOD: u64 = 100_000;

#[derive(Debug, Clone, Default)]
pub struct PulseChainSpecParser;

impl ChainSpecParser for PulseChainSpecParser {
    type ChainSpec = ChainSpec;

    const SUPPORTED_CHAINS: &'static [&'static str] =
        &["pulsechain", "pulsechain-testnet-v4", "mainnet", "dev"];

    fn parse(chain: &str) -> eyre::Result<Arc<ChainSpec>> {
        match chain {
            "pulsechain" => Ok(pulsechain_rpls_chainspec()),
            "pulsechain-testnet-v4" | "pulsechain-devnet" => {
                Ok(pulsechain_testnet_v4_rpls_chainspec())
            }
            other => EthereumChainSpecParser::parse(other),
        }
    }
}

pub fn pulsechain_rpls_chainspec() -> Arc<ChainSpec> {
    pulsechain_rpls_chainspec_from(PULSECHAIN_MAINNET)
}

pub fn pulsechain_testnet_v4_rpls_chainspec() -> Arc<ChainSpec> {
    pulsechain_rpls_chainspec_from(PULSECHAIN_TESTNET_V4)
}

fn pulsechain_rpls_chainspec_from(pulse_spec: PulseChainSpec) -> Arc<ChainSpec> {
    let mut spec = (**MAINNET).clone();
    let genesis = &mut spec.genesis;

    genesis.nonce = 0x42;
    genesis.timestamp = 0;
    genesis.extra_data = "0x11bbe8db4e347b4e8c937c1c8370e4b5ed33adb3db69cbdb7a38e1e50b1b82fa"
        .parse()
        .expect("go-pulse genesis extra data is valid bytes");
    genesis.gas_limit = 5_000;
    genesis.difficulty = U256::from(17_179_869_184u64);
    genesis.mix_hash = B256::ZERO;
    genesis.coinbase = Address::ZERO;
    genesis.base_fee_per_gas = None;
    genesis.excess_blob_gas = None;
    genesis.blob_gas_used = None;
    genesis.number = None;

    genesis.config.chain_id = pulse_spec.chain_id;
    genesis.config.homestead_block = Some(1_150_000);
    genesis.config.dao_fork_block = Some(1_920_000);
    genesis.config.dao_fork_support = true;
    genesis.config.eip150_block = Some(2_463_000);
    genesis.config.eip155_block = Some(2_675_000);
    genesis.config.eip158_block = Some(2_675_000);
    genesis.config.byzantium_block = Some(4_370_000);
    genesis.config.constantinople_block = Some(7_280_000);
    genesis.config.petersburg_block = Some(7_280_000);
    genesis.config.istanbul_block = Some(9_069_000);
    genesis.config.muir_glacier_block = Some(9_200_000);
    genesis.config.berlin_block = Some(12_244_000);
    genesis.config.london_block = Some(12_965_000);
    genesis.config.arrow_glacier_block = Some(13_773_000);
    genesis.config.gray_glacier_block = Some(15_050_000);
    genesis.config.merge_netsplit_block = None;
    genesis.config.shanghai_time = Some(pulse_spec.shanghai_timestamp);
    genesis.config.cancun_time = None;
    genesis.config.prague_time = None;
    genesis.config.osaka_time = None;
    genesis.config.bpo1_time = None;
    genesis.config.bpo2_time = None;
    genesis.config.bpo3_time = None;
    genesis.config.bpo4_time = None;
    genesis.config.bpo5_time = None;
    genesis.config.deposit_contract_address = None;
    genesis.config.terminal_total_difficulty = Some(pulse_spec.terminal_total_difficulty);
    genesis.config.terminal_total_difficulty_passed = false;
    genesis.config.ethash = Some(EthashConfig {});
    genesis.config.clique = None;
    genesis.config.parlia = None;

    spec.chain = Chain::from(pulse_spec.chain_id);
    spec.deposit_contract = None;
    spec.paris_block_and_final_difficulty = Some((
        pulse_spec.primordial_pulse_block,
        pulse_spec.terminal_total_difficulty,
    ));
    spec.hardforks.insert(
        EthereumHardfork::Paris,
        ForkCondition::TTD {
            activation_block_number: pulse_spec.primordial_pulse_block,
            total_difficulty: pulse_spec.terminal_total_difficulty,
            fork_block: Some(pulse_spec.primordial_pulse_block),
        },
    );
    spec.hardforks.insert(
        EthereumHardfork::Shanghai,
        ForkCondition::Timestamp(pulse_spec.shanghai_timestamp),
    );
    for fork in [
        EthereumHardfork::Cancun,
        EthereumHardfork::Prague,
        EthereumHardfork::Osaka,
        EthereumHardfork::Bpo1,
        EthereumHardfork::Bpo2,
        EthereumHardfork::Bpo3,
        EthereumHardfork::Bpo4,
        EthereumHardfork::Bpo5,
    ] {
        spec.hardforks.remove(fork);
    }
    spec.blob_params.scheduled.clear();
    Arc::new(spec)
}

/// PulseChain consensus wrapper over the upstream Ethereum beacon consensus.
///
/// go-pulse keeps Ethereum beacon validation for normal post-Paris blocks, but
/// allows the PrimordialPulse block itself to cross from a zero-difficulty POS
/// parent back into a POW-style header with the Pulse TTD offset difficulty.
#[derive(Debug, Clone)]
pub struct PulseBeaconConsensus<ChainSpec> {
    inner: EthBeaconConsensus<ChainSpec>,
}

impl<ChainSpec> PulseBeaconConsensus<ChainSpec>
where
    ChainSpec: EthChainSpec + EthereumHardforks,
{
    pub fn new(chain_spec: Arc<ChainSpec>) -> Self {
        Self {
            inner: EthBeaconConsensus::new(chain_spec),
        }
    }

    pub const fn inner(&self) -> &EthBeaconConsensus<ChainSpec> {
        &self.inner
    }
}

impl<ChainSpec, N> FullConsensus<N> for PulseBeaconConsensus<ChainSpec>
where
    ChainSpec:
        Send + Sync + EthChainSpec<Header = N::BlockHeader> + EthereumHardforks + std::fmt::Debug,
    N: NodePrimitives,
{
    fn validate_block_post_execution(
        &self,
        block: &RecoveredBlock<N::Block>,
        result: &BlockExecutionResult<N::Receipt>,
    ) -> Result<(), ConsensusError> {
        validate_block_post_execution(
            block,
            self.inner.chain_spec(),
            &result.receipts,
            &result.requests,
        )
    }
}

impl<B, ChainSpec> Consensus<B> for PulseBeaconConsensus<ChainSpec>
where
    B: BlockTrait,
    ChainSpec: EthChainSpec<Header = B::Header> + EthereumHardforks + std::fmt::Debug + Send + Sync,
{
    type Error = ConsensusError;

    fn validate_body_against_header(
        &self,
        body: &B::Body,
        header: &SealedHeader<B::Header>,
    ) -> Result<(), Self::Error> {
        validate_body_against_header(body, header.header())
    }

    fn validate_block_pre_execution(&self, block: &SealedBlock<B>) -> Result<(), Self::Error> {
        validate_pulse_block_pre_execution(block, self.inner.chain_spec())
    }
}

impl<H, ChainSpec> HeaderValidator<H> for PulseBeaconConsensus<ChainSpec>
where
    H: BlockHeader,
    ChainSpec: EthChainSpec<Header = H> + EthereumHardforks + std::fmt::Debug + Send + Sync,
{
    fn validate_header(&self, header: &SealedHeader<H>) -> Result<(), ConsensusError> {
        if self.is_primordial_pulse_pow_header(header.header()) {
            return self.validate_primordial_pulse_pow_header(header.header());
        }

        if self.is_post_primordial_pulse_pow_header(header.header()) {
            return Err(ConsensusError::TheMergeDifficultyIsNotZero);
        }

        self.validate_header_standalone(header.header())
    }

    fn validate_header_against_parent(
        &self,
        header: &SealedHeader<H>,
        parent: &SealedHeader<H>,
    ) -> Result<(), ConsensusError> {
        validate_against_parent_hash_number(header.header(), parent)?;

        if parent.difficulty().is_zero()
            && !header.difficulty().is_zero()
            && !self.is_primordial_pulse_pow_header(header.header())
        {
            return Err(ConsensusError::TheMergeDifficultyIsNotZero);
        }

        validate_against_parent_timestamp(header.header(), parent.header())?;
        if !header.difficulty().is_zero() {
            self.validate_pow_difficulty_against_parent(header.header(), parent.header())?;
        }
        validate_against_parent_gas_limit(header, parent, self.inner.chain_spec())?;
        validate_against_parent_eip1559_base_fee(
            header.header(),
            parent.header(),
            self.inner.chain_spec(),
        )?;

        if let Some(blob_params) = self
            .inner
            .chain_spec()
            .blob_params_at_timestamp(header.timestamp())
        {
            validate_against_parent_4844(header.header(), parent.header(), blob_params)?;
        }

        Ok(())
    }
}

impl<ChainSpec> PulseBeaconConsensus<ChainSpec>
where
    ChainSpec: EthChainSpec + EthereumHardforks,
{
    fn validate_header_standalone<H: BlockHeader>(&self, header: &H) -> Result<(), ConsensusError>
    where
        ChainSpec: EthChainSpec<Header = H>,
    {
        if header.difficulty().is_zero() {
            return self.validate_pos_header_standalone(header);
        }

        self.validate_pow_header_standalone(header)
    }

    fn validate_pos_header_standalone<H: BlockHeader>(
        &self,
        header: &H,
    ) -> Result<(), ConsensusError>
    where
        ChainSpec: EthChainSpec<Header = H>,
    {
        let chain_spec = self.inner.chain_spec();

        if !header.nonce().is_some_and(|nonce| nonce.is_zero()) {
            return Err(ConsensusError::TheMergeNonceIsNotZero);
        }

        if header.ommers_hash() != EMPTY_OMMER_ROOT_HASH {
            return Err(ConsensusError::TheMergeOmmerRootIsNotEmpty);
        }

        validate_header_extra_data(header)?;
        validate_header_gas(header)?;
        validate_header_base_fee(header, chain_spec)?;

        let is_shanghai = pulse_shanghai_active_at(chain_spec, header.number(), header.timestamp());
        if is_shanghai && header.withdrawals_root().is_none() {
            return Err(ConsensusError::WithdrawalsRootMissing);
        } else if !is_shanghai && header.withdrawals_root().is_some() {
            return Err(ConsensusError::WithdrawalsRootUnexpected);
        }

        if let Some(blob_params) = chain_spec.blob_params_at_timestamp(header.timestamp()) {
            validate_4844_header_standalone(header, blob_params)?;
        } else if header.blob_gas_used().is_some() {
            return Err(ConsensusError::BlobGasUsedUnexpected);
        } else if header.excess_blob_gas().is_some() {
            return Err(ConsensusError::ExcessBlobGasUnexpected);
        } else if header.parent_beacon_block_root().is_some() {
            return Err(ConsensusError::ParentBeaconBlockRootUnexpected);
        }

        if chain_spec.is_prague_active_at_timestamp(header.timestamp()) {
            if header.requests_hash().is_none() {
                return Err(ConsensusError::RequestsHashMissing);
            }
        } else if header.requests_hash().is_some() {
            return Err(ConsensusError::RequestsHashUnexpected);
        }

        Ok(())
    }

    fn validate_pow_header_standalone<H: BlockHeader>(
        &self,
        header: &H,
    ) -> Result<(), ConsensusError>
    where
        ChainSpec: EthChainSpec<Header = H>,
    {
        let chain_spec = self.inner.chain_spec();
        let present_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if header.timestamp() > present_timestamp + ALLOWED_FUTURE_BLOCK_TIME_SECONDS {
            return Err(ConsensusError::TimestampIsInFuture {
                timestamp: header.timestamp(),
                present_timestamp,
            });
        }

        validate_header_extra_data(header)?;
        validate_header_gas(header)?;
        validate_header_base_fee(header, chain_spec)?;

        if pulse_shanghai_active_at(chain_spec, header.number(), header.timestamp())
            || header.withdrawals_root().is_some()
        {
            return Err(ConsensusError::WithdrawalsRootUnexpected);
        }

        if chain_spec.is_cancun_active_at_timestamp(header.timestamp()) {
            return Err(ConsensusError::BlobGasUsedUnexpected);
        }
        if header.blob_gas_used().is_some() {
            return Err(ConsensusError::BlobGasUsedUnexpected);
        }
        if header.excess_blob_gas().is_some() {
            return Err(ConsensusError::ExcessBlobGasUnexpected);
        }
        if header.parent_beacon_block_root().is_some() {
            return Err(ConsensusError::ParentBeaconBlockRootUnexpected);
        }
        if header.requests_hash().is_some() {
            return Err(ConsensusError::RequestsHashUnexpected);
        }

        Ok(())
    }

    fn validate_pow_difficulty_against_parent<H: BlockHeader>(
        &self,
        header: &H,
        parent: &H,
    ) -> Result<(), ConsensusError>
    where
        ChainSpec: EthChainSpec<Header = H>,
    {
        let expected =
            calculate_pow_difficulty(self.inner.chain_spec(), header.timestamp(), parent);
        if header.difficulty() != expected {
            return Err(ConsensusError::Other(
                PulseConsensusError::InvalidPowDifficulty {
                    got: header.difficulty(),
                    expected,
                }
                .to_string(),
            ));
        }

        Ok(())
    }

    fn is_primordial_pulse_pow_header<H: BlockHeader>(&self, header: &H) -> bool {
        primordial_pulse_block_for_chain_id(self.inner.chain_spec().chain().id())
            == Some(header.number())
            && header.difficulty() == U256::from(PULSECHAIN_TTD_OFFSET)
    }

    fn is_post_primordial_pulse_pow_header<H: BlockHeader>(&self, header: &H) -> bool {
        !header.difficulty().is_zero()
            && primordial_pulse_block_for_chain_id(self.inner.chain_spec().chain().id())
                .is_some_and(|primordial_pulse_block| header.number() >= primordial_pulse_block)
    }

    fn validate_primordial_pulse_pow_header<H: BlockHeader>(
        &self,
        header: &H,
    ) -> Result<(), ConsensusError>
    where
        ChainSpec: EthChainSpec<Header = H>,
    {
        self.validate_pow_header_standalone(header)
    }
}

#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct PulseConsensusBuilder;

impl<Node> ConsensusBuilder<Node> for PulseConsensusBuilder
where
    Node: FullNodeTypes<
        Types: NodeTypes<ChainSpec: EthChainSpec + EthereumHardforks, Primitives = EthPrimitives>,
    >,
{
    type Consensus = Arc<PulseBeaconConsensus<<Node::Types as NodeTypes>::ChainSpec>>;

    async fn build_consensus(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::Consensus> {
        Ok(Arc::new(PulseBeaconConsensus::new(ctx.chain_spec())))
    }
}

#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct PulseExecutorBuilder;

impl<Types, Node> ExecutorBuilder<Node> for PulseExecutorBuilder
where
    Types: NodeTypes<ChainSpec = ChainSpec, Primitives = EthPrimitives>,
    Node: FullNodeTypes<Types = Types>,
{
    type EVM = PulseEvmConfig;

    async fn build_evm(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::EVM> {
        Ok(PulseEvmConfig {
            inner: EthEvmConfig::new(ctx.chain_spec()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct PulseEvmConfig {
    inner: EthEvmConfig,
}

impl BlockExecutorFactory for PulseEvmConfig {
    type EvmFactory = EthEvmFactory;
    type ExecutionCtx<'a> = EthBlockExecutionCtx<'a>;
    type Transaction = TransactionSigned;
    type Receipt = Receipt;

    fn evm_factory(&self) -> &Self::EvmFactory {
        self.inner.evm_factory()
    }

    fn create_executor<'a, DB, I>(
        &'a self,
        evm: EthEvm<&'a mut State<DB>, I, PrecompilesMap>,
        ctx: EthBlockExecutionCtx<'a>,
    ) -> impl BlockExecutorFor<'a, Self, DB, I>
    where
        DB: Database + 'a,
        I: InspectorFor<Self, &'a mut State<DB>> + 'a,
    {
        let chain_id = self.inner.chain_spec().chain().id();
        let primordial_pulse_block =
            primordial_pulse_block_for_chain(chain_id, self.inner.chain_spec());
        let treasury = pulsechain_spec_for_chain_id(chain_id).and_then(|spec| spec.treasury);

        PulseBlockExecutor {
            inner: EthBlockExecutor::new(
                evm,
                ctx,
                self.inner.chain_spec(),
                self.inner.executor_factory.receipt_builder(),
            ),
            primordial_pulse_block,
            chain_id,
            treasury,
        }
    }
}

impl ConfigureEvm for PulseEvmConfig {
    type Primitives = <EthEvmConfig as ConfigureEvm>::Primitives;
    type Error = <EthEvmConfig as ConfigureEvm>::Error;
    type NextBlockEnvCtx = <EthEvmConfig as ConfigureEvm>::NextBlockEnvCtx;
    type BlockExecutorFactory = Self;
    type BlockAssembler = EthBlockAssembler<ChainSpec>;

    fn block_executor_factory(&self) -> &Self::BlockExecutorFactory {
        self
    }

    fn block_assembler(&self) -> &Self::BlockAssembler {
        self.inner.block_assembler()
    }

    fn evm_env(&self, header: &Header) -> Result<EvmEnv<SpecId>, Self::Error> {
        let mut env = self.inner.evm_env(header)?;
        self.apply_pulse_cfg_env(&mut env.cfg_env, header.number, header.timestamp);
        Ok(env)
    }

    fn next_evm_env(
        &self,
        parent: &Header,
        attributes: &NextBlockEnvAttributes,
    ) -> Result<EvmEnv<SpecId>, Self::Error> {
        let mut env = self.inner.next_evm_env(parent, attributes)?;
        self.apply_pulse_cfg_env(
            &mut env.cfg_env,
            parent.number.saturating_add(1),
            attributes.timestamp,
        );
        Ok(env)
    }

    fn context_for_block<'a>(
        &self,
        block: &'a SealedBlock<Block>,
    ) -> Result<EthBlockExecutionCtx<'a>, Self::Error> {
        self.inner.context_for_block(block)
    }

    fn context_for_next_block(
        &self,
        parent: &SealedHeader,
        attributes: Self::NextBlockEnvCtx,
    ) -> Result<EthBlockExecutionCtx<'_>, Self::Error> {
        self.inner.context_for_next_block(parent, attributes)
    }
}

impl ConfigureEngineEvm<ExecutionData> for PulseEvmConfig {
    fn evm_env_for_payload(&self, payload: &ExecutionData) -> Result<EvmEnvFor<Self>, Self::Error> {
        let mut env = self.inner.evm_env_for_payload(payload)?;
        self.apply_pulse_cfg_env(
            &mut env.cfg_env,
            payload.payload.block_number(),
            payload.payload.timestamp(),
        );
        Ok(env)
    }

    fn context_for_payload<'a>(
        &self,
        payload: &'a ExecutionData,
    ) -> Result<ExecutionCtxFor<'a, Self>, Self::Error> {
        self.inner.context_for_payload(payload)
    }

    fn tx_iterator_for_payload(
        &self,
        payload: &ExecutionData,
    ) -> Result<impl ExecutableTxIterator<Self>, Self::Error> {
        self.inner.tx_iterator_for_payload(payload)
    }
}

impl PulseEvmConfig {
    fn apply_pulse_cfg_env(&self, cfg_env: &mut CfgEnv<SpecId>, block_number: u64, timestamp: u64) {
        cfg_env.chain_id = self.transaction_chain_id(block_number);

        let Some((primordial_pulse_block, shanghai_timestamp)) = self.pulse_hardfork_context()
        else {
            return;
        };

        if is_shanghai_active_at(
            block_number,
            timestamp,
            primordial_pulse_block,
            shanghai_timestamp,
        ) && cfg_env.spec < SpecId::SHANGHAI
        {
            cfg_env.spec = SpecId::SHANGHAI;
        }
    }

    fn transaction_chain_id(&self, block_number: u64) -> u64 {
        let chain_id = self.inner.chain_spec().chain().id();
        let Some((primordial_pulse_block, _)) = self.pulse_hardfork_context() else {
            return chain_id;
        };

        transaction_chain_id_at(block_number, primordial_pulse_block, chain_id)
    }

    fn pulse_hardfork_context(&self) -> Option<(u64, u64)> {
        let chain_id = self.inner.chain_spec().chain().id();
        let primordial_pulse_block = match chain_id {
            PULSECHAIN_MAINNET_CHAIN_ID | PULSECHAIN_TESTNET_V4_CHAIN_ID => {
                self.inner.chain_spec().paris_block().unwrap_or(u64::MAX)
            }
            _ => return None,
        };
        let shanghai_timestamp = self
            .inner
            .chain_spec()
            .ethereum_fork_activation(EthereumHardfork::Shanghai)
            .as_timestamp()
            .unwrap_or(u64::MAX);

        Some((primordial_pulse_block, shanghai_timestamp))
    }
}

pub struct PulseBlockExecutor<'a, Evm> {
    inner: EthBlockExecutor<'a, Evm, &'a Arc<ChainSpec>, &'a RethReceiptBuilder>,
    primordial_pulse_block: Option<u64>,
    chain_id: u64,
    treasury: Option<TreasuryCredit>,
}

impl<'db, DB, E> BlockExecutor for PulseBlockExecutor<'_, E>
where
    DB: Database + 'db,
    E: Evm<DB = &'db mut State<DB>, Tx = TxEnv>,
{
    type Transaction = TransactionSigned;
    type Receipt = Receipt;
    type Evm = E;

    fn apply_pre_execution_changes(&mut self) -> Result<(), BlockExecutionError> {
        self.inner.apply_pre_execution_changes()
    }

    fn execute_transaction_without_commit(
        &mut self,
        tx: impl ExecutableTx<Self>,
    ) -> Result<ResultAndState<<Self::Evm as Evm>::HaltReason>, BlockExecutionError> {
        self.inner.execute_transaction_without_commit(tx)
    }

    fn commit_transaction(
        &mut self,
        output: ResultAndState<<Self::Evm as Evm>::HaltReason>,
        tx: impl ExecutableTx<Self>,
    ) -> Result<u64, BlockExecutionError> {
        self.inner.commit_transaction(output, tx)
    }

    fn finish(mut self) -> Result<(Self::Evm, BlockExecutionResult<Receipt>), BlockExecutionError> {
        let block_number: u64 = self.inner.evm().block().number.saturating_to();
        if should_apply_primordial_pulse(block_number, self.primordial_pulse_block) {
            apply_primordial_pulse_state(
                self.inner.evm_mut().db_mut(),
                self.treasury.as_ref(),
                self.chain_id,
            )?;
        }
        self.inner.finish()
    }

    fn set_state_hook(&mut self, hook: Option<Box<dyn OnStateHook>>) {
        self.inner.set_state_hook(hook)
    }

    fn evm_mut(&mut self) -> &mut Self::Evm {
        self.inner.evm_mut()
    }

    fn evm(&self) -> &Self::Evm {
        self.inner.evm()
    }
}

fn primordial_pulse_block_for_chain(chain_id: u64, chain_spec: &ChainSpec) -> Option<u64> {
    match chain_id {
        PULSECHAIN_MAINNET_CHAIN_ID | PULSECHAIN_TESTNET_V4_CHAIN_ID => chain_spec.paris_block(),
        _ => None,
    }
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

fn calculate_pow_difficulty<ChainSpec, H>(
    chain_spec: &ChainSpec,
    timestamp: u64,
    parent: &H,
) -> U256
where
    ChainSpec: EthChainSpec + EthereumHardforks,
    H: BlockHeader,
{
    let next_block = parent.number().saturating_add(1);
    if primordial_pulse_block_for_chain_id(chain_spec.chain().id()) == Some(next_block) {
        return U256::from(PULSECHAIN_TTD_OFFSET);
    }

    if chain_spec
        .ethereum_fork_activation(EthereumHardfork::GrayGlacier)
        .active_at_block(next_block)
    {
        return calculate_dynamic_pow_difficulty(timestamp, parent, 11_400_000);
    }
    if chain_spec
        .ethereum_fork_activation(EthereumHardfork::ArrowGlacier)
        .active_at_block(next_block)
    {
        return calculate_dynamic_pow_difficulty(timestamp, parent, 10_700_000);
    }
    if chain_spec
        .ethereum_fork_activation(EthereumHardfork::London)
        .active_at_block(next_block)
    {
        return calculate_dynamic_pow_difficulty(timestamp, parent, 9_700_000);
    }
    if chain_spec
        .ethereum_fork_activation(EthereumHardfork::MuirGlacier)
        .active_at_block(next_block)
    {
        return calculate_dynamic_pow_difficulty(timestamp, parent, 9_000_000);
    }
    if chain_spec
        .ethereum_fork_activation(EthereumHardfork::Constantinople)
        .active_at_block(next_block)
    {
        return calculate_dynamic_pow_difficulty(timestamp, parent, 5_000_000);
    }
    if chain_spec
        .ethereum_fork_activation(EthereumHardfork::Byzantium)
        .active_at_block(next_block)
    {
        return calculate_dynamic_pow_difficulty(timestamp, parent, 3_000_000);
    }
    if chain_spec
        .ethereum_fork_activation(EthereumHardfork::Homestead)
        .active_at_block(next_block)
    {
        return calculate_homestead_pow_difficulty(timestamp, parent);
    }

    calculate_frontier_pow_difficulty(timestamp, parent)
}

fn calculate_dynamic_pow_difficulty<H: BlockHeader>(
    timestamp: u64,
    parent: &H,
    bomb_delay: u64,
) -> U256 {
    let parent_difficulty = parent.difficulty();
    let adjustment = parent_difficulty / U256::from(DIFFICULTY_BOUND_DIVISOR);
    let elapsed = timestamp.saturating_sub(parent.timestamp()) / 9;
    let uncle_factor = if parent.ommers_hash() == EMPTY_OMMER_ROOT_HASH {
        1
    } else {
        2
    };
    let adjustment_factor = elapsed.abs_diff(uncle_factor).min(99);
    let adjustment = adjustment * U256::from(adjustment_factor);

    let mut difficulty = if elapsed >= uncle_factor {
        parent_difficulty.saturating_sub(adjustment)
    } else {
        parent_difficulty.saturating_add(adjustment)
    };
    difficulty = difficulty.max(U256::from(MINIMUM_DIFFICULTY));

    let bomb_delay_from_parent = bomb_delay.saturating_sub(1);
    if parent.number() >= bomb_delay_from_parent {
        let fake_block_number = parent.number() - bomb_delay_from_parent;
        let period_count = fake_block_number / EXP_DIFFICULTY_PERIOD;
        if period_count > 1 {
            difficulty = difficulty.saturating_add(U256::from(1) << (period_count - 2));
        }
    }

    difficulty
}

fn calculate_homestead_pow_difficulty<H: BlockHeader>(timestamp: u64, parent: &H) -> U256 {
    let parent_difficulty = parent.difficulty();
    let adjustment = parent_difficulty / U256::from(DIFFICULTY_BOUND_DIVISOR);
    let elapsed = timestamp.saturating_sub(parent.timestamp()) / 10;
    let adjustment_factor = elapsed.abs_diff(1).min(99);
    let adjustment = adjustment * U256::from(adjustment_factor);

    let mut difficulty = if elapsed >= 1 {
        parent_difficulty.saturating_sub(adjustment)
    } else {
        parent_difficulty.saturating_add(adjustment)
    };
    difficulty = difficulty.max(U256::from(MINIMUM_DIFFICULTY));

    let period_count = (parent.number() + 1) / EXP_DIFFICULTY_PERIOD;
    if period_count > 1 {
        difficulty = difficulty.saturating_add(U256::from(1) << (period_count - 2));
    }

    difficulty
}

fn calculate_frontier_pow_difficulty<H: BlockHeader>(timestamp: u64, parent: &H) -> U256 {
    let parent_difficulty = parent.difficulty();
    let adjustment = parent_difficulty / U256::from(DIFFICULTY_BOUND_DIVISOR);
    let mut difficulty =
        if timestamp.saturating_sub(parent.timestamp()) < FRONTIER_DURATION_LIMIT_SECONDS {
            parent_difficulty.saturating_add(adjustment)
        } else {
            parent_difficulty.saturating_sub(adjustment)
        };
    difficulty = difficulty.max(U256::from(MINIMUM_DIFFICULTY));

    let period_count = (parent.number() + 1) / EXP_DIFFICULTY_PERIOD;
    if period_count > 1 {
        difficulty = difficulty.saturating_add(U256::from(1) << (period_count - 2));
        difficulty = difficulty.max(U256::from(MINIMUM_DIFFICULTY));
    }

    difficulty
}

fn primordial_pulse_block_for_chain_id(chain_id: u64) -> Option<u64> {
    match chain_id {
        PULSECHAIN_MAINNET_CHAIN_ID => Some(PRIMORDIAL_PULSE_BLOCK),
        PULSECHAIN_TESTNET_V4_CHAIN_ID => Some(PULSECHAIN_TESTNET_V4_PRIMORDIAL_PULSE_BLOCK),
        _ => None,
    }
}

fn should_apply_primordial_pulse(block_number: u64, primordial_pulse_block: Option<u64>) -> bool {
    primordial_pulse_block == Some(block_number)
}

fn validate_pulse_block_pre_execution<B, ChainSpec>(
    block: &SealedBlock<B>,
    chain_spec: &ChainSpec,
) -> Result<(), ConsensusError>
where
    B: BlockTrait,
    ChainSpec: EthereumHardforks + EthChainSpec,
{
    validate_pulse_post_merge_hardfork_fields(block, chain_spec)?;

    if let Err(error) = block.ensure_transaction_root_valid() {
        return Err(ConsensusError::BodyTransactionRootDiff(error.into()));
    }

    if chain_spec.is_osaka_active_at_timestamp(block.timestamp()) {
        for tx in block.body().transactions() {
            if tx.gas_limit() > MAX_TX_GAS_LIMIT_OSAKA {
                return Err(TxGasLimitTooHighErr {
                    tx_hash: *tx.tx_hash(),
                    gas_limit: tx.gas_limit(),
                    max_allowed: MAX_TX_GAS_LIMIT_OSAKA,
                }
                .into());
            }
        }
    }

    Ok(())
}

fn validate_pulse_post_merge_hardfork_fields<B, ChainSpec>(
    block: &SealedBlock<B>,
    chain_spec: &ChainSpec,
) -> Result<(), ConsensusError>
where
    B: BlockTrait,
    ChainSpec: EthereumHardforks + EthChainSpec,
{
    let ommers_hash = block.body().calculate_ommers_root();
    if Some(block.ommers_hash()) != ommers_hash {
        return Err(ConsensusError::BodyOmmersHashDiff(
            GotExpected {
                got: ommers_hash.unwrap_or(EMPTY_OMMER_ROOT_HASH),
                expected: block.ommers_hash(),
            }
            .into(),
        ));
    }

    if pulse_shanghai_active_at(chain_spec, block.number(), block.timestamp()) {
        validate_shanghai_withdrawals(block)?;
    }

    if chain_spec.is_cancun_active_at_timestamp(block.timestamp()) {
        validate_cancun_gas(block)?;
    }

    if chain_spec.is_osaka_active_at_timestamp(block.timestamp())
        && block.rlp_length() > MAX_RLP_BLOCK_SIZE
    {
        return Err(ConsensusError::BlockTooLarge {
            rlp_length: block.rlp_length(),
            max_rlp_length: MAX_RLP_BLOCK_SIZE,
        });
    }

    Ok(())
}

fn apply_primordial_pulse_state<DB>(
    state: &mut State<DB>,
    treasury: Option<&TreasuryCredit>,
    chain_id: u64,
) -> Result<(), BlockExecutionError>
where
    DB: Database,
{
    let mut state_diff = EvmState::default();

    apply_sacrifice_credits(state, &mut state_diff, treasury, chain_id)?;
    replace_deposit_contract(state, &mut state_diff)?;

    state.commit(state_diff);
    Ok(())
}

fn apply_sacrifice_credits<DB>(
    state: &mut State<DB>,
    state_diff: &mut EvmState,
    treasury: Option<&TreasuryCredit>,
    chain_id: u64,
) -> Result<(), BlockExecutionError>
where
    DB: Database,
{
    if let Some(treasury) = treasury {
        add_balance(state, state_diff, treasury.address, treasury.amount)?;
    }

    for credit in allocation_for_chain_id(chain_id)
        .credits()
        .map_err(|err| internal_execution_error(format!("invalid sacrifice allocation: {err}")))?
    {
        add_balance(state, state_diff, credit.address, credit.amount)?;
    }

    Ok(())
}

fn replace_deposit_contract<DB>(
    state: &mut State<DB>,
    state_diff: &mut EvmState,
) -> Result<(), BlockExecutionError>
where
    DB: Database,
{
    let old_contract_exists = ensure_account(state, state_diff, ETHEREUM_DEPOSIT_CONTRACT)?;
    let old_contract = state_diff
        .get_mut(&ETHEREUM_DEPOSIT_CONTRACT)
        .expect("account was inserted above");
    old_contract.info.balance = U256::ZERO;
    old_contract
        .info
        .set_code(Bytecode::new_raw(NIL_CONTRACT_BYTECODE.to_vec().into()));
    old_contract.mark_touch();
    if old_contract_exists {
        old_contract.mark_selfdestruct();
    }

    ensure_account(state, state_diff, PULSECHAIN_DEPOSIT_CONTRACT)?;

    let mut storage = EvmStorage::default();
    for entry in DEPOSIT_CONTRACT_STORAGE {
        let key = go_pulse_hash_to_u256(entry.key);
        let value = go_pulse_hash_to_u256(entry.value);
        let original = state
            .storage(PULSECHAIN_DEPOSIT_CONTRACT, key)
            .map_err(|_| internal_execution_error("failed to load deposit contract storage"))?;
        storage.insert(
            key,
            EvmStorageSlot::new_changed(original, value, Default::default()),
        );
    }

    let pulse_contract = state_diff
        .get_mut(&PULSECHAIN_DEPOSIT_CONTRACT)
        .expect("account was inserted above");
    pulse_contract.info.balance = U256::ZERO;
    pulse_contract.info.nonce = 0;
    pulse_contract.info.set_code(Bytecode::new_raw(
        PULSE_DEPOSIT_CONTRACT_BYTECODE.to_vec().into(),
    ));
    pulse_contract.storage.extend(storage);
    pulse_contract.mark_touch();

    Ok(())
}

fn add_balance<DB>(
    state: &mut State<DB>,
    state_diff: &mut EvmState,
    address: Address,
    amount: U256,
) -> Result<(), BlockExecutionError>
where
    DB: Database,
{
    ensure_account(state, state_diff, address)?;
    let account = state_diff
        .get_mut(&address)
        .expect("account was inserted above");
    account.info.balance = account
        .info
        .balance
        .checked_add(amount)
        .expect("go-pulse sacrifice credit balance overflow");
    account.mark_touch();
    Ok(())
}

fn ensure_account<DB>(
    state: &mut State<DB>,
    state_diff: &mut EvmState,
    address: Address,
) -> Result<bool, BlockExecutionError>
where
    DB: Database,
{
    if state_diff.contains_key(&address) {
        return Ok(true);
    }

    let info = state
        .basic(address)
        .map_err(|_| internal_execution_error("failed to load account for PrimordialPulse"))?;
    let exists = info.is_some();
    state_diff.insert(address, Account::from(info.unwrap_or_default()));
    Ok(exists)
}

fn go_pulse_hash_to_u256(hex_value: &str) -> U256 {
    U256::from_be_slice(go_pulse_hex_to_hash(hex_value).as_slice())
}

fn internal_execution_error(message: impl Into<String>) -> BlockExecutionError {
    BlockExecutionError::Internal(InternalBlockExecutionError::Other(message.into().into()))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use pulsechain_chainspec::PULSECHAIN_MAINNET;
    use pulsechain_hardforks::ETHEREUM_MAINNET_SHANGHAI_TIMESTAMP;
    use reth_ethereum::evm::revm::{
        primitives::{B256, alloy_primitives},
        state::AccountInfo,
    };

    #[test]
    fn pulse_chainspec_does_not_inherit_ethereum_post_shanghai_forks() {
        let spec = pulsechain_rpls_chainspec();
        let timestamp_after_ethereum_osaka = 1_782_000_000;

        for fork in [
            EthereumHardfork::Cancun,
            EthereumHardfork::Prague,
            EthereumHardfork::Osaka,
            EthereumHardfork::Bpo1,
            EthereumHardfork::Bpo2,
            EthereumHardfork::Bpo3,
            EthereumHardfork::Bpo4,
            EthereumHardfork::Bpo5,
        ] {
            assert_eq!(spec.ethereum_fork_activation(fork), ForkCondition::Never);
            assert!(
                !spec.is_ethereum_fork_active_at_timestamp(fork, timestamp_after_ethereum_osaka)
            );
        }

        assert!(!spec.is_cancun_active_at_timestamp(timestamp_after_ethereum_osaka));
        assert!(!spec.is_prague_active_at_timestamp(timestamp_after_ethereum_osaka));
        assert!(!spec.is_osaka_active_at_timestamp(timestamp_after_ethereum_osaka));
        assert_eq!(
            spec.blob_params_at_timestamp(timestamp_after_ethereum_osaka),
            None
        );
        assert_eq!(spec.genesis.config.cancun_time, None);
        assert_eq!(spec.genesis.config.prague_time, None);
        assert_eq!(spec.genesis.config.osaka_time, None);
    }

    #[test]
    fn pulse_consensus_allows_primordial_pulse_pos_to_pow_header() {
        let consensus = PulseBeaconConsensus::new(pulsechain_rpls_chainspec());
        let parent_hash = B256::repeat_byte(0x11);
        let parent = SealedHeader::new(
            pulse_transition_header(
                PULSECHAIN_MAINNET.primordial_pulse_block - 1,
                PULSECHAIN_MAINNET.shanghai_timestamp - 24,
                B256::repeat_byte(0x01),
                U256::ZERO,
            ),
            parent_hash,
        );
        let header = SealedHeader::new(
            pulse_transition_header(
                PULSECHAIN_MAINNET.primordial_pulse_block,
                PULSECHAIN_MAINNET.shanghai_timestamp - 12,
                parent_hash,
                U256::from(PULSECHAIN_TTD_OFFSET),
            ),
            B256::repeat_byte(0x22),
        );

        consensus.validate_header(&header).unwrap();
        consensus
            .validate_header_against_parent(&header, &parent)
            .unwrap();
    }

    #[test]
    fn pulse_consensus_rejects_invalid_parent_dependent_pow_difficulty() {
        let consensus = PulseBeaconConsensus::new(pulsechain_rpls_chainspec());
        let parent_hash = B256::repeat_byte(0x23);
        let parent_header = pulse_transition_header(
            15_050_000 - 1,
            1_000,
            B256::repeat_byte(0x21),
            U256::from(1_000_000_000u64),
        );
        let parent = SealedHeader::new(parent_header, parent_hash);

        let mut header = pulse_transition_header(
            15_050_000,
            1_012,
            parent_hash,
            calculate_pow_difficulty(consensus.inner.chain_spec(), 1_012, parent.header())
                + U256::from(1),
        );
        header.withdrawals_root = None;
        let header = SealedHeader::new(header, B256::repeat_byte(0x24));

        let err = consensus
            .validate_header_against_parent(&header, &parent)
            .unwrap_err();
        assert!(
            err.to_string()
                .contains("invalid PulseChain PoW difficulty")
        );
    }

    #[test]
    fn pulse_consensus_rejects_late_pos_to_pow_header() {
        let consensus = PulseBeaconConsensus::new(pulsechain_rpls_chainspec());
        let parent_hash = B256::repeat_byte(0x33);
        let parent = SealedHeader::new(
            pulse_transition_header(
                PULSECHAIN_MAINNET.primordial_pulse_block,
                PULSECHAIN_MAINNET.shanghai_timestamp,
                B256::repeat_byte(0x01),
                U256::ZERO,
            ),
            parent_hash,
        );
        let header = SealedHeader::new(
            pulse_transition_header(
                PULSECHAIN_MAINNET.primordial_pulse_block + 1,
                PULSECHAIN_MAINNET.shanghai_timestamp + 12,
                parent_hash,
                U256::from(PULSECHAIN_TTD_OFFSET),
            ),
            B256::repeat_byte(0x44),
        );

        assert_eq!(
            consensus.validate_header(&header),
            Err(ConsensusError::TheMergeDifficultyIsNotZero)
        );
        assert_eq!(
            consensus.validate_header_against_parent(&header, &parent),
            Err(ConsensusError::TheMergeDifficultyIsNotZero)
        );
    }

    #[test]
    fn pulse_consensus_rejects_wrong_primordial_pulse_pow_difficulty() {
        let consensus = PulseBeaconConsensus::new(pulsechain_rpls_chainspec());
        let header = SealedHeader::new(
            pulse_transition_header(
                PULSECHAIN_MAINNET.primordial_pulse_block,
                PULSECHAIN_MAINNET.shanghai_timestamp,
                B256::repeat_byte(0x45),
                U256::from(PULSECHAIN_TTD_OFFSET + 1),
            ),
            B256::repeat_byte(0x46),
        );

        assert_eq!(
            consensus.validate_header(&header),
            Err(ConsensusError::TheMergeDifficultyIsNotZero)
        );
    }

    #[test]
    fn pulse_consensus_rejects_shanghai_active_primordial_pulse_pow_header() {
        let consensus = PulseBeaconConsensus::new(pulsechain_rpls_chainspec());
        let header = SealedHeader::new(
            pulse_transition_header(
                PULSECHAIN_MAINNET.primordial_pulse_block,
                PULSECHAIN_MAINNET.shanghai_timestamp,
                B256::repeat_byte(0x47),
                U256::from(PULSECHAIN_TTD_OFFSET),
            ),
            B256::repeat_byte(0x48),
        );

        assert_eq!(
            consensus.validate_header(&header),
            Err(ConsensusError::WithdrawalsRootUnexpected)
        );
    }

    #[test]
    fn pulse_consensus_rejects_withdrawals_root_on_primordial_pulse_pow_header() {
        let consensus = PulseBeaconConsensus::new(pulsechain_rpls_chainspec());
        let mut header = pulse_transition_header(
            PULSECHAIN_MAINNET.primordial_pulse_block,
            PULSECHAIN_MAINNET.shanghai_timestamp - 12,
            B256::repeat_byte(0x49),
            U256::from(PULSECHAIN_TTD_OFFSET),
        );
        header.withdrawals_root = Some(B256::ZERO);

        assert_eq!(
            consensus.validate_header(&SealedHeader::new(header, B256::repeat_byte(0x4a))),
            Err(ConsensusError::WithdrawalsRootUnexpected)
        );
    }

    #[test]
    fn pulse_consensus_uses_go_pulse_shanghai_rule_for_header_withdrawals() {
        let consensus = PulseBeaconConsensus::new(pulsechain_rpls_chainspec());
        let mut header = pulse_transition_header(
            PULSECHAIN_MAINNET.primordial_pulse_block - 1,
            ETHEREUM_MAINNET_SHANGHAI_TIMESTAMP,
            B256::repeat_byte(0x55),
            U256::ZERO,
        );
        header.withdrawals_root = Some(B256::ZERO);
        consensus
            .validate_header(&SealedHeader::new(header.clone(), B256::repeat_byte(0x56)))
            .unwrap();

        header.withdrawals_root = None;
        assert_eq!(
            consensus.validate_header(&SealedHeader::new(header, B256::repeat_byte(0x57))),
            Err(ConsensusError::WithdrawalsRootMissing)
        );

        let mut pulse_header = pulse_transition_header(
            PULSECHAIN_MAINNET.primordial_pulse_block,
            ETHEREUM_MAINNET_SHANGHAI_TIMESTAMP,
            B256::repeat_byte(0x58),
            U256::ZERO,
        );
        pulse_header.withdrawals_root = Some(B256::ZERO);
        assert_eq!(
            consensus.validate_header(&SealedHeader::new(pulse_header, B256::repeat_byte(0x59))),
            Err(ConsensusError::WithdrawalsRootUnexpected)
        );
    }

    #[test]
    fn pulse_consensus_treats_zero_difficulty_headers_as_pos_before_primordial_pulse() {
        let consensus = PulseBeaconConsensus::new(pulsechain_rpls_chainspec());
        let mut header = pulse_transition_header(
            PULSECHAIN_MAINNET.primordial_pulse_block - 1,
            ETHEREUM_MAINNET_SHANGHAI_TIMESTAMP,
            B256::repeat_byte(0x60),
            U256::ZERO,
        );
        header.withdrawals_root = Some(B256::ZERO);
        header.ommers_hash = B256::repeat_byte(0x61);

        assert_eq!(
            consensus.validate_header(&SealedHeader::new(header, B256::repeat_byte(0x62))),
            Err(ConsensusError::TheMergeOmmerRootIsNotEmpty)
        );
    }

    #[test]
    fn pulse_rpls_chainspec_uses_pulse_identity() {
        let spec = pulsechain_rpls_chainspec();
        assert_eq!(spec.chain.id(), PULSECHAIN_MAINNET.chain_id);
        assert_eq!(spec.genesis.config.chain_id, PULSECHAIN_MAINNET.chain_id);
        assert_eq!(
            spec.genesis.config.shanghai_time,
            Some(PULSECHAIN_MAINNET.shanghai_timestamp)
        );
        assert_eq!(spec.genesis.nonce, 66);
        assert_eq!(spec.genesis.gas_limit, 5_000);
        assert_eq!(spec.genesis.config.deposit_contract_address, None);
        assert!(spec.genesis.config.ethash.is_some());
        assert_eq!(spec.deposit_contract(), None);
        assert_eq!(spec.genesis_hash(), PULSECHAIN_MAINNET.genesis_hash);
    }

    #[test]
    fn pulse_rpls_testnet_v4_chainspec_uses_go_pulse_identity() {
        let spec = pulsechain_testnet_v4_rpls_chainspec();
        assert_eq!(spec.chain.id(), PULSECHAIN_TESTNET_V4.chain_id);
        assert_eq!(spec.genesis.config.chain_id, PULSECHAIN_TESTNET_V4.chain_id);
        assert_eq!(
            spec.genesis.config.shanghai_time,
            Some(PULSECHAIN_TESTNET_V4.shanghai_timestamp)
        );
        assert_eq!(spec.genesis_hash(), PULSECHAIN_TESTNET_V4.genesis_hash);
        assert_eq!(
            spec.paris_block_and_final_difficulty,
            Some((
                PULSECHAIN_TESTNET_V4.primordial_pulse_block,
                PULSECHAIN_TESTNET_V4.terminal_total_difficulty
            ))
        );
    }

    #[test]
    fn chain_parser_accepts_testnet_v4_and_devnet_alias() {
        let testnet = PulseChainSpecParser::parse("pulsechain-testnet-v4").unwrap();
        let devnet_alias = PulseChainSpecParser::parse("pulsechain-devnet").unwrap();

        assert_eq!(testnet.chain.id(), PULSECHAIN_TESTNET_V4.chain_id);
        assert_eq!(devnet_alias.chain.id(), PULSECHAIN_TESTNET_V4.chain_id);
        assert!(!PulseChainSpecParser::SUPPORTED_CHAINS.contains(&"pulsechain-devnet"));
    }

    #[test]
    fn executor_trigger_is_pulse_only_and_exact_block() {
        let pulse_mainnet = pulsechain_rpls_chainspec();
        let pulse_testnet = pulsechain_testnet_v4_rpls_chainspec();
        let ethereum_mainnet = PulseChainSpecParser::parse("mainnet").unwrap();
        let dev = PulseChainSpecParser::parse("dev").unwrap();

        assert_eq!(
            primordial_pulse_block_for_chain(pulse_mainnet.chain.id(), &pulse_mainnet),
            Some(PULSECHAIN_MAINNET.primordial_pulse_block)
        );
        assert_eq!(
            primordial_pulse_block_for_chain(pulse_testnet.chain.id(), &pulse_testnet),
            Some(PULSECHAIN_TESTNET_V4.primordial_pulse_block)
        );
        assert_eq!(
            primordial_pulse_block_for_chain(ethereum_mainnet.chain.id(), &ethereum_mainnet),
            None
        );
        assert_eq!(primordial_pulse_block_for_chain(dev.chain.id(), &dev), None);

        let fork_block = Some(PULSECHAIN_MAINNET.primordial_pulse_block);
        assert!(!should_apply_primordial_pulse(
            PULSECHAIN_MAINNET.primordial_pulse_block - 1,
            fork_block
        ));
        assert!(should_apply_primordial_pulse(
            PULSECHAIN_MAINNET.primordial_pulse_block,
            fork_block
        ));
        assert!(!should_apply_primordial_pulse(
            PULSECHAIN_MAINNET.primordial_pulse_block + 1,
            fork_block
        ));
        assert!(!should_apply_primordial_pulse(
            PULSECHAIN_MAINNET.primordial_pulse_block,
            None
        ));
    }

    #[test]
    fn evm_env_uses_ethereum_chain_id_before_pulse_and_pulse_chain_id_at_fork() {
        let config = PulseEvmConfig {
            inner: EthEvmConfig::new(pulsechain_rpls_chainspec()),
        };

        let mut header = Header {
            number: PULSECHAIN_MAINNET.primordial_pulse_block - 1,
            timestamp: 1_683_786_514,
            ..Default::default()
        };
        assert_eq!(config.evm_env(&header).unwrap().cfg_env.chain_id, 1);

        header.number = PULSECHAIN_MAINNET.primordial_pulse_block;
        header.timestamp = PULSECHAIN_MAINNET.shanghai_timestamp;
        assert_eq!(
            config.evm_env(&header).unwrap().cfg_env.chain_id,
            PULSECHAIN_MAINNET.chain_id
        );
    }

    #[test]
    fn evm_env_uses_go_pulse_shanghai_rule_around_primordial_pulse() {
        let config = PulseEvmConfig {
            inner: EthEvmConfig::new(pulsechain_rpls_chainspec()),
        };

        let mut header = Header {
            number: PULSECHAIN_MAINNET.primordial_pulse_block - 1,
            timestamp: ETHEREUM_MAINNET_SHANGHAI_TIMESTAMP,
            ..Default::default()
        };
        assert_eq!(
            config.evm_env(&header).unwrap().cfg_env.spec,
            SpecId::SHANGHAI
        );

        header.number = PULSECHAIN_MAINNET.primordial_pulse_block;
        assert!(
            !config
                .evm_env(&header)
                .unwrap()
                .cfg_env
                .spec
                .is_enabled_in(SpecId::SHANGHAI)
        );

        header.timestamp = PULSECHAIN_MAINNET.shanghai_timestamp;
        assert_eq!(
            config.evm_env(&header).unwrap().cfg_env.spec,
            SpecId::SHANGHAI
        );
    }

    #[test]
    fn testnet_v4_evm_env_uses_ethereum_chain_id_before_pulse_and_testnet_chain_id_at_fork() {
        let config = PulseEvmConfig {
            inner: EthEvmConfig::new(pulsechain_testnet_v4_rpls_chainspec()),
        };

        let mut header = Header {
            number: PULSECHAIN_TESTNET_V4.primordial_pulse_block - 1,
            timestamp: 1_682_700_368,
            ..Default::default()
        };
        assert_eq!(config.evm_env(&header).unwrap().cfg_env.chain_id, 1);

        header.number = PULSECHAIN_TESTNET_V4.primordial_pulse_block;
        header.timestamp = PULSECHAIN_TESTNET_V4.shanghai_timestamp;
        assert_eq!(
            config.evm_env(&header).unwrap().cfg_env.chain_id,
            PULSECHAIN_TESTNET_V4.chain_id
        );
    }

    fn pulse_transition_header(
        number: u64,
        timestamp: u64,
        parent_hash: B256,
        difficulty: U256,
    ) -> Header {
        Header {
            parent_hash,
            number,
            timestamp,
            difficulty,
            gas_limit: 30_000_000,
            gas_used: 15_000_000,
            base_fee_per_gas: Some(1_000_000_000),
            ..Default::default()
        }
    }

    #[test]
    fn test_apply_sacrifice_credits() {
        let mut state = State::builder().build();
        let mut state_diff = EvmState::default();

        let treasury = TreasuryCredit {
            address: alloy_primitives::address!("0xceB59257450820132aB274ED61C49E5FD96E8868"),
            amount: U256::from_str("0xC9F2C9CD04674EDEA40000000").unwrap(),
        };

        apply_sacrifice_credits(
            &mut state,
            &mut state_diff,
            Some(&treasury),
            PULSECHAIN_MAINNET_CHAIN_ID,
        )
        .unwrap();
        state.commit(state_diff);

        assert_eq!(
            account_balance(&mut state, treasury.address),
            treasury.amount
        );
    }

    #[test]
    fn sacrifice_credits_are_additive() {
        let mut state = State::builder().build();
        let recipient = mainnet_known_sacrifice_recipient();
        insert_account_with_balance(&mut state, recipient, U256::from(7u64));

        let mut state_diff = EvmState::default();
        apply_sacrifice_credits(
            &mut state,
            &mut state_diff,
            PULSECHAIN_MAINNET.treasury.as_ref(),
            PULSECHAIN_MAINNET_CHAIN_ID,
        )
        .unwrap();
        state.commit(state_diff);

        assert_eq!(
            account_balance(&mut state, recipient),
            U256::from(64_000_000_000_000_000_007u128)
        );
    }

    #[test]
    fn testnet_v4_sacrifice_uses_testnet_allocation_and_treasury() {
        let mut state = State::builder().build();
        let mut state_diff = EvmState::default();

        apply_sacrifice_credits(
            &mut state,
            &mut state_diff,
            PULSECHAIN_TESTNET_V4.treasury.as_ref(),
            PULSECHAIN_TESTNET_V4_CHAIN_ID,
        )
        .unwrap();
        state.commit(state_diff);

        assert_eq!(
            account_balance(&mut state, testnet_v4_differential_sacrifice_recipient()),
            U256::from(64_000_000_000_000_000_000u128)
        );

        let treasury = PULSECHAIN_TESTNET_V4.treasury.unwrap();
        assert_eq!(
            account_balance(&mut state, treasury.address),
            treasury.amount
        );
    }

    #[test]
    fn primordial_pulse_applies_sacrifice_then_deposit_replacement() {
        let mut state = State::builder().build();

        apply_primordial_pulse_state(
            &mut state,
            PULSECHAIN_MAINNET.treasury.as_ref(),
            PULSECHAIN_MAINNET_CHAIN_ID,
        )
        .unwrap();

        assert_eq!(
            account_balance(&mut state, mainnet_known_sacrifice_recipient()),
            U256::from(64_000_000_000_000_000_000u128)
        );
        assert_pulse_deposit_contract_replaced(&mut state);
    }

    #[test]
    fn testnet_v4_primordial_pulse_applies_testnet_sacrifice_treasury_and_deposit_replacement() {
        let mut state = State::builder().build();

        apply_primordial_pulse_state(
            &mut state,
            PULSECHAIN_TESTNET_V4.treasury.as_ref(),
            PULSECHAIN_TESTNET_V4_CHAIN_ID,
        )
        .unwrap();

        assert_eq!(
            account_balance(&mut state, testnet_v4_differential_sacrifice_recipient()),
            U256::from(64_000_000_000_000_000_000u128)
        );
        let treasury = PULSECHAIN_TESTNET_V4.treasury.unwrap();
        assert_eq!(
            account_balance(&mut state, treasury.address),
            treasury.amount
        );
        assert_pulse_deposit_contract_replaced(&mut state);
    }

    #[test]
    fn replace_deposit_contract_matches_go_pulse_test() {
        let mut state = State::builder().build();
        let mut state_diff = EvmState::default();

        replace_deposit_contract(&mut state, &mut state_diff).unwrap();
        state.commit(state_diff);

        let old_account = state
            .basic(ETHEREUM_DEPOSIT_CONTRACT)
            .unwrap()
            .expect("Ethereum deposit contract should exist in empty-state replacement");
        assert_eq!(
            old_account.code.unwrap().original_byte_slice(),
            NIL_CONTRACT_BYTECODE
        );

        assert_pulse_deposit_contract_replaced(&mut state);
    }

    #[test]
    fn replace_deposit_contract_selfdestructs_existing_eth_deposit_contract() {
        let mut state = State::builder().build();
        insert_account_with_code_and_balance(
            &mut state,
            ETHEREUM_DEPOSIT_CONTRACT,
            &[0x60, 0x00, 0x60, 0x00],
            U256::from(123u64),
        );

        let mut state_diff = EvmState::default();
        replace_deposit_contract(&mut state, &mut state_diff).unwrap();
        state.commit(state_diff);

        assert!(
            state.basic(ETHEREUM_DEPOSIT_CONTRACT).unwrap().is_none(),
            "existing Ethereum deposit contract should be removed after selfdestruct"
        );
        assert_pulse_deposit_contract_replaced(&mut state);
    }

    fn assert_pulse_deposit_contract_replaced<DB>(state: &mut State<DB>)
    where
        DB: Database,
    {
        let account = state
            .basic(PULSECHAIN_DEPOSIT_CONTRACT)
            .unwrap()
            .expect("PulseChain deposit contract should exist");
        assert_eq!(account.balance, U256::ZERO);
        assert_eq!(
            account.code.unwrap().original_byte_slice(),
            PULSE_DEPOSIT_CONTRACT_BYTECODE
        );

        for entry in DEPOSIT_CONTRACT_STORAGE {
            let actual = state
                .storage(
                    PULSECHAIN_DEPOSIT_CONTRACT,
                    go_pulse_hash_to_u256(entry.key),
                )
                .unwrap();
            assert_eq!(actual, go_pulse_hash_to_u256(entry.value));
        }
    }

    fn account_balance<DB>(state: &mut State<DB>, address: Address) -> U256
    where
        DB: Database,
    {
        state
            .basic(address)
            .unwrap()
            .map(|account| account.balance)
            .unwrap_or_default()
    }

    fn insert_account_with_balance<DB>(state: &mut State<DB>, address: Address, balance: U256)
    where
        DB: Database,
    {
        let mut account = Account::from(AccountInfo::default());
        account.info.balance = balance;
        account.mark_touch();

        state.basic(address).unwrap();
        let mut state_diff = EvmState::default();
        state_diff.insert(address, account);
        state.commit(state_diff);
    }

    fn insert_account_with_code_and_balance<DB>(
        state: &mut State<DB>,
        address: Address,
        code: &[u8],
        balance: U256,
    ) where
        DB: Database,
    {
        let mut account = Account::from(AccountInfo::default());
        account.info.balance = balance;
        account
            .info
            .set_code(Bytecode::new_raw(code.to_vec().into()));
        account.mark_touch();

        state.basic(address).unwrap();
        let mut state_diff = EvmState::default();
        state_diff.insert(address, account);
        state.commit(state_diff);
    }

    fn mainnet_known_sacrifice_recipient() -> Address {
        Address::from([
            0x00, 0x00, 0x00, 0x00, 0x5d, 0xce, 0xe1, 0x1e, 0x13, 0xfb, 0x53, 0x6f, 0xa4, 0x0d,
            0x65, 0x45, 0x0f, 0x53, 0xc5, 0xa8,
        ])
    }

    fn testnet_v4_differential_sacrifice_recipient() -> Address {
        Address::from([
            0x00, 0x82, 0x7a, 0x4d, 0x91, 0x02, 0x8b, 0x7d, 0x0e, 0xba, 0x24, 0x1f, 0x7f, 0x1a,
            0x2e, 0xc2, 0xca, 0xa5, 0x6b, 0x78,
        ])
    }
}
