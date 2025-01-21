use crate::book_management::AggregatedOrderBook;
use crate::book_management::multibook::Multibook;
use chrono::TimeZone;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use std::sync::Mutex;
use tokio::task;

pub struct MyApp {
    books: Multibook,
    book_properties: BTreeMap<u32, Arc<AppBookProperties>>,
}

struct AppBookProperties {
    pretty_output: Arc<Mutex<Option<String>>>,
    curr_time: Arc<Mutex<Duration>>,
    imbalance: Arc<Mutex<f64>>,
}

impl Default for AppBookProperties {
    fn default() -> Self {
        AppBookProperties {
            pretty_output: Arc::new(Mutex::new(None)),
            curr_time: Arc::new(Mutex::new(Duration::from_secs(0))),
            imbalance: Arc::new(Mutex::new(1.0)),
        }
    }
}

impl MyApp {
    pub fn new(books: impl Iterator<Item = Arc<AggregatedOrderBook>>) -> Self {
        let mut book_properties = BTreeMap::new();
        let mut app = Self {
            books: {
                let mut multibook = Multibook::new();

                for book in books {
                    let id = multibook.insert(&book);
                    book_properties.insert(id, Arc::new(AppBookProperties::default()));
                }

                multibook
            },
            book_properties: book_properties,
        };

        app.create_refresh_tasks();

        app
    }

    fn create_refresh_tasks(&mut self) {
        let key_book = self.books.subscribed.iter();

        for (key, book) in key_book {
            let key = key.clone();
            let book = Arc::clone(book);
            let property = self.book_properties.get(&key);

            if property.is_none() {
                log::error!("Error finding property for some id.");
            }

            let property = Arc::clone(property.unwrap());

            task::spawn(async move {
                loop {
                    if let Err(err) = book.update_state().await {
                        log::error!(
                            "Error during book state update in refresh app task: {}",
                            err
                        );
                    }

                    let output = book
                        .pretty_print()
                        .await
                        .unwrap_or_else(|e| format!("Error: {e}"));

                    *property.pretty_output.lock().unwrap() = Some(output);
                    *property.curr_time.lock().unwrap() = book.last_time().await;
                    *property.imbalance.lock().unwrap() = book.imbalance().await;

                    tokio::time::sleep(Duration::from_millis(300)).await;
                }
            });
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (_, property) in &self.book_properties {
                    let property = Arc::clone(property);

                    ui.vertical(|ui| {
                        ui.heading("market-aggregator");
                        let curr_time = *property.curr_time.lock().unwrap();
                        ui.horizontal(|ui| {
                            ui.label(format!("Last msg recv: {:#?}", {
                                let secs = curr_time.as_secs() as i64;
                                let nsecs = curr_time.subsec_nanos() as u32;

                                let datetime = chrono::Utc.timestamp_opt(secs, nsecs).unwrap();

                                datetime.format("%Y-%m-%d %H:%M:%S%.3f %Z").to_string()
                            }));
                            ui.label(format!(
                                "Imbalance l/r: {}",
                                property.imbalance.lock().unwrap()
                            ))
                        });

                        if let Some(output) = &*property.pretty_output.lock().unwrap() {
                            ui.monospace(output);
                        } else {
                            ui.label("Loading...");
                        }
                    });
                }

                ctx.request_repaint_after(std::time::Duration::from_millis(300));
            });
        });
    }
}
