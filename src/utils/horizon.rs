use anyhow::{Result, Context};
use serde::Deserialize;

pub fn horizon_url(network: &str) -> &'static str {
    match network {
        "mainnet" => "https://horizon.stellar.org",
        _ => "https://horizon-testnet.stellar.org",
    }
}

#[derive(Debug, Deserialize)]
pub struct AccountResponse {
    pub id: String,
    pub sequence: String,
    pub balances: Vec<Balance>,
    pub subentry_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct Balance {
    pub balance: String,
    pub asset_type: String,
    pub asset_code: Option<String>,
}

pub fn fund_account(public_key: &str) -> Result<()> {
    let url = format!("https://friendbot.stellar.org?addr={}", public_key);
    let res = ureq::get(&url).call()
        .with_context(|| "Friendbot request failed")?;
    if res.status() == 200 {
        Ok(())
    } else {
        anyhow::bail!("Friendbot returned status {}", res.status())
    }
}

pub fn fetch_account(public_key: &str, network: &str) -> Result<AccountResponse> {
    let url = format!("{}/accounts/{}", horizon_url(network), public_key);
    let res = ureq::get(&url).call()
        .with_context(|| format!("Failed to reach Horizon on {}", network))?;
    if res.status() == 200 {
        let account: AccountResponse = res.into_json()
            .with_context(|| "Failed to parse account response")?;
        Ok(account)
    } else {
        anyhow::bail!("Account not found on {}", network)
    }
}

pub fn check_network(network: &str) -> bool {
    let url = format!("{}/", horizon_url(network));
    ureq::get(&url).call().map(|r| r.status() == 200).unwrap_or(false)
}

#[derive(Debug, Deserialize)]
pub struct TransactionRecord {
    pub hash: String,
    pub successful: bool,
    pub operation_count: u32,
    pub fee_charged: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
struct TransactionsResponse {
    #[serde(rename = "_embedded")]
    embedded: TransactionsEmbedded,
}

#[derive(Debug, Deserialize)]
struct TransactionsEmbedded {
    records: Vec<TransactionRecord>,
}

pub fn fetch_transactions(
    public_key: &str,
    network: &str,
    limit: u8,
) -> Result<Vec<TransactionRecord>> {
    let url = format!(
        "{}/accounts/{}/transactions?order=desc&limit={}",
        horizon_url(network),
        public_key,
        limit
    );

    let res = ureq::get(&url).call().with_context(|| {
        format!(
            "Account '{}' not found on {}. Has it been funded?",
            public_key, network
        )
    })?;

    let parsed: TransactionsResponse = res
        .into_json()
        .with_context(|| "Failed to parse transactions response")?;

    Ok(parsed.embedded.records)
}