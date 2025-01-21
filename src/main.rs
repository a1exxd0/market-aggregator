use book_management::traded_instruments::Instrument;
use market_aggregator::{
    book_management::{self, AggregatedOrderBook},
    exchange_connectivity::{Exchange, ExchangeKeys, ExchangeType},
};
use std::sync::{Arc, atomic::Ordering};

use std::sync::Mutex;
use tokio::task;

use colored::Colorize;

#[tokio::main]
async fn main() {
    let keys = ExchangeKeys::get_environment();

    logging_config();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    // let (binance, binance_keep_alive) = Exchange::connect(ExchangeType::Binance, &keys)
    //     .await
    //     .unwrap();
    let (deribit, deribit_keep_alive) = Exchange::connect(ExchangeType::Deribit, &keys)
        .await
        .unwrap();

    let exchanges = vec![
        deribit, 
        //binance,
    ];

    if let Err(err) = eframe::run_native(
        "Market Aggregator",
        options,
        Box::new(move |_cc| {
            let book = Arc::new(AggregatedOrderBook::new(Instrument::BtcUsdt, {
                &exchanges
            }));

            Ok(Box::<MyApp>::new(MyApp::new(book)))
        }),
    ) {
        log::error!("Failure whilst hosting UI: {}", err);
    }

    binance_keep_alive.store(false, Ordering::Relaxed);
    deribit_keep_alive.store(false, Ordering::Relaxed);
}

struct MyApp {
    book: Arc<AggregatedOrderBook>,
    pretty_output: Arc<Mutex<Option<String>>>,
}

impl MyApp {
    fn new(book: Arc<AggregatedOrderBook>) -> Self {
        Self {
            book: book,
            pretty_output: Arc::new(std::sync::Mutex::new(None)),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("market-aggregator");

            if ui.button("Refresh").clicked() {
                let book = self.book.clone();
                let pretty_output = self.pretty_output.clone();

                task::spawn(async move {
                    let _ = book.update_state().await;
                    let output = book
                        .pretty_print()
                        .await
                        .unwrap_or_else(|e| format!("Error: {e}"));
                    *pretty_output.lock().unwrap() = Some(output);
                });
            }

            if let Some(output) = &*self.pretty_output.lock().unwrap() {
                ui.monospace(output);
            } else {
                ui.label("Loading...");
            }
        });
    }
}

fn logging_config() {
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
