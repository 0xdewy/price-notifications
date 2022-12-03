use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use coingecko::SimplePriceReq;

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceDetails {
    pub currency: String,
    pub price: Decimal,
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
            let req = SimplePriceReq::new(currency.clone(), "usd".into()).include_market_cap();
            let price = client.simple_price(req).await.unwrap();
            *price.get(&currency).expect("").get("usd").expect("")
        };
        results.push(PriceDetails { currency, price });
    }
    results
}