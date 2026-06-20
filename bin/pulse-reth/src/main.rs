//! PulseChain Reth binary.
//!
//! The binary reuses Reth's CLI and Ethereum node components while selecting
//! PulseChain chain specs, installing the Pulse executor, validating embedded
//! fork artifacts, and injecting go-pulse default bootnodes when the user did
//! not provide explicit peers.

use clap::Parser;
use pulsechain_chainspec::{PULSECHAIN_BOOTNODES, PULSECHAIN_TESTNET_V4_BOOTNODES};
use pulsechain_evm::sacrifice::MAINNET_ALLOCATION;
use pulsechain_hardforks::{PULSECHAIN_MAINNET_CHAIN_ID, PULSECHAIN_TESTNET_V4_CHAIN_ID};
use pulsechain_node::{PulseChainSpecParser, PulseConsensusBuilder, PulseExecutorBuilder};
use reth_ethereum::{
    cli::interface::{Cli, Commands},
    node::{EthereumNode, node::EthereumAddOns},
};
use reth_network_peers::TrustedPeer;

const GO_PULSE_RPC_TX_FEE_CAP_WEI: u128 = 1_000_000_000_000_000_000_000_000;

fn main() {
    reth_cli_util::sigsegv_handler::install();

    if std::env::var_os("RUST_BACKTRACE").is_none() {
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }

    MAINNET_ALLOCATION
        .validate()
        .expect("embedded PulseChain sacrifice allocation must match go-pulse");

    let mut cli = Cli::<PulseChainSpecParser>::parse();
    apply_default_pulsechain_bootnodes(&mut cli);
    apply_default_go_pulse_rpc_tx_fee_cap(&mut cli, rpc_tx_fee_cap_was_set(std::env::args_os()));

    cli.run(|builder, _| async move {
        let handle = builder
            .with_types::<EthereumNode>()
            .with_components(
                EthereumNode::components()
                    .executor(PulseExecutorBuilder::default())
                    .consensus(PulseConsensusBuilder::default()),
            )
            .with_add_ons(EthereumAddOns::default())
            .launch()
            .await?;

        handle.wait_for_node_exit().await
    })
    .unwrap();
}

fn apply_default_pulsechain_bootnodes(cli: &mut Cli<PulseChainSpecParser>) {
    let Commands::Node(command) = &mut cli.command else {
        return;
    };

    if command.network.bootnodes.is_some() {
        return;
    }

    let bootnodes = match command.chain.chain.id() {
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

fn apply_default_go_pulse_rpc_tx_fee_cap(
    cli: &mut Cli<PulseChainSpecParser>,
    rpc_tx_fee_cap_was_set: bool,
) {
    let Commands::Node(command) = &mut cli.command else {
        return;
    };

    if rpc_tx_fee_cap_was_set {
        return;
    }

    match command.chain.chain.id() {
        PULSECHAIN_MAINNET_CHAIN_ID | PULSECHAIN_TESTNET_V4_CHAIN_ID => {
            command.rpc.rpc_tx_fee_cap = GO_PULSE_RPC_TX_FEE_CAP_WEI;
        }
        _ => {}
    }
}

fn rpc_tx_fee_cap_was_set<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    args.into_iter().any(|arg| {
        let Some(arg) = arg.as_ref().to_str() else {
            return false;
        };

        matches!(arg, "--rpc.txfeecap" | "--rpc-txfeecap")
            || arg.starts_with("--rpc.txfeecap=")
            || arg.starts_with("--rpc-txfeecap=")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn help_lists_only_visible_builtin_chains() {
        let mut command = Cli::<PulseChainSpecParser>::command();
        let help = command
            .find_subcommand_mut("node")
            .expect("node subcommand should exist")
            .render_long_help()
            .to_string();

        assert!(help.contains("pulsechain, pulsechain-testnet-v4, mainnet, dev"));
        assert!(!help.contains("pulsechain, pulse,"));
        assert!(!help.contains("pulsechain-devnet"));
    }

    #[test]
    fn injects_mainnet_bootnodes_by_default() {
        let mut cli = parse_node_cli(["pulse-reth", "node", "--chain", "pulsechain"]);

        apply_default_pulsechain_bootnodes(&mut cli);

        let Commands::Node(command) = cli.command else {
            panic!("expected node command");
        };
        assert_eq!(
            command.network.bootnodes.unwrap().len(),
            PULSECHAIN_BOOTNODES.len()
        );
    }

    #[test]
    fn injects_testnet_v4_bootnodes_by_default() {
        let mut cli = parse_node_cli(["pulse-reth", "node", "--chain", "pulsechain-testnet-v4"]);

        apply_default_pulsechain_bootnodes(&mut cli);

        let Commands::Node(command) = cli.command else {
            panic!("expected node command");
        };
        assert_eq!(
            command.network.bootnodes.unwrap().len(),
            PULSECHAIN_TESTNET_V4_BOOTNODES.len()
        );
    }

    #[test]
    fn user_bootnodes_are_not_overwritten() {
        let explicit_bootnode = PULSECHAIN_BOOTNODES[0];
        let mut cli = parse_node_cli([
            "pulse-reth",
            "node",
            "--chain",
            "pulsechain",
            "--bootnodes",
            explicit_bootnode,
        ]);

        apply_default_pulsechain_bootnodes(&mut cli);

        let Commands::Node(command) = cli.command else {
            panic!("expected node command");
        };
        assert_eq!(command.network.bootnodes.unwrap().len(), 1);
    }

    #[test]
    fn applies_go_pulse_rpc_tx_fee_cap_by_default() {
        let mut cli = parse_node_cli(["pulse-reth", "node", "--chain", "pulsechain"]);

        apply_default_go_pulse_rpc_tx_fee_cap(&mut cli, false);

        let Commands::Node(command) = cli.command else {
            panic!("expected node command");
        };
        assert_eq!(command.rpc.rpc_tx_fee_cap, GO_PULSE_RPC_TX_FEE_CAP_WEI);
    }

    #[test]
    fn preserves_user_rpc_tx_fee_cap() {
        let mut cli = parse_node_cli([
            "pulse-reth",
            "node",
            "--chain",
            "pulsechain",
            "--rpc.txfeecap",
            "2",
        ]);

        apply_default_go_pulse_rpc_tx_fee_cap(&mut cli, true);

        let Commands::Node(command) = cli.command else {
            panic!("expected node command");
        };
        assert_eq!(command.rpc.rpc_tx_fee_cap, 2_000_000_000_000_000_000);
    }

    #[test]
    fn leaves_ethereum_chains_on_reth_rpc_tx_fee_cap_default() {
        let mut cli = parse_node_cli(["pulse-reth", "node", "--chain", "mainnet"]);

        apply_default_go_pulse_rpc_tx_fee_cap(&mut cli, false);

        let Commands::Node(command) = cli.command else {
            panic!("expected node command");
        };
        assert_eq!(command.rpc.rpc_tx_fee_cap, 1_000_000_000_000_000_000);
    }

    #[test]
    fn detects_rpc_tx_fee_cap_flag_forms() {
        assert!(rpc_tx_fee_cap_was_set([
            "pulse-reth",
            "node",
            "--rpc.txfeecap",
            "2"
        ]));
        assert!(rpc_tx_fee_cap_was_set([
            "pulse-reth",
            "node",
            "--rpc.txfeecap=2"
        ]));
        assert!(rpc_tx_fee_cap_was_set([
            "pulse-reth",
            "node",
            "--rpc-txfeecap",
            "2"
        ]));
        assert!(rpc_tx_fee_cap_was_set([
            "pulse-reth",
            "node",
            "--rpc-txfeecap=2"
        ]));
        assert!(!rpc_tx_fee_cap_was_set(["pulse-reth", "node"]));
    }

    fn parse_node_cli<const N: usize>(args: [&str; N]) -> Cli<PulseChainSpecParser> {
        Cli::<PulseChainSpecParser>::try_parse_from(args).expect("node CLI args should parse")
    }
}
