use dialoguer::theme::ColorfulTheme;
use dialoguer::*;
use anyhow::Result;

mod config;
use config::Config;

mod price;
use price::{prices, PriceDetails, SupportedCurrenciesTrait};

mod notify;
use notify::Notification;

#[tokio::main]
async fn main() -> Result<()> {
    let home_dir = std::env::home_dir().unwrap();
    let config_dir = home_dir.join(".prices/");
    let config_src = config_dir.join("config.json");

    // Load or setup new config
    let mut config: Config = match std::fs::read_to_string(config_src.clone()) {
        Ok(file) => serde_json::from_str(&file)?,
        Err(_e) => {
            println!("Creating new config file");
            println!("creating dir: {:?}", config_dir);
            std::fs::create_dir_all(config_dir)?;
            let mut c = Config::default();
            c.my_number = Input::new()
                .with_prompt("Twilio number from")
                .interact_text()?;
            c.to_number = Input::new()
                .with_prompt("Twilio number to")
                .interact_text()?;
            c.account_id = Input::new()
                .with_prompt("Twilio account id")
                .interact_text()?;
            c.auth_token = Password::new()
                .with_prompt("Twilio auth token")
                .interact()?;

            serde_json::to_writer(&std::fs::File::create(&config_src).unwrap(), &c).unwrap();
            println!("Successfully created new config file!");
            c
        }
    };

    // CLI options
    let items = vec![
        "Get prices",
        "Add Currency",
        "Add Notification",
        "Listen for notifications",
        "Show config",
    ];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()?;

    // Parse command and execute
    match selection {
        // get prices
        0 => {
            let prices = prices(config.currencies.clone()).await;
            for price in prices {
                println!("{:?}", &price);
            }
        }
        // add currency
        1 => {
            // get currency from user prompt
            let currency_string: String = Input::new()
                .with_prompt("Full name of currencies: bitcoin, ethereum, dogecoin \n Or else pass the symbol: btc, eth, doge")
                .interact_text()?;

            // seperate currency_string by commas and trim whitespace annd make it lowercase
            let currencies: Vec<String> = currency_string
                .split(",")
                .map(|s| s.trim().to_lowercase())
                .collect::<Vec<String>>();

            for currency in currencies.iter() {
                let currency = currency.full_name()?;
                if !config.currencies.contains(&currency) {
                    config.currencies.push(currency);
                } else {
                    println!("Currency is already added")
                }
            }
        }
        // add notification
        2 => {
            let selection = Select::with_theme(&ColorfulTheme::default())
                .items(&config.currencies)
                .interact()?;
            let currency = config
                .currencies
                .get(selection)
                .expect("Currency isnt added");

            let pricing = PriceDetails {
                currency: currency.clone(),
                price: 0.0,
            };

            let max: u32 = Input::new()
                .with_prompt("What is the notification price when it goes above")
                .interact_text()?;
            let max = if max == 0 { None } else { Some(max) };

            let min: u32 = Input::new()
                .with_prompt("What is the notification price when it goes below")
                .interact_text()?;
            let min = if min == 0 { None } else { Some(min) };

            pricing.add_notifications(&mut config, max, min)
        }
        // Listen for notifications
        3 => {
            let client = twilio::Client::new(&config.account_id, &config.auth_token);
            let delay: u64 = Input::new()
                .with_prompt("How many seconds to wait between queries?")
                .default(600)
                .interact_text()?;
            // Send all messages and then sleep for inputted number of seconds
            loop {
                let mut messages: Vec<String> = Vec::new();
                let prices = prices(config.currencies.clone()).await;
                for price in prices {
                    messages.append(&mut price.get_notifications(&config));
                }

                if messages.len() > 0 {
                    println!("Sending {} messages!", messages.len());
                    notify::send_messages(&config, &client, messages).await;
                }

                let seconds = std::time::Duration::from_secs(delay);
                std::thread::sleep(seconds);
            }
        }
        // Show config
        4 => {
            // TODO: implement display
            println!("{}", &config);
            return Ok(());
        }
        _ => {
            panic!("Unknown command")
        }
    }

    println!("{:?}", &config);
    serde_json::to_writer(&std::fs::File::create(config_src).unwrap(), &config).unwrap();

    Ok(())
}
