use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub instruments: HashMap<u64, OrderBookConfig>,
    pub incremental_buffer_size: usize,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct OrderBookConfig {
    pub id: u64,
    pub min_price: f64,
    pub max_price: f64,
    pub tick_size: f64,
}
