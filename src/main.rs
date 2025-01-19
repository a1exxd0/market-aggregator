use market_aggregator::{
    book_management,
    exchange_connectivity::{ConnectedExchange, ExchangeKeys, deribit::Deribit},
};
use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
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
        .pull_bids_asks(10, book_management::traded_instruments::Instrument::BtcUsdt)
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
