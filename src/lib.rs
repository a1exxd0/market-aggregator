#![feature(let_chains)]
pub mod book_management;
pub mod exchange_connectivity;
pub mod time_series_array;

#[cfg(test)]
mod test {
    use crate::{
        book_management::{AggregatedOrderBook, traded_instruments::Instrument},
        exchange_connectivity::{Exchange, ExchangeKeys, ExchangeType},
    };

    #[tokio::test]
    async fn aggregate() {
        let keys = ExchangeKeys::get_environment();

        let (binance, _) = Exchange::connect(ExchangeType::Binance, &keys)
            .await
            .unwrap();
        let (deribit, _) = Exchange::connect(ExchangeType::Deribit, &keys)
            .await
            .unwrap();

        let exchanges = vec![deribit, binance];

        let aggregated = AggregatedOrderBook::new(Instrument::BtcUsdt, &exchanges);

        if let Err(err) = aggregated.update_state().await {
            panic!("Failed aggregation state update with err {}", err);
        }

        // uncomment and run cargo test if binance api doesnt work.
        // panic!("{}", aggregated.pretty_print().await.unwrap());
    }
}
