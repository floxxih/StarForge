mod commands;
mod utils;

use clap::{Parser, Subcommand};
use colored::*;

#[derive(Parser)]
#[command(
    name = "starforge",
    about = "⚡ Stellar & Soroban developer productivity CLI",
    long_about = "starforge is an open-source CLI toolkit for developers building on the Stellar network.\nManage wallets, deploy Soroban contracts, and scaffold new projects — all from your terminal.",
    version = "0.1.0",
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Suppress the ASCII banner and decorative output
    #[arg(long, short = 'q', global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage test wallets (create, list, fund, show, remove)
    #[command(subcommand)]
    Wallet(commands::wallet::WalletCommands),
    /// Generate Soroban project boilerplate
    #[command(subcommand)]
    New(commands::new::NewCommands),
    /// Contract operations (invoke, etc.)
    #[command(subcommand)]
    Contract(commands::contract::ContractCommands),
    /// Deploy a compiled Soroban contract (.wasm)
    Deploy(commands::deploy::DeployArgs),
    /// Show starforge config and environment info
    Info,

    Tx(commands::tx::TxArgs),   // fetch transaction for the account

    /// View or switch the active network (testnet/mainnet)
    #[command(subcommand)]
    Network(commands::network::NetworkCommands),
    /// Generate shell completions for bash, zsh, and fish
    Completions(commands::completions::CompletionShell),
}

fn main() {
    let cli = Cli::parse();

    if !cli.quiet {
        print_banner();
    }

    let result = match cli.command {
        Commands::Wallet(cmd)  => commands::wallet::handle(cmd),
        Commands::New(cmd)     => commands::new::handle(cmd),
        Commands::Contract(cmd) => commands::contract::handle(cmd),
        Commands::Deploy(args) => commands::deploy::handle(args),
        Commands::Info         => commands::info::handle(),
        Commands::Tx(args) => commands::tx::handle(args),
        Commands::Network(cmd) => commands::network::handle(cmd),
        Commands::Completions(shell) => commands::completions::handle(shell),
    };

    if let Err(e) = result {
        eprintln!("\n  {} {}\n", "✗ Error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn print_banner() {
    println!(
        "{}",
        "\n  ███████╗████████╗ █████╗ ██████╗ ███████╗ ██████╗ ██████╗  ██████╗ ███████╗\n  ██╔════╝╚══██╔══╝██╔══██╗██╔══██╗██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔════╝\n  ███████╗   ██║   ███████║██████╔╝█████╗  ██║   ██║██████╔╝██║  ███╗█████╗  \n  ╚════██║   ██║   ██╔══██║██╔══██╗██╔══╝  ██║   ██║██╔══██╗██║   ██║██╔══╝  \n  ███████║   ██║   ██║  ██║██║  ██║██║     ╚██████╔╝██║  ██║╚██████╔╝███████╗\n  ╚══════╝   ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝      ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝\n"
        .cyan().bold()
    );
    println!(
        "  {} {}\n",
        "⚡ Stellar & Soroban Developer CLI".bright_white(),
        "v0.1.0".dimmed()
    );
}
