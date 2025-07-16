// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

use std::{
    fmt::Debug,
    num::NonZeroU32,
    sync::{
        Arc, LazyLock,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use ahash::{AHashMap, AHashSet};
use async_stream::stream;
use futures_util::Stream;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription};
use nautilus_core::nanos::UnixNanos;
use nautilus_network::ratelimiter::quota::Quota;
use tokio::sync::{Mutex, mpsc};
use tracing::{debug, error, info};
use ustr::Ustr;

use nautilus_model::{
    enums::BookAction,
    identifiers::{AccountId, InstrumentId},
    instruments::{Instrument, InstrumentAny},
};

use super::{
    enums::{HyperliquidWsChannel, NautilusWsMessage},
    error::{HyperliquidWsError, Result},
    parse::parse_l2_book_data,
};

/// Default Hyperliquid WebSocket rate limit: 10 requests per second.
pub static HYPERLIQUID_WS_QUOTA: LazyLock<Quota> =
    LazyLock::new(|| Quota::per_second(NonZeroU32::new(10).unwrap()));

#[derive(Clone)]
pub struct HyperliquidWebSocketClient {
    base_url: BaseUrl,
    account_id: AccountId,
    info_client: Option<Arc<Mutex<InfoClient>>>,
    rx: Option<Arc<mpsc::UnboundedReceiver<NautilusWsMessage>>>,
    tx: Option<mpsc::UnboundedSender<NautilusWsMessage>>,
    signal: Arc<AtomicBool>,
    subscriptions: Arc<Mutex<AHashMap<HyperliquidWsChannel, AHashSet<Ustr>>>>,
    instruments_cache: Arc<AHashMap<Ustr, InstrumentAny>>,
    symbol_to_asset_id: Arc<AHashMap<Ustr, String>>,
    subscription_handles: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
}

impl Debug for HyperliquidWebSocketClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HyperliquidWebSocketClient")
            .field("account_id", &self.account_id)
            .finish_non_exhaustive()
    }
}

impl HyperliquidWebSocketClient {
    pub fn new(
        account_id: AccountId,
        base_url: Option<BaseUrl>,
        instruments: Option<Vec<InstrumentAny>>,
    ) -> Self {
        let instruments_cache = if let Some(instruments) = instruments {
            instruments
                .into_iter()
                .map(|inst| (inst.symbol().inner(), inst))
                .collect()
        } else {
            AHashMap::new()
        };

        Self {
            base_url: base_url.unwrap_or(BaseUrl::Mainnet),
            account_id,
            info_client: None,
            rx: None,
            tx: None,
            signal: Arc::new(AtomicBool::new(false)),
            subscriptions: Arc::new(Mutex::new(AHashMap::new())),
            instruments_cache: Arc::new(instruments_cache),
            symbol_to_asset_id: Arc::new(AHashMap::new()),
            subscription_handles: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn new_with_asset_ids(
        account_id: AccountId,
        base_url: Option<BaseUrl>,
        instruments: Vec<InstrumentAny>,
        asset_id_map: AHashMap<Ustr, String>,
    ) -> Self {
        let instruments_cache = instruments
            .into_iter()
            .map(|inst| (inst.symbol().inner(), inst))
            .collect();

        Self {
            base_url: base_url.unwrap_or(BaseUrl::Mainnet),
            account_id,
            info_client: None,
            rx: None,
            tx: None,
            signal: Arc::new(AtomicBool::new(false)),
            subscriptions: Arc::new(Mutex::new(AHashMap::new())),
            instruments_cache: Arc::new(instruments_cache),
            symbol_to_asset_id: Arc::new(asset_id_map),
            subscription_handles: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        let info_client = InfoClient::new(None, Some(self.base_url.clone()))
            .await
            .map_err(HyperliquidWsError::Sdk)?;

        self.info_client = Some(Arc::new(Mutex::new(info_client)));

        let (tx, rx) = mpsc::unbounded_channel();
        self.tx = Some(tx);
        self.rx = Some(Arc::new(rx));

        tokio::time::sleep(Duration::from_millis(100)).await;
        info!("Connected to Hyperliquid WebSocket successfully");
        Ok(())
    }

    pub async fn subscribe_order_book(&self, instrument_id: InstrumentId) -> Result<()> {
        let full_symbol = instrument_id.symbol.to_string();
        let full_symbol_ustr = Ustr::from(&full_symbol);

        // Check that the instrument exists in our cache
        if !self.instruments_cache.contains_key(&full_symbol_ustr) {
            return Err(HyperliquidWsError::InvalidInstrument(full_symbol));
        }

        // Get the proper asset ID for this symbol
        let coin = self
            .symbol_to_asset_id
            .get(&full_symbol_ustr)
            .ok_or_else(|| {
                HyperliquidWsError::InvalidInstrument(format!(
                    "No asset ID mapping for symbol: {}",
                    full_symbol
                ))
            })?
            .clone();

        let info_client = self
            .info_client
            .as_ref()
            .ok_or(HyperliquidWsError::Connection("Not connected".to_string()))?
            .clone();

        let main_tx = self
            .tx
            .as_ref()
            .ok_or(HyperliquidWsError::Connection(
                "Output channel not initialized".to_string(),
            ))?
            .clone();

        let instrument = self
            .instruments_cache
            .get(&full_symbol_ustr)
            .unwrap()
            .clone();

        // Create subscription channel
        let (sdk_tx, mut sdk_rx) = mpsc::unbounded_channel();

        // Subscribe via SDK
        {
            let mut client = info_client.lock().await;
            let subscription = Subscription::L2Book { coin: coin.clone() };
            client
                .subscribe(subscription, sdk_tx)
                .await
                .map_err(HyperliquidWsError::Sdk)?;
        }

        // Spawn task to forward messages
        let signal = self.signal.clone();
        let task_handle = tokio::spawn(async move {
            while let Some(message) = sdk_rx.recv().await {
                if signal.load(Ordering::Relaxed) {
                    break;
                }

                if let Message::L2Book(book_data) = message {
                    let ts_init = UnixNanos::default();

                    match parse_l2_book_data(
                        &book_data.data,
                        instrument_id,
                        instrument.price_precision(),
                        instrument.size_precision(),
                        &BookAction::Add,
                        ts_init,
                    ) {
                        Ok(deltas) => {
                            let nautilus_msg = NautilusWsMessage::OrderBookDeltas(deltas);
                            if let Err(e) = main_tx.send(nautilus_msg) {
                                error!("Failed to send parsed message: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse L2Book message: {}", e);
                        }
                    }
                }
            }
        });

        // Store task handle
        {
            let mut handles = self.subscription_handles.lock().await;
            handles.push(task_handle);
        }

        // Update subscriptions tracking
        {
            let mut subscriptions = self.subscriptions.lock().await;
            subscriptions
                .entry(HyperliquidWsChannel::L2Book)
                .or_default()
                .insert(full_symbol_ustr);
        }

        debug!("Subscribed to orderbook for instrument: {}", instrument_id);
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        self.signal.store(true, Ordering::Relaxed);

        // Abort all subscription tasks
        {
            let mut handles = self.subscription_handles.lock().await;
            for handle in handles.drain(..) {
                handle.abort();
            }
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    pub fn is_active(&self) -> bool {
        self.info_client.is_some() && !self.signal.load(Ordering::Relaxed)
    }

    pub fn is_closed(&self) -> bool {
        self.signal.load(Ordering::Relaxed)
    }

    pub fn stream(&mut self) -> impl Stream<Item = NautilusWsMessage> + 'static {
        let rx = self.rx.take().expect("Data stream receiver already taken");
        let mut rx = Arc::try_unwrap(rx).expect("Cannot take ownership of receiver");

        stream! {
            while let Some(data) = rx.recv().await {
                yield data;
            }
        }
    }

    pub async fn get_subscriptions(&self) -> AHashMap<HyperliquidWsChannel, AHashSet<Ustr>> {
        let subscriptions = self.subscriptions.lock().await;
        subscriptions.clone()
    }
}
