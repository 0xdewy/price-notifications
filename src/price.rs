use anyhow::*;
use coingecko::CoinGeckoClient;
use colored::Colorize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SupportedCurrencies {
    id: String,
    symbol: String,
    name: String,
}
pub trait SupportedCurrenciesTrait {
    fn get_supported_currencies() -> Vec<SupportedCurrencies> {
        // TODO: query coingecko and save or use local file?
        let supported_currencies: Vec<SupportedCurrencies> =
            serde_json::from_str(include_str!("../data/coingecko_supported_coins.json")).unwrap();
        return supported_currencies;
    }

    fn full_name(&self) -> Result<String, Error>;

    fn is_supported_currency(&self) -> bool;
}

impl SupportedCurrenciesTrait for String {
    fn is_supported_currency(&self) -> bool {
        let supported_currencies = Self::get_supported_currencies();
        for supported_currency in supported_currencies {
            if supported_currency.id.to_lowercase() == self.to_lowercase() {
                return true;
            }
            if supported_currency.symbol.to_lowercase() == self.to_lowercase() {
                return true;
            }
            if supported_currency.name.to_lowercase() == self.to_lowercase() {
                return true;
            }
        }
        false
    }

    fn full_name(&self) -> Result<String, Error> {
        let supported_currencies = Self::get_supported_currencies();
        for supported_currency in supported_currencies {
            if supported_currency.id.to_lowercase() == self.to_lowercase() {
                return Ok(supported_currency.name.to_lowercase());
            }
            if supported_currency.symbol.to_lowercase() == self.to_lowercase() {
                return Ok(supported_currency.name.to_lowercase());
            }
            if supported_currency.name.to_lowercase() == self.to_lowercase() {
                return Ok(supported_currency.name.to_lowercase());
            }
        }
        Err(anyhow!(format!(
            "Not a supported currency: {}",
            self.to_string()
        )))
    }
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