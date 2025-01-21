use std::{sync::{atomic::Ordering, Arc}, time::Duration};

use market_aggregator::{book_management::{traded_instruments::Instrument, AggregatedOrderBook}, exchange_connectivity::{Exchange, ExchangeKeys, ExchangeType}, MyApp};

// Integration tests use the non-test version of the main module, binance sucks!
#[tokio::test]
async fn create_book_aggregation_update() {
    let keys = ExchangeKeys::get_environment();

    // market_aggregator::logging_config();

    let (deribit, deribit_keep_alive) = Exchange::connect(ExchangeType::Deribit, &keys)
        .await
        .unwrap();

    // let (binance, binance_keep_alive) = Exchange::connect(ExchangeType::Binance, &keys)
    //     .await
    //     .unwrap();

    let exchanges = Arc::new(vec![deribit]);
    
    let book_collection = vec![
        Arc::new(AggregatedOrderBook::new(Instrument::BtcUsdt, &exchanges)),
        Arc::new(AggregatedOrderBook::new(Instrument::EthUsdc, &exchanges)),
        Arc::new(AggregatedOrderBook::new(Instrument::EthBtc, &exchanges)),
    ];

    println!("before");
    let _my_app = MyApp::new(book_collection.into_iter());
    println!("after");
    
    // it refreshes automatically in this time
    tokio::time::sleep(Duration::from_millis(100)).await;

    deribit_keep_alive.store(false, Ordering::Relaxed);
    // binance_keep_alive.store(false, Ordering::Relaxed);
    return;
}