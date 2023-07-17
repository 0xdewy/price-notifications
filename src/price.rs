use anyhow::*;
use coingecko::{response::coins::CoinsListItem, CoinGeckoClient};
use colored::Colorize;
use serde::{Deserialize, Serialize};

use super::CONFIG_DIR;

pub const SUPPORTED_CURRENCIES_SRC: &str = "coingecko_supported_coins.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct SupportedCurrencies {
    id: String,
    symbol: String,
    name: String,
}

impl From<CoinsListItem> for SupportedCurrencies {
    fn from(item: CoinsListItem) -> Self {
        Self {
            id: item.id,
            symbol: item.symbol,
            name: item.name,
        }
    }
}

fn get_supported_currencies() -> Vec<SupportedCurrencies> {
    let config_dir = std::env::home_dir().unwrap().join(CONFIG_DIR);
    let supported_currency_dir = config_dir
        .join(SUPPORTED_CURRENCIES_SRC)
        .to_str()
        .expect("supported currency dir isn't configured properly")
        .to_string();
    let supported_currencies = std::fs::read_to_string(supported_currency_dir)
        .expect("failed to read supported currencies file");
    let supported_currencies: Vec<SupportedCurrencies> =
        serde_json::from_str(&supported_currencies)
            .expect("failed to parse supported currencies file");
    return supported_currencies;
}

pub async fn update_supported_currencies() -> Result<(), Error> {
    let client = CoinGeckoClient::default();
    let supported_currencies: Vec<CoinsListItem> = client.coins_list(false).await?;
    let supported_currencies: Vec<SupportedCurrencies> = supported_currencies
        .into_iter()
        .map(|item| SupportedCurrencies::from(item))
        .collect();
    let json = serde_json::to_string(&supported_currencies).unwrap();

    let config_dir = std::env::home_dir().unwrap().join(CONFIG_DIR);
    let output_dir = config_dir.join(SUPPORTED_CURRENCIES_SRC);

    println!("Writing to {}", output_dir.to_str().unwrap());
    std::fs::write(
        output_dir
            .to_str()
            .expect("failed to create supported currencies file"),
        json,
    )
    .unwrap();
    Ok(())
}

pub fn get_currency_ids(name_or_symbol: &str) -> Result<Vec<String>, Error> {
    let supported_currencies = get_supported_currencies();
    let mut matches: Vec<String> = Vec::new();
    for supported_currency in supported_currencies {
        if supported_currency.id.to_lowercase() != name_or_symbol.to_lowercase()
            && supported_currency.name.to_lowercase() != name_or_symbol.to_lowercase()
            && supported_currency.symbol.to_lowercase() != name_or_symbol.to_lowercase()
        {
            continue;
        }
        matches.push(supported_currency.id);
    }
    if matches.len() == 0 {
        return Err(anyhow!(format!(
            "Not a supported currency: {}",
            name_or_symbol.to_string()
        )));
    }

    return Ok(matches);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceDetails {
    pub currency: String,
    pub price: f64,
}

impl std::fmt::Display for PriceDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}: {:.8} USD",
            self.currency.blue(),
            self.price.to_string().green()
        )
    }
}

// TODO: use config default price
pub async fn prices(currencies: Vec<String>) -> Vec<PriceDetails> {
    let client = coingecko::CoinGeckoClient::default();
    let mut results: Vec<PriceDetails> = Vec::new();
    let prices = client
        .price(currencies.as_slice(), &["usd"], true, false, false, true)
        .await
        .unwrap();
    for currency in currencies {
        let price = {
            match prices.get(&currency) {
                Some(prices) => prices.usd.expect("no usd"),
                None => {
                    println!(
                        "{}. {}. {}",
                        "Failed to get the price of".red(),
                        &currency.blue(),
                        "Coingecko may not support the full name of this currency, try the symbol"
                            .red()
                    );
                    continue;
                }
            }
        };

        results.push(PriceDetails { currency, price });
    }
    results
}
