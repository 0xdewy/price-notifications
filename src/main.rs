use anyhow::Result;
use colored::Colorize;
use daemonize::Daemonize;
use dialoguer::theme::ColorfulTheme;
use dialoguer::*;
use flexi_logger::{opt_format, FileSpec, Logger};
use log::{error, info};
use tokio::time::{sleep, Duration};

mod config;
use config::Config;
mod price;
use price::{get_currency_ids, prices, PriceDetails};
mod notify;
use notify::Notification;

pub const CONFIG_DIR: &str = ".config/price-notifications";

#[tokio::main]
async fn main() -> Result<()> {
    let home_dir = std::env::home_dir().unwrap();
    let config_dir = home_dir.join(CONFIG_DIR);
    let logs_dir = config_dir.join("logs");
    let config_src = config_dir.join("config.json");
    // Load or setup new config and download supported currencies
    let mut config: Config = match std::fs::read_to_string(config_src.clone()) {
        Ok(file) => serde_json::from_str(&file)?,
        Err(_e) => {
            println!("{}", "Creating new config file".blue());
            println!("creating dir: {:?}", &config_dir);
            std::fs::create_dir_all(&config_dir)?;
            std::fs::create_dir_all(&logs_dir)?;
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

            price::update_supported_currencies().await?;
            c
        }
    };

    Logger::try_with_str("info")?
        .log_to_file(
            FileSpec::default()
                .directory(&logs_dir.clone()) // create files in folder ./log_files
                .basename("logs"),
        )
        .print_message()
        .start()?;

    info!("Starting price-notifications");

    // CLI options
    let items = vec![
        "Get prices",
        "Add Currency",
        "Remove Currency",
        "Add Notification",
        "Listen for notifications",
        "Update supported currencies",
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
            if prices.len() == 0 {
                println!("No currencies added yet!");
            }
            for price in prices {
                println!("{:}", &price);
            }
        }
        // add currency
        1 => {
            // get currency from user prompt
            let currency_string: String = Input::new()
                .with_prompt(
                    "Comma seperated list of currencies: bitcoin, ethereum, dogecoin, monero",
                )
                .interact_text()?;

            // seperate currency_string by commas and trim whitespace annd make it lowercase
            let currencies: Vec<String> = currency_string
                .split(",")
                .map(|s| s.trim().to_lowercase())
                .collect::<Vec<String>>();

            for currency in currencies.iter() {
                let currency_matches = get_currency_ids(&currency)?;
                for curr in currency_matches {
                    if !config.currencies.contains(&curr) {
                        config.currencies.push(curr);
                    } else {
                        println!("{}", "Currency is already added".red())
                    }
                }
            }
        }
        // remove currency
        2 => {
            let selection = Select::with_theme(&ColorfulTheme::default())
                .items(&config.currencies)
                .interact()?;

            if let Some(pos) = config
                .currencies
                .iter()
                .position(|x| *x == config.currencies[selection])
            {
                config.currencies.remove(pos);
            }

            println!("Successfully removed currency");
        }
        // add notification
        3 => {
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
        4 => {
            let current_uid = users::get_current_uid();
            let current_username = users::get_user_by_uid(current_uid)
                .unwrap()
                .name()
                .to_string_lossy()
                .into_owned();

            let current_gid = users::get_current_gid();
            let current_groupname = users::get_group_by_gid(current_gid)
                .unwrap()
                .name()
                .to_string_lossy()
                .into_owned();

            let client = twilio::Client::new(&config.account_id, &config.auth_token);
            let delay: u64 = Input::new()
                .with_prompt("How many seconds to wait between queries?")
                .default(600)
                .interact_text()
                .unwrap();

            let delay = Duration::from_secs(delay);

            info!("starting daemon to listen for price notifications");

            let pid_file = config_dir.join("price_listening_daemon.pid");
            let daemonize = Daemonize::new()
                .pid_file(pid_file) // Now the pid file will be in the CONFIG_DIR.
                .chown_pid_file(true)
                .working_directory(&config_dir) // Changing the working directory to CONFIG_DIR.
                .user(&*current_username)
                .group(&*current_groupname)
                .umask(600);

            match daemonize.start() {
                Ok(_) => info!("Success, daemonized"),
                Err(e) => {
                    println!("Error, {}", e);
                    error!("Error, {}", e);
                    std::process::exit(1);
                }
            }

            let cloned_config = config.clone();
            // Spawn an async task and get its JoinHandle
            let handle = tokio::spawn(async move {
                loop {
                    let mut messages: Vec<String> = Vec::new();
                    let prices = prices(cloned_config.currencies.clone()).await;
                    for price in prices {
                        messages.append(&mut price.get_notifications(&cloned_config));
                    }

                    if messages.len() > 0 {
                        info!("Sending {} messages!", messages.len());
                        notify::send_messages(&cloned_config, &client, messages).await;
                    } else {
                        info!("No messages to send");
                    }

                    // Sleep for the specified duration
                    sleep(delay).await;
                }
            });

            // Wait for the task to finish. This will keep the main process running
            // as long as the task is running.
            if let Err(e) = handle.await {
                error!("Error from listening task: {}", e);
            }
        }

        // Update supported currencies
        5 => {
            price::update_supported_currencies().await?;
        }
        // Show config
        6 => {
            println!("config file: {:?}", &config_src);
            println!("{:#?}", &config);
            return Ok(());
        }
        _ => {
            panic!("Unknown command")
        }
    }

    serde_json::to_writer(&std::fs::File::create(config_src).unwrap(), &config).unwrap();

    Ok(())
}
