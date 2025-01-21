pub mod book;

use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use multimap::MultiMap;
use serde_json::json;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async, tungstenite::protocol::Message,
};

type Sink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type Stream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

const BINANCE_WS_URL: &str = "wss://ws-api.binance.com:9443/ws-api/v3";
const BINANCE_WS_TEST_URL: &str = "wss://testnet.binance.vision/ws-api/v3";

/// Connect then ws manager
///
/// warning that ids for binance are 64-bit signed integers
/// but program enforces u64.
#[derive(Debug)]
pub struct Binance {
    sink: Arc<Mutex<Sink>>,
    stream: Arc<Mutex<Stream>>,
    non_main_stream: Arc<Mutex<MultiMap<u64, String>>>,
    keep_alive: Arc<AtomicBool>,
    curr_msg_id: AtomicU64,
}

impl Binance {
    pub async fn connect() -> Option<(Self, Arc<AtomicBool>)> {
        let connection_url = if cfg!(test) || cfg!(feature = "test-apis") {
            log::info!("Using Binance test URL");
            BINANCE_WS_TEST_URL
        } else {
            log::info!("Using Binance standard WS URL.");
            BINANCE_WS_URL
        };

        let connection_response = connect_async(connection_url).await;

        match connection_response {
            Err(err) => {
                println!("{}", err);
                log::error!("Error connecting to Binance client: {}", err);
                None
            }
            Ok(ok) => {
                log::info!("Connection established with Binance client");
                let (sink, stream) = ok.0.split();
                let keep_alive = Arc::new(AtomicBool::new(true));

                Some((
                    Binance {
                        sink: Arc::new(Mutex::new(sink)),
                        stream: Arc::new(Mutex::new(stream)),
                        non_main_stream: Arc::new(Mutex::new(MultiMap::new())),
                        keep_alive: Arc::clone(&keep_alive),
                        curr_msg_id: AtomicU64::new(10000),
                    },
                    keep_alive,
                ))
            }
        }
    }

    pub async fn ws_manager(&self) {
        loop {
            let keep_running = {
                let guard = &self.keep_alive;
                guard.load(Ordering::Relaxed)
            };

            if !keep_running {
                break;
            }

            while self.keep_alive.load(Ordering::Relaxed) {
                if let Err(err) = self.process_next_message().await {
                    log::error!("Error processing WebSocket message: {}", err);
                }
            }

            log::info!("WebSocket manager shutting down gracefully.");
        }
    }

    async fn ws_pong(&self, id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let msg = json!({
            "id": id,
            "method": "pong",
        });

        self.sink.lock().await.send(msg.to_string().into()).await?;

        Ok(())
    }

    async fn process_next_message(&self) -> Result<(), String> {
        let next_message = {
            let mut stream = self.stream.lock().await;
            stream.next().await
        };

        match next_message {
            Some(Ok(Message::Text(text))) => self.handle_text_message(&text).await,
            Some(Ok(_)) => {
                log::warn!("Unexpected non-text message received.");
                Ok(())
            }
            Some(Err(err)) => Err(format!("Error reading WebSocket stream: {}", err)),
            None => {
                log::warn!("WebSocket stream closed.");
                Ok(())
            }
        }
    }

    async fn handle_text_message(&self, text: &str) -> Result<(), String> {
        let msg: serde_json::Value = serde_json::from_str(text)
            .map_err(|err| format!("Failed to parse message: {}", err))?;

        if let Some("ping") = msg["method"].as_str()
            && let Some(id) = msg["id"].as_str()
        {
            log::info!("Processed message: {}", text);
            match self.ws_pong(id).await {
                Err(err) => {
                    log::error!("Error sending pong: {}", err);
                }
                Ok(_) => {
                    log::info!("Successfully responded pong to Binance ping.");
                }
            }
        } else if let Some(id_str) = msg["id"].as_str()
            && let Ok(id) = id_str.parse()
        {
            self.non_main_stream
                .lock()
                .await
                .insert(id, text.to_string());
            log::info!(
                "Unprocessed message with valid ID from Binance stored with id: {}",
                id
            );
        } else {
            log::info!(
                "Unprocessed message with no valid ID component from Binance: {}",
                text
            );
        }

        Ok(())
    }

    pub async fn ws_request_time(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let msg = json!({
            "id": "1000",
            "method": "ping",
        });

        self.sink.lock().await.send(msg.to_string().into()).await?;

        Ok(())
    }

    fn get_new_id(&self) -> u64 {
        if self.curr_msg_id.load(Ordering::Relaxed) > 1000000 {
            self.curr_msg_id.store(10000, Ordering::Relaxed);
        } else {
            self.curr_msg_id.fetch_add(1, Ordering::Relaxed);
        }

        self.curr_msg_id.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod test {
    use crate::exchange_connectivity::{ConnectedExchangeForBook, Instrument};
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;

    use super::Binance;

    async fn setup() -> (Binance, Arc<AtomicBool>) {
        match Binance::connect().await {
            None => panic!("Expected successful connection."),
            Some(x) => x,
        }
    }

    #[tokio::test]
    async fn test_connect() {
        match Binance::connect().await {
            None => panic!("Expected successful connection."),
            Some(_) => (),
        }
    }

    #[tokio::test]
    async fn test_request_time() {
        let (binance, _) = setup().await;

        let binance = Arc::new(binance);
        let binance_clone = Arc::clone(&binance);
        tokio::spawn(async move {
            binance_clone.ws_manager().await;
        });

        if let Err(err) = binance.ws_request_time().await {
            log::error!("Error sending Binance time request: {}", err);
        }
    }

    #[tokio::test]
    async fn retrieve_books() {
        let (binance, _) = setup().await;
        let binance = Arc::new(binance);

        let binance_clone = Arc::clone(&binance);
        tokio::spawn(async move {
            binance_clone.ws_manager().await;
        });

        let result_btc_usdt = match binance.pull_bids_asks(10, Instrument::BtcUsdt).await {
            Ok(vec) => vec,
            Err(err) => {
                panic!("Error getting message: {}", err);
            }
        };
        let result_eth_usdc = match binance.pull_bids_asks(10, Instrument::EthUsdc).await {
            Ok(vec) => vec,
            Err(err) => {
                panic!("Error getting message: {}", err);
            }
        };

        assert!(result_btc_usdt.0.len() <= 10);
        assert!(result_btc_usdt.1.len() <= 10);
        assert!(result_eth_usdc.0.len() <= 10);
        assert!(result_eth_usdc.1.len() <= 10);
    }
}
