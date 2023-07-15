#[derive(Debug, Serialize, Deserialize)]
pub struct SupportedCurrencies {
    pub ids: Vec<String>,
    pub symbols: Vec<String>,
    pub names: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceDetails {
    pub currencies: Vec<String>,
    pub prices: Vec<f64>,
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
