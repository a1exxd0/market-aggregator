pub mod traded_instruments;

use tokio::sync::Mutex;
use traded_instruments::Instrument;

use crate::exchange_connectivity::{ConnectedExchangeForBook, Exchange, ExchangeType};

use std::{cmp::Ordering, collections::BTreeSet, sync::Arc};

pub struct AggregatedOrderBook {
    instrument: Instrument,
    subscriptions: Vec<Exchange>,
    bids: Arc<Mutex<BTreeSet<Bid>>>,
    asks: Arc<Mutex<BTreeSet<Ask>>>,
}

impl AggregatedOrderBook {
    pub fn new(instrument: Instrument, subscriptions: &Vec<Exchange>) -> Self {
        let mut new_subs = Vec::new();
        for subscription in subscriptions {
            new_subs.push(subscription.clone());
        }

        AggregatedOrderBook {
            instrument: instrument,
            subscriptions: new_subs,
            bids: Arc::new(Mutex::new(BTreeSet::new())),
            asks: Arc::new(Mutex::new(BTreeSet::new())),
        }
    }

    pub async fn update_state(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bids = self.bids.lock().await;
        let mut asks = self.asks.lock().await;
        *bids = BTreeSet::new();
        *asks = BTreeSet::new();

        for _sub in &self.subscriptions {
            match _sub {
                Exchange::Binance(binance) => {
                    let binance = Arc::clone(binance);

                    let (new_bids, new_asks, _) =
                        binance.pull_bids_asks(10, self.instrument).await?;

                    for bid in new_bids {
                        bids.insert(bid);
                    }

                    for ask in new_asks {
                        asks.insert(ask);
                    }
                }
                Exchange::Deribit(deribit) => {
                    let deribit = Arc::clone(deribit);

                    let (new_bids, new_asks, _) =
                        deribit.pull_bids_asks(10, self.instrument).await?;

                    for bid in new_bids {
                        bids.insert(bid);
                    }

                    for ask in new_asks {
                        asks.insert(ask);
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn pretty_print(&self) -> Result<(), Box<dyn std::error::Error>> {
        let bids = self.bids.lock().await;
        let asks = self.asks.lock().await;

        println!("Instrument: {}\n", self.instrument);

        println!("{:<20} {:<20}", "Bids", "Asks");
        println!("{:-<40}", "");

        let max_rows = std::cmp::max(bids.len(), asks.len());
        let mut bids_iter = bids.iter();
        let mut asks_iter = asks.iter();

        for _ in 0..max_rows {
            let bid = bids_iter
                .next()
                .map_or("".to_string(), |b| format!("{:?}", b));
            let ask = asks_iter
                .next()
                .map_or("".to_string(), |a| format!("{:?}", a));
            println!("{:<20} {:<20}", bid, ask);
        }

        Ok(())
    }

    pub async fn imbalance(&self) -> f64 {
        let mut bid_total_qty: f64 = 0.0;
        let mut ask_total_qty: f64 = 0.0;

        for bid in &*self.bids.lock().await {
            bid_total_qty += bid.price;
        }

        for ask in &*self.asks.lock().await {
            ask_total_qty += ask.price;
        }

        bid_total_qty / ask_total_qty
    }
}

pub trait Order {
    fn new(instrument: Instrument, exchange: ExchangeType, quantity: f64, price: f64) -> Self;
    fn quantity(&self) -> f64;
    fn price(&self) -> f64;
    fn instrument(&self) -> Instrument;
    fn exchange(&self) -> ExchangeType;
}

#[derive(Debug)]
pub struct Bid {
    instrument: Instrument,
    exchange: ExchangeType,
    quantity: f64,
    price: f64,
}

impl PartialEq for Bid {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price
    }
}

impl Eq for Bid {}

impl PartialOrd for Bid {
    fn ge(&self, other: &Self) -> bool {
        self.price >= other.price
    }

    fn gt(&self, other: &Self) -> bool {
        self.price > other.price
    }

    fn lt(&self, other: &Self) -> bool {
        self.price < other.price
    }

    fn le(&self, other: &Self) -> bool {
        self.price <= other.price
    }

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.price - other.price {
            x if x < 0.0 => Some(Ordering::Less),
            x if x == 0.0 => Some(Ordering::Equal),
            x if x > 0.0 => Some(Ordering::Greater),
            _ => None,
        }
    }
}

impl Ord for Bid {
    fn cmp(&self, other: &Self) -> Ordering {
        self.price
            .partial_cmp(&other.price)
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                self.quantity
                    .partial_cmp(&other.quantity)
                    .unwrap_or(Ordering::Equal)
            })
    }
}

impl Order for Bid {
    fn new(instrument: Instrument, exchange: ExchangeType, quantity: f64, price: f64) -> Self {
        Bid {
            instrument: instrument,
            exchange: exchange,
            quantity: quantity,
            price: price,
        }
    }

    fn quantity(&self) -> f64 {
        self.quantity
    }

    fn price(&self) -> f64 {
        self.price
    }

    fn instrument(&self) -> Instrument {
        self.instrument
    }

    fn exchange(&self) -> ExchangeType {
        self.exchange
    }
}

#[derive(Debug)]
pub struct Ask {
    quantity: f64,
    price: f64,
    instrument: Instrument,
    exchange: ExchangeType,
}

impl PartialEq for Ask {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price
    }
}

impl Eq for Ask {}

impl PartialOrd for Ask {
    fn ge(&self, other: &Self) -> bool {
        self.price <= other.price
    }

    fn gt(&self, other: &Self) -> bool {
        self.price < other.price
    }

    fn lt(&self, other: &Self) -> bool {
        self.price > other.price
    }

    fn le(&self, other: &Self) -> bool {
        self.price >= other.price
    }

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.price - other.price {
            x if x < 0.0 => Some(Ordering::Less),
            x if x == 0.0 => Some(Ordering::Equal),
            x if x > 0.0 => Some(Ordering::Greater),
            _ => None,
        }
    }
}

impl Ord for Ask {
    fn cmp(&self, other: &Self) -> Ordering {
        self.price
            .partial_cmp(&other.price)
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                self.quantity
                    .partial_cmp(&other.quantity)
                    .unwrap_or(Ordering::Equal)
            })
    }
}

impl Order for Ask {
    fn new(instrument: Instrument, exchange: ExchangeType, quantity: f64, price: f64) -> Self {
        Ask {
            instrument: instrument,
            exchange: exchange,
            quantity: quantity,
            price: price,
        }
    }

    fn quantity(&self) -> f64 {
        self.quantity
    }

    fn price(&self) -> f64 {
        self.price
    }

    fn instrument(&self) -> Instrument {
        self.instrument
    }

    fn exchange(&self) -> ExchangeType {
        self.exchange
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::book_management::AggregatedOrderBook;
    use crate::book_management::traded_instruments::Instrument;
    use crate::exchange_connectivity::Exchange;
    use crate::exchange_connectivity::{ExchangeKeys, binance::Binance, deribit::Deribit};

    #[tokio::test]
    async fn get_book() {
        dotenv::dotenv().ok();
        let keys = ExchangeKeys::get_environment();
        let (binance, _) = Binance::connect()
            .await
            .expect("Binance failed to connect.");
        let (deribit, _) = Deribit::connect(keys.deribit_client_id, keys.deribit_api_key)
            .await
            .expect("Deribit failed to connect.");

        let binance = Arc::new(binance);
        let deribit = Arc::new(deribit);

        let book = AggregatedOrderBook::new(Instrument::BtcUsdt, &vec![
            Exchange::Binance(Arc::clone(&binance)),
            Exchange::Deribit(Arc::clone(&deribit)),
        ]);

        if let Err(err) = book.pretty_print().await {
            panic!("Unexpected error when printing: {}", err);
        }
    }
}
