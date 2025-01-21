mod binance;
mod deribit;

use std::sync::atomic::AtomicBool;
use std::time::Duration;
use std::{env, sync::Arc};

use binance::Binance;
use deribit::Deribit;
use dotenv::dotenv;
use std::error::Error;
use tokio::task::spawn;

use crate::book_management::{Ask, Bid, traded_instruments::Instrument};

pub trait ConnectedExchangeForBook {
    /// For a given request, pull (up to) `depth` bids and asks for
    /// some specific instrument.
    ///
    /// Returns a vector of bids and asks in the book alongside
    /// the time at which a response was recieved in time
    /// after the UNIX epoch (Deribit standard).
    fn pull_bids_asks(
        &self,
        depth: u32,
        instrument: Instrument,
    ) -> impl std::future::Future<
        Output = Result<(Vec<Bid>, Vec<Ask>, Duration), Box<dyn Error + Send + Sync>>,
    > + Send;

    fn to_instrument_name(instrument: Instrument) -> String
    where
        Self: Sized;
}

#[derive(Clone, Copy, Debug)]
pub enum ExchangeType {
    Deribit,
    Binance,
}

#[derive(Debug)]
pub enum Exchange {
    Deribit(Arc<Deribit>),
    Binance(Arc<Binance>),
}

impl Clone for Exchange {
    fn clone(&self) -> Self {
        match self {
            Exchange::Binance(binance) => Exchange::Binance(Arc::clone(&binance)),
            Exchange::Deribit(deribit) => Exchange::Deribit(Arc::clone(&deribit)),
        }
    }
}

impl Exchange {
    pub async fn pull_bids_asks(
        &self,
        depth: u32,
        instrument: Instrument,
    ) -> Result<(Vec<Bid>, Vec<Ask>, Duration), Box<dyn Error + Send + Sync>> {
        match self {
            Exchange::Deribit(deribit) => deribit.pull_bids_asks(depth, instrument).await,
            Exchange::Binance(binance) => binance.pull_bids_asks(depth, instrument).await,
        }
    }

    pub async fn connect(
        exchange: ExchangeType,
        keys: &ExchangeKeys,
    ) -> Option<(Exchange, Arc<AtomicBool>)> {
        match exchange {
            ExchangeType::Binance => {
                let (binance, keep_alive) = {
                    let (binance, keep_alive) = Binance::connect().await?;
                    (Arc::new(binance), keep_alive)
                };

                let binance_clone = Arc::clone(&binance);
                spawn(async move {
                    binance_clone.ws_manager().await;
                });

                Some((Exchange::Binance(binance), keep_alive))
            }
            ExchangeType::Deribit => {
                let (deribit, keep_alive) = {
                    let (deribit, keep_alive) = Deribit::connect(
                        keys.deribit_client_id.to_string(),
                        keys.deribit_api_key.to_string(),
                    )
                    .await?;
                    (Arc::new(deribit), keep_alive)
                };

                let deribit_clone = Arc::clone(&deribit);
                spawn(async move {
                    deribit_clone.ws_manager().await;
                });

                Some((Exchange::Deribit(deribit), keep_alive))
            }
        }
    }
}

pub struct ExchangeKeys {
    pub deribit_client_id: String,
    pub deribit_api_key: String,
}

impl ExchangeKeys {
    pub fn get_environment() -> ExchangeKeys {
        dotenv().ok();

        let deribit_client_id = env::var("DERIBIT_CLIENT_ID").expect(
            "You must create a .env file with your Deribit Client ID. Read the README.md for more info.",
        );

        let deribit_api_key = env::var("DERIBIT_API_KEY").expect(
            "You must create a .env file with your Deribit API key. Read the README.md for more info.",
        );

        ExchangeKeys {
            deribit_client_id: deribit_client_id,
            deribit_api_key: deribit_api_key,
        }
    }
}
