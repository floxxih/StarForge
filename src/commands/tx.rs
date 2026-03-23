use anyhow::Result;
use clap::{Args, Subcommand};
use colored::*;

use crate::utils::{config, horizon};

#[derive(Args)]
pub struct TxArgs {
    #[command(subcommand)]
    pub command: TxCommands,
}

#[derive(Subcommand)]
pub enum TxCommands {
    // fetch and display recent transactions for a Stellar account
    History {
        public_key: String,

        /// number of transactions
        #[arg(short, long, default_value_t = 10)]
        limit: u8,

        #[arg(short, long)]
        network: Option<String>,
    },
}

pub fn handle(args: TxArgs) -> Result<()> {
    match args.command {
        TxCommands::History {
            public_key,
            limit,
            network,
        } => handle_history(public_key, limit, network),
    }
}

fn handle_history(public_key: String, limit: u8, network_override: Option<String>) -> Result<()> {
    let limit = limit.min(50);

    let network = network_override.unwrap_or_else(|| {
        config::load()
            .map(|c| c.network)
            .unwrap_or_else(|_| "testnet".to_string())
    });

    println!();
    println!("  {} {}", "◆".cyan().bold(), "Transaction History".white().bold());
    println!("  {} {}", "Account :".dimmed(), public_key.yellow());
    println!("  {} {}", "Network :".dimmed(), network.cyan());
    println!("  {} {}", "Showing :".dimmed(), format!("last {} txs", limit).white());
    println!("  {}", "─".repeat(72).dimmed());

    match horizon::fetch_transactions(&public_key, &network, limit) {
        Err(e) => {
            println!("\n  {} {}\n", "✗".red().bold(), e.to_string().red());
        }
        Ok(txs) if txs.is_empty() => {
            println!("\n  {} No transactions found for this account.\n", "!".yellow().bold());
        }
        Ok(txs) => {
            print_transactions(&txs, &network);
        }
    }

    Ok(())
}

fn print_transactions(txs: &[horizon::TransactionRecord], network: &str) {
    
    println!(
        "  {:<14}  {:<6}  {:<4}  {:<12}  {}",
        "Hash".dimmed(),
        "Status".dimmed(),
        "Ops".dimmed(),
        "Fee (XLM)".dimmed(),
        "Timestamp (UTC)".dimmed(),
    );
    println!("  {}", "─".repeat(72).dimmed());

    for tx in txs {
        let short_hash = format!("{}…", &tx.hash[..12]);

        let status = if tx.successful {
            "✓ ok".green().to_string()
        } else {
            "✗ fail".red().to_string()
        };

        // returns fee in stroops; 1 XLM is equal to 10_000_000 stroops
        let fee_xlm = tx
            .fee_charged
            .parse::<f64>()
            .map(|s| format!("{:.7}", s / 10_000_000.0))
            .unwrap_or_else(|_| tx.fee_charged.clone());

        let ts = tx
            .created_at
            .replace('T', " ")
            .get(..16)
            .unwrap_or(&tx.created_at)
            .to_string();

        println!(
            "  {:<14}  {:<6}  {:<4}  {:<12}  {}",
            short_hash.white(),
            status,
            tx.operation_count.to_string().white(),
            fee_xlm.yellow(),
            ts.dimmed(),
        );
    }

    println!("  {}", "─".repeat(72).dimmed());

    // footer === Stellar Expert deep link
    let explorer_base = if network == "mainnet" {
        "https://stellar.expert/explorer/public/tx"
    } else {
        "https://stellar.expert/explorer/testnet/tx"
    };

    if let Some(first) = txs.first() {
        println!(
            "\n  {} {}/{}\n",
            "🔗 Latest tx:".dimmed(),
            explorer_base,
            first.hash.cyan()
        );
    }
}