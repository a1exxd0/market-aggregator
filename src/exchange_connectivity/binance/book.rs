use super::Binance;
use crate::book_management::{Ask, Bid};
use crate::{
    book_management::{Order, traded_instruments::Instrument},
    exchange_connectivity::{ConnectedExchangeForBook, ExchangeType},
};

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use std::error::Error;

use futures_util::SinkExt;
use serde_json::{Value, json};

impl Binance {
    async fn request_get_order_book(
        &self,
        id: u64,
        instrument_name: &str,
        depth: u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let msg = json!({
            "id": id.to_string(),
            "method": "depth",
            "params": {
                "symbol": instrument_name,
                "limit": depth,
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
                        let price = order_pair[0].as_str()?.parse().ok()?;
                        let qty = order_pair[1].as_str()?.parse().ok()?;
                        Some(T::new(instrument, exchange, qty, price))
                    })
            })
            .collect::<Vec<T>>())
    }
}

impl ConnectedExchangeForBook for Binance {
    fn to_instrument_name(instrument: Instrument) -> String {
        match instrument {
            Instrument::BtcUsdt => "BTCUSDT".to_string(),
            Instrument::EthUsdc => "ETHUSDC".to_string(),
            Instrument::EthBtc => "ETCBTC".to_string(),
        }
    }

    async fn pull_bids_asks(
        &self,
        mut depth: u32,
        instrument: Instrument,
    ) -> Result<(Vec<Bid>, Vec<Ask>, Duration), Box<dyn Error + Send + Sync>> {
        if depth > 5000 {
            depth = 5000;
        }

        let req_id = self.get_new_id();

        self.request_get_order_book(req_id, &Binance::to_instrument_name(instrument), depth)
            .await?;

        for _ in 0..5 {
            let mut multimap = self.non_main_stream.lock().await;
            let multimap_entry = multimap.get(&req_id);
            if let Some(entry) = multimap_entry {
                log::info!("Found msg with id {}: {}", &req_id, entry);
                let msg: serde_json::Value = serde_json::from_str(&entry)
                    .map_err(|err| format!("Failed to parse message: {}", err))?;

                if let Ok(timestamp) = SystemTime::now().duration_since(UNIX_EPOCH) {
                    if let Some(bids) = msg["result"]["bids"].as_array()
                        && let Some(asks) = msg["result"]["asks"].as_array()
                    {
                        let bid_vec = Binance::convert_vec_values_to_orders(
                            ExchangeType::Binance,
                            &bids,
                            instrument,
                        )?;

                        let ask_vec = Binance::convert_vec_values_to_orders(
                            ExchangeType::Binance,
                            &asks,
                            instrument,
                        )?;
                        multimap.remove(&req_id);
                        return Ok((bid_vec, ask_vec, timestamp));
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Err(
            "Did not recieve response for order book from Binance after multiple attempts. "
                .to_string()
                .into(),
        )
    }
}
