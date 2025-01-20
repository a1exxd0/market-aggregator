use book_management::traded_instruments::Instrument;
use market_aggregator::{
    book_management,
    exchange_connectivity::{binance::Binance, deribit::Deribit, ConnectedExchangeForBook, ExchangeKeys},
};
use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use colored::Colorize;

#[tokio::main]
async fn main() {
    fern::Dispatch::new()
        .format(move |out, message, record| {
            let level_colored = match record.level() {
                log::Level::Error => record.level().to_string().red(),
                log::Level::Warn => record.level().to_string().yellow(),
                log::Level::Info => record.level().to_string().green(),
                log::Level::Debug => record.level().to_string().blue(),
                log::Level::Trace => record.level().to_string().magenta(),
            };

            out.finish(format_args!(
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                level_colored,
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    let keys = ExchangeKeys::get_environment();
    let (deribit_base, keep_alive) = Deribit::connect(keys.deribit_client_id, keys.deribit_api_key)
        .await
        .expect("Issue found connecting to Deribit");

    let deribit = Arc::new(deribit_base);

    let deribit_clone = Arc::clone(&deribit);
    tokio::spawn(async move {
        deribit_clone.ws_manager().await;
    });

    let bids_asks = Arc::clone(&deribit)
        .pull_bids_asks(10, Instrument::BtcUsdt)
        .await;

    match bids_asks {
        Ok(val) => {
            println!("{:?}", val.0);
            println!("{:?}", val.1);
        }
        Err(err) => {
            log::error!("{}", err);
        }
    }

    let bids_asks = Arc::clone(&deribit)
        .pull_bids_asks(10, Instrument::EthUsdc)
        .await;

    match bids_asks {
        Ok(val) => {
            println!("{:?}", val.0);
            println!("{:?}", val.1);
        }
        Err(err) => {
            log::error!("{}", err);
        }
    }

    // tokio::time::sleep(Duration::from_secs(5)).await;

    keep_alive.store(false, Ordering::Relaxed);



    let (binance, _) = Binance::connect().await.unwrap();
    let binance = Arc::new(binance);

    let binance_clone = Arc::clone(&binance);
    tokio::spawn(async move {
        binance_clone.ws_manager().await;
    });

    let bids_asks = Arc::clone(&binance)
        .pull_bids_asks(10, Instrument::BtcUsdt)
        .await;

    match bids_asks {
        Ok(val) => {
            println!("{:?}", val.0);
            println!("{:?}", val.1);
        }
        Err(err) => {
            log::error!("{}", err);
        }
    }

    tokio::time::sleep(Duration::from_secs(5)).await;

    keep_alive.store(false, Ordering::Relaxed);
}
