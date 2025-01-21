use book_management::traded_instruments::Instrument;
use market_aggregator::{
    book_management::{self, AggregatedOrderBook},
    exchange_connectivity::{Exchange, ExchangeKeys, ExchangeType},
    gui::MyApp,
};

use std::sync::{Arc, atomic::Ordering};

#[tokio::main]
async fn main() {
    market_aggregator::logging_config();
    
    if cfg!(feature = "include-binance") || cfg!(feature = "test-apis") {
        main_with_binance().await;
    } else {
        main_without_binance().await;
    }
}

async fn main_without_binance() {
    let keys = ExchangeKeys::get_environment();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1920.0, 1080.0]),
        ..Default::default()
    };

    let (deribit, deribit_keep_alive) = Exchange::connect(ExchangeType::Deribit, &keys)
        .await
        .unwrap();

    let exchanges = Arc::new(vec![deribit]);

    let book_collection = vec![
        Arc::new(AggregatedOrderBook::new(Instrument::BtcUsdt, &exchanges)),
        Arc::new(AggregatedOrderBook::new(Instrument::EthUsdc, &exchanges)),
        Arc::new(AggregatedOrderBook::new(Instrument::EthBtc, &exchanges)),
    ];

    if let Err(err) = eframe::run_native(
        "Market Aggregator",
        options,
        Box::new(move |_cc| {
            let books = book_collection;

            Ok(Box::<MyApp>::new(MyApp::new(books.into_iter())))
        }),
    ) {
        log::error!("Failure whilst hosting UI: {}", err);
    }

    deribit_keep_alive.store(false, Ordering::Relaxed);
}

async fn main_with_binance() {
    let keys = ExchangeKeys::get_environment();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1920.0, 1080.0]),
        ..Default::default()
    };

    let (binance, binance_keep_alive) = Exchange::connect(ExchangeType::Binance, &keys)
        .await
        .unwrap();
    let (deribit, deribit_keep_alive) = Exchange::connect(ExchangeType::Deribit, &keys)
        .await
        .unwrap();

    let exchanges = Arc::new(vec![deribit, binance]);

    let book_collection = vec![
        Arc::new(AggregatedOrderBook::new(Instrument::BtcUsdt, &exchanges)),
        Arc::new(AggregatedOrderBook::new(Instrument::EthUsdc, &exchanges)),
        Arc::new(AggregatedOrderBook::new(Instrument::EthBtc, &exchanges)),
    ];

    if let Err(err) = eframe::run_native(
        "Market Aggregator",
        options,
        Box::new(move |_cc| {
            let books = book_collection;

            Ok(Box::<MyApp>::new(MyApp::new(books.into_iter())))
        }),
    ) {
        log::error!("Failure whilst hosting UI: {}", err);
    }

    binance_keep_alive.store(false, Ordering::Relaxed);
    deribit_keep_alive.store(false, Ordering::Relaxed);
}