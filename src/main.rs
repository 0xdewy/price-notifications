use clap::Parser;
use coingecko::SimplePriceReq;
use env_file_reader::read_file;
use isahc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use twilio::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[structopt(short, long)]
    add_currency: Option<String>,

    #[structopt(short, long)]
    notify: Option<String>,

    #[structopt(long)]
    min: Option<i64>,

    #[structopt(long)]
    max: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    currencies: Vec<String>,
    priced_in: String,
    notify_above: HashMap<String, Decimal>,
    notify_below: HashMap<String, Decimal>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            currencies: Vec::new(),
            priced_in: String::from("usd"),
            notify_above: HashMap::new(),
            notify_below: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceDetails {
    currency: String,
    price: Decimal,
}

impl std::fmt::Display for PriceDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: ({})", self.currency, self.price)
    }
}

pub async fn prices(currencies: Vec<String>) -> Vec<PriceDetails> {
    let http = isahc::HttpClient::new().unwrap();
    let client = coingecko::Client::new(http);
    let mut results: Vec<PriceDetails> = Vec::new();
    for currency in currencies {
        let price = {
            let req =
                SimplePriceReq::new(currency.clone().into(), "usd".into()).include_market_cap();
            let price = client.simple_price(req).await.unwrap();
            price
                .get(&currency)
                .expect("")
                .get("usd")
                .expect("")
                .clone()
        };
        results.push(PriceDetails { currency, price });
    }
    results
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let my_number = "+14254751977";
    let to = "+41798458229";

    let home_dir = std::env::home_dir().unwrap();
    let config_dir = home_dir.join(".prices/");
    let config_src = config_dir.join("config");

    let env_variables = read_file(".env")?;

    let mut config: Config = match std::fs::read_to_string(config_src.clone()) {
        Ok(file) => serde_json::from_str(&file)?,
        Err(e) => {
            println!("Creating new config file");
            println!("creating dir: {:?}", config_dir);
            std::fs::create_dir_all(config_dir)?;
            Config::default()
        }
    };

    let args = Cli::parse();

    match args.add_currency {
        Some(c) => {
            if !config.currencies.contains(&c) {
                config.currencies.push(c);
            }
        }
        None => (),
    }

    // Add notification?
    match args.notify {
        Some(currency) => {
            if !config.currencies.contains(&currency) {
                panic!("Currency not added")
            }
            match args.max {
                Some(max) => {
                    config
                        .notify_above
                        .insert(currency.clone(), Decimal::from(max));
                }
                None => (),
            }

            match args.min {
                Some(min) => {
                    config.notify_below.insert(currency, Decimal::from(min));
                }
                None => (),
            }
        }
        None => (),
    }

    serde_json::to_writer(&std::fs::File::create(config_src).unwrap(), &config).unwrap();

    let prices = prices(config.currencies.clone()).await;

    let mut messages: Vec<String> = Vec::new();
    let client = twilio::Client::new(&env_variables["ACCOUNT_ID"], &env_variables["AUTH_TOKEN"]);

    for price in prices {
        println!("{:?}", price);
        if !config.currencies.contains(&price.currency) {
            panic!("Currency not added")
        }

        match config.notify_below.get(&price.currency) {
            Some(target) => {
                if &price.price < target {
                    let message =
                        format!("{} target price dropped below: {}", price.currency, target);
                    messages.push(message);
                }
            }
            None => (),
        }

        match config.notify_above.get(&price.currency) {
            Some(target) => {
                if &price.price > target {
                    let message = format!("{} target price went above: {}", price.currency, target);
                    messages.push(message);
                }
            }
            None => (),
        }
    }

    for message in messages {
        // Send message
        match client
            .send_message(OutboundMessage::new(my_number, to, &message))
            .await
        {
            Ok(m) => {
                println!("Message: {:?}", m);
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        } //
    }
    Ok(())
}
