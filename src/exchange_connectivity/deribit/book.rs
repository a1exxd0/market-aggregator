//! Order book related bits
use crate::book_management::{Ask, Bid, Order};
use crate::exchange_connectivity::{ConnectedExchangeForBook, ExchangeType, Instrument};

use super::Deribit;
use serde_json::{Value, json};

use std::error::Error;

use futures_util::SinkExt;
use std::time::Duration;

#[derive(Clone, Copy)]
pub enum ValidOrderDepth {
    One = 1,
    Five = 5,
    Ten = 10,
    Twenty = 20,
    Fifty = 50,
    OneHundered = 100,
    OneThousand = 1000,
    TenThousand = 10000,
}

impl ValidOrderDepth {
    pub fn from_number(value: u32) -> Self {
        match value {
            n if n >= 10000 => ValidOrderDepth::TenThousand,
            n if n >= 1000 => ValidOrderDepth::OneThousand,
            n if n >= 100 => ValidOrderDepth::OneHundered,
            n if n >= 50 => ValidOrderDepth::Fifty,
            n if n >= 20 => ValidOrderDepth::Twenty,
            n if n >= 10 => ValidOrderDepth::Ten,
            n if n >= 5 => ValidOrderDepth::Five,
            _ => ValidOrderDepth::One,
        }
    }
}

impl Deribit {
    async fn request_get_order_book(
        &self,
        id: u64,
        instrument_name: &str,
        depth: ValidOrderDepth,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "public/get_order_book",
            "params": {
                "instrument_name": instrument_name,
                "depth": depth as u32,
            },
        });

        self.sink.lock().await.send(msg.to_string().into()).await?;

        Ok(())
    }

    fn convert_vec_values_to_orders<T: Order>(
        exchange: ExchangeType,
        vec: &[Value],
        instrument: Instrument,
    ) -> Result<Vec<T>, Box<dyn Error + Send + Sync>> {
        Ok(vec
            .iter()
            .filter_map(|elem| {
                elem.as_array()
                    .filter(|order_pair| order_pair.len() == 2)
                    .and_then(|order_pair| {
                        let price = order_pair[0].as_f64()?;
                        let qty = order_pair[1].as_f64()?;
                        Some(T::new(instrument, exchange, qty, price))
                    })
            })
            .collect::<Vec<T>>())
    }
}

impl ConnectedExchangeForBook for Deribit {
    async fn pull_bids_asks(
        &self,
        depth: u32,
        instrument: Instrument,
    ) -> Result<(Vec<Bid>, Vec<Ask>, Duration), Box<dyn Error + Send + Sync>> {
        let depth = ValidOrderDepth::from_number(depth);
        let req_id = self.get_new_id();

        self.request_get_order_book(req_id, &Deribit::to_instrument_name(instrument), depth)
            .await?;

        for _ in 0..5 {
            let mut multimap = self.non_main_stream.lock().await;
            let multimap_entry = multimap.get(&req_id);
            if let Some(entry) = multimap_entry {
                log::info!("Found depth msg with id {}: {}", &req_id, entry);
                let msg: serde_json::Value = serde_json::from_str(&entry)
                    .map_err(|err| format!("Failed to parse message: {}", err))?;

                if let Some(timestamp) = msg["result"]["timestamp"].as_u64() {
                    if let Some(bids) = msg["result"]["bids"].as_array()
                        && let Some(asks) = msg["result"]["asks"].as_array()
                    {
                        let bid_vec = Deribit::convert_vec_values_to_orders(
                            ExchangeType::Deribit,
                            &bids,
                            instrument,
                        )?;

                        let ask_vec = Deribit::convert_vec_values_to_orders(
                            ExchangeType::Deribit,
                            &asks,
                            instrument,
                        )?;
                        multimap.remove(&req_id);
                        return Ok((bid_vec, ask_vec, Duration::from_millis(timestamp)));
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Err(
            "Did not recieve response for order book from Deribit after multiple attempts. "
                .to_string()
                .into(),
        )
    }

    fn to_instrument_name(instrument: Instrument) -> String {
        match instrument {
            Instrument::BtcUsdt => "BTC_USDT".to_string(),
            Instrument::EthUsdc => "ETH_USDC".to_string(),
            Instrument::EthBtc => "ETH_BTC".to_string(),
        }
    }
}
