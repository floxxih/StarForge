use crate::utils::{config, print as p};
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum NetworkCommands {
    /// Show the current active network
    Show,
    /// Switch the active network (testnet or mainnet)
    Switch {
        /// Target network to switch to
        #[arg(value_parser = ["testnet", "mainnet"])]
        network: String,
    },
}

pub fn handle(cmd: NetworkCommands) -> Result<()> {
    match cmd {
        NetworkCommands::Show => show(),
        NetworkCommands::Switch { network } => switch(network),
    }
}

fn show() -> Result<()> {
    let cfg = config::load()?;
    p::info(&format!("Active network: {}", cfg.network));
    Ok(())
}

fn switch(target: String) -> Result<()> {
    let mut cfg = config::load()?;

    // Check if already on the target network
    if cfg.network == target {
        p::info(&format!("Already on {}. No changes made.", target));
        return Ok(());
    }

    let previous = cfg.network.clone();
    cfg.network = target.clone();
    config::save(&cfg)?;

    // Print mainnet warning
    if target == "mainnet" {
        p::warn("You are now on MAINNET. Transactions use real funds!");
        p::warn("Double-check all addresses and amounts before sending.");
    }

    p::success(&format!(
        "Network switched from {} to {}.",
        previous, target
    ));

    Ok(())
}
