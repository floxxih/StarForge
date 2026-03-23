use crate::utils::{config, soroban, print as p};
use anyhow::Result;
use clap::{Args, Subcommand};
use colored::*;

#[derive(Subcommand)]
pub enum ContractCommands {
    /// Invoke a deployed Soroban contract function
    Invoke(InvokeArgs),
}

#[derive(Args)]
pub struct InvokeArgs {
    /// Contract ID to invoke
    pub contract_id: String,
    /// Function name to call
    pub function: String,
    /// Function arguments (use multiple --arg flags)
    #[arg(long = "arg", action = clap::ArgAction::Append)]
    pub args: Vec<String>,
    /// Argument types (use multiple --type flags, must match --arg count)
    #[arg(long = "type", action = clap::ArgAction::Append)]
    pub types: Vec<String>,
    /// Network to use
    #[arg(long, default_value = "testnet", value_parser = ["testnet", "mainnet"])]
    pub network: String,
    /// Wallet name to use for signing (required with --submit)
    #[arg(long)]
    pub wallet: Option<String>,
    /// Submit the transaction after simulation
    #[arg(long, default_value = "false")]
    pub submit: bool,
}

pub fn handle(cmd: ContractCommands) -> Result<()> {
    match cmd {
        ContractCommands::Invoke(args) => handle_invoke(args),
    }
}

fn handle_invoke(args: InvokeArgs) -> Result<()> {
    p::header("Invoke Soroban Contract");

    // Validate arguments and types match
    if args.args.len() != args.types.len() && !args.types.is_empty() {
        anyhow::bail!(
            "Argument count mismatch: {} args but {} types specified",
            args.args.len(),
            args.types.len()
        );
    }

    // Default to string type if no types specified
    let arg_types = if args.types.is_empty() {
        vec!["string".to_string(); args.args.len()]
    } else {
        args.types.clone()
    };

    p::separator();
    p::kv("Contract ID", &args.contract_id);
    p::kv("Function", &args.function);
    p::kv("Network", &args.network);
    
    if !args.args.is_empty() {
        p::kv("Arguments", &format!("{} args", args.args.len()));
        for (i, (arg, arg_type)) in args.args.iter().zip(arg_types.iter()).enumerate() {
            p::kv(&format!("  Arg {}", i + 1), &format!("{} ({})", arg, arg_type));
        }
    } else {
        p::kv("Arguments", "none");
    }

    if args.network == "mainnet" {
        p::warn("You are invoking on MAINNET. This may cost real XLM if submitted.");
    }

    // Load wallet if needed for submission
    let wallet = if args.submit {
        let cfg = config::load()?;
        let wallet = if let Some(ref wallet_name) = args.wallet {
            cfg.wallets
                .iter()
                .find(|w| &w.name == wallet_name)
                .ok_or_else(|| anyhow::anyhow!("Wallet '{}' not found. Run `starforge wallet list`", wallet_name))?
        } else if !cfg.wallets.is_empty() {
            p::info(&format!(
                "No --wallet specified. Using: {}",
                cfg.wallets[0].name.cyan()
            ));
            &cfg.wallets[0]
        } else {
            anyhow::bail!(
                "No wallets found for submission. Create one first:\n  starforge wallet create deployer --fund"
            );
        };
        p::kv("Wallet", &wallet.name);
        Some(wallet.clone())
    } else {
        None
    };

    p::separator();

    // Step 1: Simulate the transaction
    println!();
    p::step(1, if args.submit { 2 } else { 1 }, "Simulating contract invocation…");
    
    let simulation_result = soroban::simulate_transaction(
        &args.contract_id,
        &args.function,
        &args.args,
        &arg_types,
        &args.network,
    )?;

    p::kv_accent("Simulation", "✓ Success");
    p::kv("Return Value", &simulation_result.return_value);
    p::kv("Fee (stroops)", &simulation_result.fee.to_string());
    p::kv("Fee (XLM)", &format!("{:.7}", simulation_result.fee as f64 / 10_000_000.0));

    if !simulation_result.events.is_empty() {
        p::kv("Events", &format!("{} emitted", simulation_result.events.len()));
        for (i, event) in simulation_result.events.iter().enumerate() {
            p::kv(&format!("  Event {}", i + 1), event);
        }
    }

    // Step 2: Submit if requested
    if args.submit {
        if let Some(wallet) = wallet {
            println!();
            p::step(2, 2, "Submitting transaction…");
            
            let tx_result = soroban::submit_transaction(
                &args.contract_id,
                &args.function,
                &args.args,
                &arg_types,
                &args.network,
                &wallet,
            )?;

            p::kv_accent("Transaction", "✓ Submitted");
            p::kv("TX Hash", &tx_result.hash);
            p::kv("Return Value", &tx_result.return_value);
        }
    } else {
        println!();
        p::info("Simulation complete. Add --submit to execute the transaction.");
    }

    p::separator();
    Ok(())
}