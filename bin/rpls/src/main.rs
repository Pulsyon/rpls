//! rpls binary.
//!
//! The binary reuses the upstream CLI and Ethereum node components while selecting
//! PulseChain chain specs, installing the Pulse executor, validating embedded
//! fork artifacts, and injecting go-pulse default bootnodes when the user did
//! not provide explicit peers.

use clap::Parser;
use pulsechain_chainspec::{PulseNetworkBuilder, apply_default_pulsechain_bootnodes};
use pulsechain_evm::sacrifice::MAINNET_ALLOCATION;
use pulsechain_hardforks::{PULSECHAIN_MAINNET_CHAIN_ID, PULSECHAIN_TESTNET_V4_CHAIN_ID};
use pulsechain_node::{PulseChainSpecParser, PulseConsensusBuilder, PulseExecutorBuilder};
use reth_ethereum::{
    cli::interface::{Cli, Commands},
    node::{EthereumNode, node::EthereumAddOns},
};
use std::{
    ffi::OsStr,
    fmt, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

const GO_PULSE_RPC_TX_FEE_CAP_WEI: u128 = 1_000_000_000_000_000_000_000_000;
const RPLS_APP_DIR_NAME: &str = "rpls";
const RPLS_MINIMAL_PRUNING_DISTANCE: u64 = 32 * 2 + 10_000;

#[derive(Debug, Clone, clap::Args)]
struct RplsArgs {
    /// Use aggressive pruning defaults to minimize disk usage.
    #[arg(long = "rpls.minimal-pruning", default_value_t = false)]
    minimal_pruning: bool,
}

fn main() {
    reth_cli_util::sigsegv_handler::install();

    if std::env::var_os("RUST_BACKTRACE").is_none() {
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }

    MAINNET_ALLOCATION
        .validate()
        .expect("embedded PulseChain sacrifice allocation must match go-pulse");

    let args: Vec<_> = std::env::args_os().collect();
    let mut cli = Cli::<PulseChainSpecParser, RplsArgs>::parse();
    apply_default_pulsechain_bootnodes(&mut cli);
    apply_default_rpls_datadir(&mut cli, datadir_was_set(&args));
    apply_default_rpls_logs(
        &mut cli,
        log_file_directory_was_set(&args),
        log_file_name_was_set(&args),
    );

    apply_default_go_pulse_rpc_tx_fee_cap(&mut cli, rpc_tx_fee_cap_was_set(&args));
    apply_minimal_rpls_pruning(&mut cli, PruneFlagSet::from_args(&args));
    repair_existing_minimal_pruning_config(&cli)
        .expect("could not repair existing rpls minimal pruning config");

    cli.run(|builder, _| async move {
        let handle = builder
            .with_types::<EthereumNode>()
            .with_components(
                EthereumNode::components()
                    .network(PulseNetworkBuilder)
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

fn repair_existing_minimal_pruning_config(
    cli: &Cli<PulseChainSpecParser, RplsArgs>,
) -> std::io::Result<()> {
    let Commands::Node(command) = &cli.command else {
        return Ok(());
    };

    if !command.ext.minimal_pruning {
        return Ok(());
    }

    let data_dir = command.datadir.clone().resolve_datadir(command.chain.chain);
    let config_path = command.config.clone().unwrap_or_else(|| data_dir.config());
    repair_minimal_pruning_config_file(&config_path)
}

fn repair_minimal_pruning_config_file(config_path: &Path) -> std::io::Result<()> {
    let contents = match fs::read_to_string(config_path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err),
    };
    let repaired = repair_minimal_pruning_config_contents(&contents);

    if repaired != contents {
        fs::write(config_path, repaired)?;
    }

    Ok(())
}

fn repair_minimal_pruning_config_contents(contents: &str) -> String {
    let mut repaired = contents.to_owned();

    for segment in [
        "receipts",
        "account_history",
        "storage_history",
        "bodies_history",
    ] {
        for quote in ['"', '\''] {
            repaired = repaired.replace(
                &format!("{segment} = {quote}full{quote}"),
                &format!("{segment} = {{ distance = {RPLS_MINIMAL_PRUNING_DISTANCE} }}"),
            );
        }
    }

    repaired
}

fn apply_default_rpls_datadir<Ext>(cli: &mut Cli<PulseChainSpecParser, Ext>, datadir_was_set: bool)
where
    Ext: clap::Args + fmt::Debug,
{
    let Commands::Node(command) = &mut cli.command else {
        return;
    };

    if datadir_was_set {
        return;
    }

    command.datadir.datadir = default_rpls_chain_datadir(command.chain.chain.to_string()).into();
}

fn default_rpls_datadir() -> PathBuf {
    dirs_next::data_dir()
        .map(|root| root.join(RPLS_APP_DIR_NAME))
        .expect("could not resolve default rpls data directory; set --datadir manually")
}

fn default_rpls_chain_datadir(chain: impl AsRef<str>) -> PathBuf {
    default_rpls_datadir().join(chain.as_ref())
}

fn apply_default_rpls_logs<Ext>(
    cli: &mut Cli<PulseChainSpecParser, Ext>,
    log_file_directory_was_set: bool,
    log_file_name_was_set: bool,
) where
    Ext: clap::Args + fmt::Debug,
{
    if !log_file_directory_was_set {
        let log_dir = default_rpls_logs_dir();
        cli.logs.log_file_directory = log_dir
            .to_string_lossy()
            .parse()
            .expect("could not parse default rpls log directory");
    }

    if !log_file_name_was_set {
        cli.logs.log_file_name = "rpls.log".to_string();
    }
}

fn default_rpls_logs_dir() -> PathBuf {
    dirs_next::cache_dir()
        .map(|root| root.join(RPLS_APP_DIR_NAME).join("logs"))
        .expect("could not resolve default rpls log directory")
}

fn apply_default_go_pulse_rpc_tx_fee_cap<Ext>(
    cli: &mut Cli<PulseChainSpecParser, Ext>,
    rpc_tx_fee_cap_was_set: bool,
) where
    Ext: clap::Args + fmt::Debug,
{
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

fn apply_minimal_rpls_pruning(
    cli: &mut Cli<PulseChainSpecParser, RplsArgs>,
    prune_flags: PruneFlagSet,
) {
    let Commands::Node(command) = &mut cli.command else {
        return;
    };

    if !command.ext.minimal_pruning {
        return;
    }

    command.pruning.full = true;

    if !prune_flags.sender_recovery {
        command.pruning.sender_recovery_full = true;
    }
    if !prune_flags.transaction_lookup {
        command.pruning.transaction_lookup_full = true;
    }
    if !prune_flags.receipts {
        command.pruning.receipts_distance = Some(RPLS_MINIMAL_PRUNING_DISTANCE);
    }
    if !prune_flags.account_history {
        command.pruning.account_history_distance = Some(RPLS_MINIMAL_PRUNING_DISTANCE);
    }
    if !prune_flags.storage_history {
        command.pruning.storage_history_distance = Some(RPLS_MINIMAL_PRUNING_DISTANCE);
    }
    if !prune_flags.bodies {
        command.pruning.bodies_distance = Some(RPLS_MINIMAL_PRUNING_DISTANCE);
    }
}

fn rpc_tx_fee_cap_was_set<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
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

fn datadir_was_set<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    args.into_iter().any(|arg| {
        let Some(arg) = arg.as_ref().to_str() else {
            return false;
        };

        arg == "--datadir" || arg.starts_with("--datadir=")
    })
}

fn log_file_directory_was_set<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    args.into_iter().any(|arg| {
        let Some(arg) = arg.as_ref().to_str() else {
            return false;
        };

        arg == "--log.file.directory" || arg.starts_with("--log.file.directory=")
    })
}

fn log_file_name_was_set<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    args.into_iter().any(|arg| {
        let Some(arg) = arg.as_ref().to_str() else {
            return false;
        };

        arg == "--log.file.name" || arg.starts_with("--log.file.name=")
    })
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct PruneFlagSet {
    sender_recovery: bool,
    transaction_lookup: bool,
    receipts: bool,
    account_history: bool,
    storage_history: bool,
    bodies: bool,
}

impl PruneFlagSet {
    fn from_args<I, S>(args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut flags = Self::default();

        for arg in args {
            let Some(arg) = arg.as_ref().to_str() else {
                continue;
            };

            flags.sender_recovery |= arg.starts_with("--prune.senderrecovery.");
            flags.transaction_lookup |= arg.starts_with("--prune.transactionlookup.");
            flags.receipts |= arg.starts_with("--prune.receipts.")
                || arg.starts_with("--prune.receiptslogfilter");
            flags.account_history |= arg.starts_with("--prune.accounthistory.");
            flags.storage_history |= arg.starts_with("--prune.storagehistory.");
            flags.bodies |= arg.starts_with("--prune.bodies.");
        }

        flags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn help_lists_only_visible_builtin_chains() {
        let mut command = Cli::<PulseChainSpecParser, RplsArgs>::command();
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
    fn help_lists_rpls_minimal_pruning() {
        let mut command = Cli::<PulseChainSpecParser, RplsArgs>::command();
        let help = command
            .find_subcommand_mut("node")
            .expect("node subcommand should exist")
            .render_long_help()
            .to_string();

        assert!(help.contains("--rpls.minimal-pruning"));
    }

    #[test]
    fn applies_go_pulse_rpc_tx_fee_cap_by_default() {
        let mut cli = parse_node_cli(["rpls", "node", "--chain", "pulsechain"]);

        apply_default_go_pulse_rpc_tx_fee_cap(&mut cli, false);

        let Commands::Node(command) = cli.command else {
            panic!("expected node command");
        };
        assert_eq!(command.rpc.rpc_tx_fee_cap, GO_PULSE_RPC_TX_FEE_CAP_WEI);
    }

    #[test]
    fn preserves_user_rpc_tx_fee_cap() {
        let mut cli = parse_node_cli([
            "rpls",
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
    fn leaves_ethereum_chains_on_upstream_rpc_tx_fee_cap_default() {
        let mut cli = parse_node_cli(["rpls", "node", "--chain", "mainnet"]);

        apply_default_go_pulse_rpc_tx_fee_cap(&mut cli, false);

        let Commands::Node(command) = cli.command else {
            panic!("expected node command");
        };
        assert_eq!(command.rpc.rpc_tx_fee_cap, 1_000_000_000_000_000_000);
    }

    #[test]
    fn detects_rpc_tx_fee_cap_flag_forms() {
        assert!(rpc_tx_fee_cap_was_set([
            "rpls",
            "node",
            "--rpc.txfeecap",
            "2"
        ]));
        assert!(rpc_tx_fee_cap_was_set(["rpls", "node", "--rpc.txfeecap=2"]));
        assert!(rpc_tx_fee_cap_was_set([
            "rpls",
            "node",
            "--rpc-txfeecap",
            "2"
        ]));
        assert!(rpc_tx_fee_cap_was_set(["rpls", "node", "--rpc-txfeecap=2"]));
        assert!(!rpc_tx_fee_cap_was_set(["rpls", "node"]));
    }

    #[test]
    fn applies_rpls_datadir_by_default() {
        let mut cli = parse_node_cli(["rpls", "node", "--chain", "pulsechain"]);

        apply_default_rpls_datadir(&mut cli, false);

        let Commands::Node(command) = cli.command else {
            panic!("expected node command");
        };
        let data_dir = command.datadir.resolve_datadir(command.chain.chain);
        assert_eq!(
            data_dir.as_ref(),
            default_rpls_chain_datadir("pulsechain").as_path()
        );
    }

    #[test]
    fn preserves_user_datadir() {
        let mut cli = parse_node_cli([
            "rpls",
            "node",
            "--chain",
            "pulsechain",
            "--datadir",
            "custom-data",
        ]);

        apply_default_rpls_datadir(&mut cli, true);

        let Commands::Node(command) = cli.command else {
            panic!("expected node command");
        };
        let data_dir = command.datadir.resolve_datadir(command.chain.chain);
        assert!(data_dir.as_ref().ends_with("custom-data"));
    }

    #[test]
    fn detects_datadir_flag_forms() {
        assert!(datadir_was_set([
            "rpls",
            "node",
            "--datadir",
            "custom-data"
        ]));
        assert!(datadir_was_set(["rpls", "node", "--datadir=custom-data"]));
        assert!(!datadir_was_set(["rpls", "node"]));
    }

    #[test]
    fn applies_rpls_logs_by_default() {
        let mut cli = parse_node_cli(["rpls", "node", "--chain", "pulsechain"]);

        apply_default_rpls_logs(&mut cli, false, false);

        assert_eq!(
            cli.logs.log_file_directory.to_string(),
            default_rpls_logs_dir().display().to_string()
        );
        assert_eq!(cli.logs.log_file_name, "rpls.log");
    }

    #[test]
    fn preserves_user_logs() {
        let mut cli = parse_node_cli([
            "rpls",
            "--log.file.directory",
            "custom-logs",
            "--log.file.name",
            "custom.log",
            "node",
            "--chain",
            "pulsechain",
        ]);

        apply_default_rpls_logs(&mut cli, true, true);

        assert!(
            cli.logs
                .log_file_directory
                .to_string()
                .ends_with("custom-logs")
        );
        assert_eq!(cli.logs.log_file_name, "custom.log");
    }

    #[test]
    fn applies_minimal_rpls_pruning() {
        let mut cli = parse_node_cli([
            "rpls",
            "node",
            "--chain",
            "pulsechain",
            "--rpls.minimal-pruning",
        ]);

        apply_minimal_rpls_pruning(&mut cli, PruneFlagSet::default());

        let Commands::Node(command) = cli.command else {
            panic!("expected node command");
        };

        assert!(command.pruning.full);
        assert!(command.pruning.sender_recovery_full);
        assert!(command.pruning.transaction_lookup_full);
        assert_eq!(
            command.pruning.receipts_distance,
            Some(RPLS_MINIMAL_PRUNING_DISTANCE)
        );
        assert_eq!(
            command.pruning.account_history_distance,
            Some(RPLS_MINIMAL_PRUNING_DISTANCE)
        );
        assert_eq!(
            command.pruning.storage_history_distance,
            Some(RPLS_MINIMAL_PRUNING_DISTANCE)
        );
        assert_eq!(
            command.pruning.bodies_distance,
            Some(RPLS_MINIMAL_PRUNING_DISTANCE)
        );
    }

    #[test]
    fn repairs_invalid_minimal_pruning_config_entries() {
        let config = r#"
[prune.segments]
sender_recovery = "full"
transaction_lookup = 'full'
receipts = "full"
account_history = 'full'
storage_history = "full"
bodies_history = 'full'
"#;

        let repaired = repair_minimal_pruning_config_contents(config);
        let distance = format!("{{ distance = {RPLS_MINIMAL_PRUNING_DISTANCE} }}");

        assert!(repaired.contains("sender_recovery = \"full\""));
        assert!(repaired.contains("transaction_lookup = 'full'"));
        assert!(repaired.contains(&format!("receipts = {distance}")));
        assert!(repaired.contains(&format!("account_history = {distance}")));
        assert!(repaired.contains(&format!("storage_history = {distance}")));
        assert!(repaired.contains(&format!("bodies_history = {distance}")));
    }

    #[test]
    fn minimal_rpls_pruning_preserves_explicit_segment_pruning() {
        let mut cli = parse_node_cli([
            "rpls",
            "node",
            "--chain",
            "pulsechain",
            "--rpls.minimal-pruning",
            "--prune.bodies.distance",
            "20000",
            "--prune.receipts.distance",
            "20000",
        ]);
        let prune_flags = PruneFlagSet::from_args([
            "rpls",
            "node",
            "--chain",
            "pulsechain",
            "--rpls.minimal-pruning",
            "--prune.bodies.distance",
            "20000",
            "--prune.receipts.distance",
            "20000",
        ]);

        apply_minimal_rpls_pruning(&mut cli, prune_flags);

        let Commands::Node(command) = cli.command else {
            panic!("expected node command");
        };

        assert!(command.pruning.full);
        assert!(!command.pruning.receipts_full);
        assert_eq!(command.pruning.receipts_distance, Some(20000));
        assert_eq!(command.pruning.bodies_distance, Some(20000));
    }

    fn parse_node_cli<const N: usize>(args: [&str; N]) -> Cli<PulseChainSpecParser, RplsArgs> {
        Cli::<PulseChainSpecParser, RplsArgs>::try_parse_from(args)
            .expect("node CLI args should parse")
    }
}
