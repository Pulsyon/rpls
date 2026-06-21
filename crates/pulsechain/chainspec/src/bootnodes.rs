use crate::{PulseChainSpec, pulsechain_spec_for_chain_id};
use pulsechain_hardforks::{PULSECHAIN_MAINNET_CHAIN_ID, PULSECHAIN_TESTNET_V4_CHAIN_ID};
use reth_ethereum::{
    chainspec::{EthChainSpec, ForkFilter, ForkFilterKey, Hardforks, Head},
    cli::interface::{Cli, Commands},
};
use reth_network::{NetworkHandle, NetworkManager, PeersInfo, primitives::BasicNetworkPrimitives};
use reth_network_peers::TrustedPeer;
use reth_node_api::{FullNodeTypes, NodeTypes, PrimitivesTy, TxTy};
use reth_node_builder::{BuilderContext, components::NetworkBuilder};
use reth_transaction_pool::{PoolPooledTx, PoolTransaction, TransactionPool};
use std::fmt;
use tracing::info;

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

#[cfg(test)]
pub const PULSE_DNS_DISCOVERY_PREFIX: &str =
    "enrtree://APFXO36RU3TWV7XFGWI2TYF5IDA3WM2GPTRL3TCZINWHZX4R6TAOK@";
pub const PULSECHAIN_DNS_DISCOVERY_URL: &str =
    "enrtree://APFXO36RU3TWV7XFGWI2TYF5IDA3WM2GPTRL3TCZINWHZX4R6TAOK@all.mainnet.pulsedisco.net";
pub const PULSECHAIN_TESTNET_V4_DNS_DISCOVERY_URL: &str =
    "enrtree://APFXO36RU3TWV7XFGWI2TYF5IDA3WM2GPTRL3TCZINWHZX4R6TAOK@all.testnet-v4.pulsedisco.net";

pub const fn pulse_dns_discovery_url_for_chain_id(chain_id: u64) -> Option<&'static str> {
    match chain_id {
        PULSECHAIN_MAINNET_CHAIN_ID => Some(PULSECHAIN_DNS_DISCOVERY_URL),
        PULSECHAIN_TESTNET_V4_CHAIN_ID => Some(PULSECHAIN_TESTNET_V4_DNS_DISCOVERY_URL),
        _ => None,
    }
}

pub fn apply_default_pulsechain_bootnodes<C, Ext, Rpc>(cli: &mut Cli<C, Ext, Rpc>)
where
    C: reth_cli::chainspec::ChainSpecParser,
    C::ChainSpec: EthChainSpec,
    Ext: clap::Args + fmt::Debug,
    Rpc: reth_rpc_server_types::RpcModuleValidator,
{
    let Commands::Node(command) = &mut cli.command else {
        return;
    };

    if command.network.bootnodes.is_some() {
        return;
    }

    let bootnodes = match command.chain.chain().id() {
        PULSECHAIN_MAINNET_CHAIN_ID => PULSECHAIN_BOOTNODES,
        PULSECHAIN_TESTNET_V4_CHAIN_ID => PULSECHAIN_TESTNET_V4_BOOTNODES,
        _ => return,
    };

    command.network.bootnodes = Some(
        bootnodes
            .iter()
            .map(|bootnode| {
                bootnode
                    .parse::<TrustedPeer>()
                    .expect("embedded PulseChain bootnode must parse as an enode URL")
            })
            .collect(),
    );
}

fn apply_pulse_dns_discovery<C, N>(
    network_config: &mut reth_network::NetworkConfig<C, N>,
) -> eyre::Result<()>
where
    N: reth_network::NetworkPrimitives,
{
    let Some(dns_url) = pulse_dns_discovery_url_for_chain_id(network_config.chain_id) else {
        return Ok(());
    };

    let Some(dns_config) = network_config.dns_discovery_config.as_mut() else {
        return Ok(());
    };

    let dns_networks = dns_config
        .bootstrap_dns_networks
        .get_or_insert_with(Default::default);
    dns_networks.insert(dns_url.parse()?);
    Ok(())
}

fn pulse_fork_filter_keys(pulse_spec: PulseChainSpec) -> Vec<ForkFilterKey> {
    let mut forks = Vec::with_capacity(pulse_spec.ethereum_block_forks.len() + 2);
    forks.extend(
        pulse_spec
            .ethereum_block_forks
            .iter()
            .map(|fork| ForkFilterKey::Block(fork.block)),
    );
    forks.push(ForkFilterKey::Block(pulse_spec.primordial_pulse_block));
    forks.push(ForkFilterKey::Time(pulse_spec.shanghai_timestamp));
    forks
}

fn pulse_fork_filter_for_chain_id(chain_id: u64, head: Head) -> Option<ForkFilter> {
    let pulse_spec = pulsechain_spec_for_chain_id(chain_id)?;
    Some(ForkFilter::new(
        head,
        pulse_spec.genesis_hash,
        0,
        pulse_fork_filter_keys(pulse_spec),
    ))
}

fn apply_pulse_fork_filter<C, N>(network_config: &mut reth_network::NetworkConfig<C, N>, head: Head)
where
    N: reth_network::NetworkPrimitives,
{
    let Some(fork_filter) = pulse_fork_filter_for_chain_id(network_config.chain_id, head) else {
        return;
    };

    let fork_id = fork_filter.current();
    network_config.fork_filter = fork_filter;
    network_config.status.forkid = fork_id;
    info!(target: "rpls::cli", ?fork_id, "Applied go-pulse fork ID filter");
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PulseNetworkBuilder;

impl<Node, Pool> NetworkBuilder<Node, Pool> for PulseNetworkBuilder
where
    Node: FullNodeTypes<Types: NodeTypes<ChainSpec: Hardforks>>,
    Pool: TransactionPool<Transaction: PoolTransaction<Consensus = TxTy<Node::Types>>>
        + Unpin
        + 'static,
{
    type Network =
        NetworkHandle<BasicNetworkPrimitives<PrimitivesTy<Node::Types>, PoolPooledTx<Pool>>>;

    async fn build_network(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
    ) -> eyre::Result<Self::Network> {
        let network_config_builder = ctx.network_config_builder()?;
        let mut network_config = ctx.build_network_config(network_config_builder);
        apply_pulse_dns_discovery(&mut network_config)?;
        apply_pulse_fork_filter(&mut network_config, ctx.head());

        let network = NetworkManager::builder(network_config).await?;
        let handle = ctx.start_network(network, pool);
        info!(target: "rpls::cli", enode=%handle.local_node_record(), "P2P networking initialized");
        Ok(handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::B256;
    use reth_ethereum::chainspec::{ForkHash, ForkId};

    fn head(number: u64, timestamp: u64) -> Head {
        Head {
            number,
            timestamp,
            hash: B256::ZERO,
            difficulty: Default::default(),
            total_difficulty: Default::default(),
        }
    }

    fn fork_id(hash: [u8; 4], next: u64) -> ForkId {
        ForkId {
            hash: ForkHash(hash),
            next,
        }
    }

    fn pulse_fork_id(chain_id: u64, number: u64, timestamp: u64) -> ForkId {
        pulse_fork_filter_for_chain_id(chain_id, head(number, timestamp))
            .expect("PulseChain fork filter must exist")
            .current()
    }

    #[test]
    fn pulse_dns_discovery_urls_match_go_pulse() {
        assert!(PULSECHAIN_DNS_DISCOVERY_URL.starts_with(PULSE_DNS_DISCOVERY_PREFIX));
        assert!(PULSECHAIN_TESTNET_V4_DNS_DISCOVERY_URL.starts_with(PULSE_DNS_DISCOVERY_PREFIX));
        assert_eq!(
            pulse_dns_discovery_url_for_chain_id(PULSECHAIN_MAINNET_CHAIN_ID),
            Some(PULSECHAIN_DNS_DISCOVERY_URL)
        );
        assert_eq!(
            pulse_dns_discovery_url_for_chain_id(PULSECHAIN_TESTNET_V4_CHAIN_ID),
            Some(PULSECHAIN_TESTNET_V4_DNS_DISCOVERY_URL)
        );
    }

    #[test]
    fn non_pulse_chains_do_not_get_pulse_dns_discovery() {
        assert_eq!(pulse_dns_discovery_url_for_chain_id(1), None);
    }

    #[test]
    fn pulse_dns_discovery_urls_parse_as_enrtree_links() {
        for url in [
            PULSECHAIN_DNS_DISCOVERY_URL,
            PULSECHAIN_TESTNET_V4_DNS_DISCOVERY_URL,
        ] {
            url.parse::<reth_dns_discovery::tree::LinkEntry>()
                .expect("Pulse DNS discovery URL must parse as an ENR tree link");
        }
    }

    #[test]
    fn mainnet_fork_ids_match_go_pulse() {
        let cases = [
            (0, 0, fork_id([0xfc, 0x64, 0xec, 0x04], 1_150_000)),
            (1_149_999, 0, fork_id([0xfc, 0x64, 0xec, 0x04], 1_150_000)),
            (1_150_000, 0, fork_id([0x97, 0xc2, 0xc3, 0x4c], 1_920_000)),
            (15_050_000, 0, fork_id([0xf0, 0xaf, 0xd0, 0xe3], 17_233_000)),
            (
                17_232_999,
                1_683_786_514,
                fork_id([0xf0, 0xaf, 0xd0, 0xe3], 17_233_000),
            ),
            (
                17_233_000,
                1_683_786_514,
                fork_id([0xec, 0x48, 0xdb, 0xcc], 1_683_786_515),
            ),
            (
                17_233_000,
                1_683_786_515,
                fork_id([0x62, 0xeb, 0xcf, 0xaf], 0),
            ),
        ];

        for (number, timestamp, expected) in cases {
            assert_eq!(
                pulse_fork_id(PULSECHAIN_MAINNET_CHAIN_ID, number, timestamp),
                expected
            );
        }
    }

    #[test]
    fn testnet_v4_fork_ids_match_go_pulse() {
        let cases = [
            (0, 0, fork_id([0xfc, 0x64, 0xec, 0x04], 1_150_000)),
            (15_050_000, 0, fork_id([0xf0, 0xaf, 0xd0, 0xe3], 16_492_700)),
            (
                16_492_700,
                1_682_700_368,
                fork_id([0x45, 0x36, 0x78, 0x77], 1_682_700_369),
            ),
            (
                16_492_700,
                1_682_700_369,
                fork_id([0x56, 0x82, 0xe2, 0x87], 0),
            ),
        ];

        for (number, timestamp, expected) in cases {
            assert_eq!(
                pulse_fork_id(PULSECHAIN_TESTNET_V4_CHAIN_ID, number, timestamp),
                expected
            );
        }
    }

    #[test]
    fn non_pulse_chains_do_not_get_a_pulse_fork_filter() {
        assert!(pulse_fork_filter_for_chain_id(1, head(0, 0)).is_none());
    }
}
