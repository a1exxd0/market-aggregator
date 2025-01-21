pub mod book;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use futures_util::{SinkExt, StreamExt, stream::SplitSink, stream::SplitStream};
use log::info;
use multimap::MultiMap;
use serde_json::json;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async, tungstenite::protocol::Message,
};

type Sink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type Stream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

#[derive(Debug)]
pub struct Deribit {
    client_id: String,
    client_secret: String,
    sink: Arc<Mutex<Sink>>,
    stream: Arc<Mutex<Stream>>,
    non_main_stream: Arc<Mutex<MultiMap<u64, String>>>,
    refresh_token: Arc<Mutex<Option<String>>>,
    refresh_token_expiry_time: Arc<Mutex<Option<Duration>>>,
    keep_alive: Arc<AtomicBool>,
    curr_msg_id: AtomicU64,
}

const DERIBIT_WS_URL: &str = "wss://www.deribit.com/ws/api/v2";
const DERIBIT_WS_TEST_URL: &str = "wss://test.deribit.com/ws/api/v2";

impl Deribit {
    pub async fn connect(
        client_id: String,
        client_secret: String,
    ) -> Option<(Self, Arc<AtomicBool>)> {
        let connection_url = if cfg!(test) || cfg!(feature = "test-apis") {
            info!("Using Deribit test URL");
            DERIBIT_WS_TEST_URL
        } else {
            DERIBIT_WS_URL
        };

        let connection_response = connect_async(connection_url).await;
        match connection_response {
            Err(err) => {
                log::error!("Error connecting to client. {}", err);
                None
            }
            Ok(ok) => {
                let (sink, stream) = ok.0.split();
                let keep_alive = Arc::new(AtomicBool::new(true));

                Some((
                    Deribit {
                        client_id: client_id,
                        client_secret: client_secret,
                        sink: Arc::new(Mutex::new(sink)),
                        stream: Arc::new(Mutex::new(stream)),
                        non_main_stream: Arc::new(Mutex::new(MultiMap::new())),
                        refresh_token: Arc::new(Mutex::new(None)),
                        refresh_token_expiry_time: Arc::new(Mutex::new(None)),
                        keep_alive: keep_alive.clone(),
                        curr_msg_id: AtomicU64::new(10000),
                    },
                    keep_alive.clone(),
                ))
            }
        }
    }

    pub async fn ws_manager(&self) {
        if let Err(err) = self.initialize_ws().await {
            log::error!("WebSocket initialization failed: {}", err);
            return;
        }

        self.spawn_refresh_auth_task();

        // Subscribe here

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

    async fn initialize_ws(&self) -> Result<(), String> {
        self.ws_auth()
            .await
            .map_err(|e| format!("Failed to authenticate: {}", e))?;
        log::info!("Successfully authenticated WebSocket connection.");

        self.establish_heartbeat()
            .await
            .map_err(|e| format!("Failed to establish heartbeat: {}", e))?;
        log::info!("Heartbeat successfully established.");
        Ok(())
    }

    async fn ws_auth(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 9929,
            "method": "public/auth",
            "params": {
                "grant_type": "client_credentials",
                "client_id": self.client_id,
                "client_secret": self.client_secret,
            },
        });

        self.sink
            .lock()
            .await
            .send(Message::Text(msg.to_string().into()))
            .await?;

        Ok(())
    }

    fn spawn_refresh_auth_task(&self) {
        tokio::spawn(Self::ws_refresh_auth(
            Arc::clone(&self.sink),
            Arc::clone(&self.refresh_token),
            Arc::clone(&self.refresh_token_expiry_time),
        ));
        log::info!("Refresh authentication task spawned.");
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

        if let Some(9929) = msg["id"].as_u64() {
            log::info!("Processed message: {}", text);
            self.update_auth_tokens(&msg).await?;
        } else if let Some("heartbeat") = msg["method"].as_str() {
            log::info!("Processed heartbeat: {}", text);
            self.heartbeat_response()
                .await
                .map_err(|e| format!("Failed to send heartbeat response: {}", e))?;
        } else if let Some(8212) = msg["id"].as_u64() {
            log::info!("Recieved Deribit heartbeat response {}", msg);
        } else if let Some(id) = msg["id"].as_u64() {
            self.non_main_stream
                .lock()
                .await
                .insert(id, text.to_string());
            log::info!(
                "Unprocessed message with valid ID from Deribit stored with id: {}\n\n msg: {}",
                id,
                msg,
            );
        } else {
            log::info!(
                "Unprocessed message with no valid ID component from Deribit: {}\n\n msg: {}",
                text,
                msg,
            );
        }
        Ok(())
    }

    async fn update_auth_tokens(&self, msg: &serde_json::Value) -> Result<(), String> {
        let refresh_token = msg["result"]["refresh_token"]
            .as_str()
            .ok_or("Missing refresh token")?
            .to_string();

        let expires_in = msg["result"]["expires_in"]
            .as_u64()
            .map(Duration::from_secs)
            .unwrap_or_else(|| Duration::from_secs(540))
            - Duration::from_secs(240);

        let mut refresh_token_guard = self.refresh_token.lock().await;
        *refresh_token_guard = Some(refresh_token);

        if let Ok(curr_time) = SystemTime::now().duration_since(UNIX_EPOCH) {
            let mut expiry_time_guard = self.refresh_token_expiry_time.lock().await;
            *expiry_time_guard = Some(curr_time + expires_in);
        } else {
            return Err("Failed to get system time".to_string());
        }

        log::info!("Authentication tokens updated successfully.");
        Ok(())
    }

    async fn establish_heartbeat(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 9098,
            "method": "public/set_heartbeat",
            "params": {
                "interval": 30,
            },
        });

        self.sink
            .lock()
            .await
            .send(Message::Text(msg.to_string().into()))
            .await?;

        Ok(())
    }

    async fn heartbeat_response(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 8212,
            "method": "public/test",
            "params": {},
        });

        self.sink.lock().await.send(msg.to_string().into()).await?;

        Ok(())
    }

    async fn ws_refresh_auth(
        sink: Arc<Mutex<Sink>>,
        refresh_token: Arc<Mutex<Option<String>>>,
        refresh_token_expiry_time: Arc<Mutex<Option<Duration>>>,
    ) {
        loop {
            log::info!("Checking if Deribit auth is close to expiry.");

            if let Some(expiry_time) = *refresh_token_expiry_time.lock().await
                && let Some(refresh_token) = &*refresh_token.lock().await
            {
                log::info!("Auth token for Deribit ConnectedExchangeclose to expiry. Refreshing.");
                if let Ok(curr_time) = SystemTime::now().duration_since(UNIX_EPOCH) {
                    if curr_time > expiry_time {
                        let msg = json!({
                            "jsonrpc": "2.0",
                            "id": 9929,
                            "method": "public/auth",
                            "params": {
                                "grant_type": "refresh_token",
                                "refresh_token": refresh_token
                            },
                        });

                        let result = sink.lock().await.send(msg.to_string().into()).await;

                        if let Err(err) = result {
                            log::error!(
                                "Failed to send reauth-refresh-token for Deribit with error: {}",
                                err
                            )
                        } else {
                            log::info!("Auth token refreshed for Deribit.")
                        }
                    } else {
                        log::info!(
                            "Deribit auth expires at {} seconds after Unix Epoch",
                            expiry_time.as_secs()
                        );
                    }
                } else {
                    log::warn!(
                        r#"Error converting current time to a Rust Duration. 
                        Verify connection to Deribit persists."#
                    )
                }
            }

            tokio::time::sleep(Duration::from_secs(150)).await;
        }
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
    use std::sync::{Arc, atomic::AtomicBool};

    use crate::{
        book_management::traded_instruments::Instrument,
        exchange_connectivity::{ConnectedExchangeForBook, ExchangeKeys},
    };

    use super::Deribit;

    async fn create_exchange() -> (Deribit, Arc<AtomicBool>) {
        let keys = ExchangeKeys::get_environment();
        Deribit::connect(keys.deribit_client_id, keys.deribit_api_key)
            .await
            .expect("Issue found connecting to Deribit")
    }

    #[tokio::test]
    async fn send_auth() {
        let deribit = create_exchange().await;

        let auth = deribit.0.ws_auth().await;

        if let Err(err) = auth {
            println!("{}", err.to_string());
            panic!("unexpected error!");
        }
    }

    #[tokio::test]
    async fn establish_heartbeat() {
        let deribit = create_exchange().await;
        let _ = deribit.0.ws_auth().await;

        let result = deribit.0.establish_heartbeat().await;

        if let Err(err) = result {
            println!("{}", err.to_string());
            panic!("unexpected error!");
        }
    }

    #[tokio::test]
    async fn retrieve_books() {
        let (deribit, _) = create_exchange().await;
        let deribit = Arc::new(deribit);

        let deribit_clone = Arc::clone(&deribit);
        tokio::spawn(async move {
            deribit_clone.ws_manager().await;
        });

        let result_btc_usdt = match deribit.pull_bids_asks(10, Instrument::BtcUsdt).await {
            Ok(vec) => vec,
            Err(err) => {
                panic!("Error getting message: {}", err);
            }
        };
        let result_eth_usdc = match deribit.pull_bids_asks(10, Instrument::EthUsdc).await {
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
