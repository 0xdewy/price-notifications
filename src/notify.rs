use super::config::Config;
use super::price::PriceDetails;
use twilio::{Client, OutboundMessage};

pub trait Notification {
    fn add_notifications(&self, config: &mut Config, max: Option<u32>, min: Option<u32>);
    fn get_notifications(&self, config: &Config) -> Vec<String>;
}

impl Notification for PriceDetails {
    fn add_notifications(&self, config: &mut Config, max: Option<u32>, min: Option<u32>) {
        if !config.currencies.contains(&self.currency) {
            panic!("Currency not added")
        }
        match max {
            Some(max) => {
                config
                    .notify_above
                    .insert(self.currency.clone(), f64::from(max));
            }
            None => (),
        }

        match min {
            Some(min) => {
                config
                    .notify_below
                    .insert(self.currency.clone(), f64::from(min));
            }
            None => (),
        }
    }

    fn get_notifications(&self, config: &Config) -> Vec<String> {
        let mut messages: Vec<String> = Vec::new();

        if !config.currencies.contains(&self.currency) {
            panic!("Currency not added")
        }

        match config.notify_below.get(&self.currency) {
            Some(target) => {
                if &self.price < target {
                    let message =
                        format!("{} target price dropped below: {}", self.currency, target);
                    messages.push(message);
                }
            }
            None => (),
        }

        match config.notify_above.get(&self.currency) {
            Some(target) => {
                if &self.price > target {
                    let message = format!("{} target price went above: {}", self.currency, target);
                    messages.push(message);
                }
            }
            None => (),
        }
        messages
    }
}

pub async fn send_messages(config: &Config, twilio_client: &Client, messages: Vec<String>) {
    for message in messages {
        // Send message
        match twilio_client
            .send_message(OutboundMessage::new(
                &config.my_number,
                &config.to_number,
                &message,
            ))
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
}
