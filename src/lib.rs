#![feature(let_chains)]
pub mod book_management;
pub mod exchange_connectivity;
pub mod gui;
pub mod time_series_array;

use colored::Colorize;

pub fn logging_config() {
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
}

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
