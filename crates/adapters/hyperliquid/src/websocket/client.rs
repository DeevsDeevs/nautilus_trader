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
use dashmap::DashMap;
use futures_util::Stream;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription};
use nautilus_network::ratelimiter::quota::Quota;
use tokio::sync::{Mutex, mpsc};
use tracing::{debug, info, warn};
use ustr::Ustr;

use nautilus_model::{
    identifiers::AccountId,
    instruments::{Instrument, InstrumentAny},
};

use crate::websocket::{
    enums::{HyperliquidWsChannel, NautilusWsMessage},
    error::{HyperliquidWsError, Result},
};

/// Default Hyperliquid WebSocket rate limit: 10 requests per second.
/// Based on Hyperliquid API documentation.
pub static HYPERLIQUID_WS_QUOTA: LazyLock<Quota> =
    LazyLock::new(|| Quota::per_second(NonZeroU32::new(10).unwrap()));

/// Hyperliquid WebSocket burst rate limit: 1000 requests.
/// Maximum burst capacity before rate limiting kicks in.
pub static HYPERLIQUID_WS_BURST_QUOTA: LazyLock<Quota> =
    LazyLock::new(|| Quota::per_second(NonZeroU32::new(1000).unwrap()));

#[derive(Clone)]
pub struct HyperliquidWebSocketClient {
    base_url: BaseUrl,
    account_id: AccountId,
    info_client: Option<Arc<Mutex<InfoClient>>>,
    rx: Option<Arc<Mutex<mpsc::UnboundedReceiver<NautilusWsMessage>>>>,
    signal: Arc<AtomicBool>,
    subscriptions: Arc<Mutex<AHashMap<HyperliquidWsChannel, AHashSet<String>>>>,
    subscription_ids: Arc<DashMap<u32, (HyperliquidWsChannel, String)>>,
    instruments_cache: Arc<AHashMap<Ustr, InstrumentAny>>,
}

impl HyperliquidWebSocketClient {
    pub fn new(
        account_id: AccountId,
        base_url: Option<BaseUrl>,
        instruments: Option<Vec<InstrumentAny>>,
    ) -> Self {
        let instruments_cache = if let Some(instruments) = instruments {
            Arc::new(
                instruments
                    .into_iter()
                    .map(|inst| (inst.id().symbol.as_str().into(), inst))
                    .collect(),
            )
        } else {
            Arc::new(AHashMap::new())
        };

        Self {
            base_url: base_url.unwrap_or(BaseUrl::Mainnet),
            account_id,
            info_client: None,
            rx: None,
            signal: Arc::new(AtomicBool::new(false)),
            subscriptions: Arc::new(Mutex::new(AHashMap::new())),
            subscription_ids: Arc::new(DashMap::new()),
            instruments_cache,
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        debug!("Connecting to Hyperliquid WebSocket");

        let info_client = InfoClient::new(None, Some(self.base_url))
            .await
            .map_err(|e| HyperliquidWsError::Sdk(e))?;

        self.info_client = Some(Arc::new(Mutex::new(info_client)));

        let (_tx, rx) = mpsc::unbounded_channel();
        self.rx = Some(Arc::new(Mutex::new(rx)));

        info!("Connected to Hyperliquid WebSocket successfully");
        Ok(())
    }

    pub async fn subscribe_order_book(&mut self, coin: String) -> Result<()> {
        let info_client = self
            .info_client
            .as_ref()
            .ok_or(HyperliquidWsError::Connection("Not connected".to_string()))?
            .clone();

        debug!("Subscribing to orderbook for coin: {}", coin);

        let (tx, mut rx) = mpsc::unbounded_channel();

        let subscription_id = {
            let mut client = info_client.lock().await;
            client
                .subscribe(Subscription::L2Book { coin: coin.clone() }, tx)
                .await
                .map_err(|e| HyperliquidWsError::Sdk(e))?
        };

        {
            let mut subscriptions = self.subscriptions.lock().await;
            subscriptions
                .entry(HyperliquidWsChannel::L2Book)
                .or_insert_with(AHashSet::new)
                .insert(coin.clone());
        }

        self.subscription_ids.insert(
            subscription_id,
            (HyperliquidWsChannel::L2Book, coin.clone()),
        );

        info!(
            "Subscribed to orderbook for {} with ID: {}",
            coin, subscription_id
        );

        let _instruments_cache = self.instruments_cache.clone();
        let signal = self.signal.clone();

        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if signal.load(Ordering::Relaxed) {
                    break;
                }

                match message {
                    Message::L2Book(book_data) => {
                        debug!(
                            "Received L2Book data for {}: {} bids, {} asks, time: {}",
                            book_data.data.coin,
                            book_data.data.levels[0].len(),
                            book_data.data.levels[1].len(),
                            book_data.data.time
                        );

                        // Log first few levels for verification
                        if !book_data.data.levels[0].is_empty()
                            && !book_data.data.levels[1].is_empty()
                        {
                            debug!(
                                "  Best bid: {} @ {}, Best ask: {} @ {}",
                                book_data.data.levels[0][0].sz,
                                book_data.data.levels[0][0].px,
                                book_data.data.levels[1][0].sz,
                                book_data.data.levels[1][0].px
                            );
                        }
                    }
                    _ => {
                        warn!("Received unexpected message type: {:?}", message);
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn unsubscribe(&mut self, subscription_id: u32) -> Result<()> {
        let info_client = self
            .info_client
            .as_ref()
            .ok_or(HyperliquidWsError::Connection("Not connected".to_string()))?
            .clone();

        {
            let mut client = info_client.lock().await;
            client
                .unsubscribe(subscription_id)
                .await
                .map_err(|e| HyperliquidWsError::Sdk(e))?;
        }

        if let Some((_, (channel, coin))) = self.subscription_ids.remove(&subscription_id) {
            let mut subscriptions = self.subscriptions.lock().await;
            if let Some(coins) = subscriptions.get_mut(&channel) {
                coins.remove(&coin);
                if coins.is_empty() {
                    subscriptions.remove(&channel);
                }
            }
        }

        info!("Unsubscribed from subscription ID: {}", subscription_id);
        Ok(())
    }

    pub fn stream(&mut self) -> impl Stream<Item = NautilusWsMessage> + 'static {
        let rx = self.rx.take();
        let signal = self.signal.clone();

        stream! {
            if let Some(rx) = rx {
                let mut rx = rx.lock().await;
                while let Some(message) = rx.recv().await {
                    if signal.load(Ordering::Relaxed) {
                        break;
                    }
                    yield message;
                }
            }
        }
    }

    pub async fn close(&self) -> Result<()> {
        info!("Shutting down Hyperliquid WebSocket client");
        self.signal.store(true, Ordering::Relaxed);
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    pub async fn resubscribe_all(&mut self) -> Result<()> {
        info!("Resubscribing to all active subscriptions");

        let subscriptions = {
            let subs = self.subscriptions.lock().await;
            subs.clone()
        };

        for (channel, coins) in subscriptions {
            match channel {
                HyperliquidWsChannel::L2Book => {
                    for coin in coins {
                        self.subscribe_order_book(coin).await?;
                    }
                }
                _ => {
                    warn!("Resubscription not implemented for channel: {:?}", channel);
                }
            }
        }

        Ok(())
    }

    pub fn is_active(&self) -> bool {
        self.info_client.is_some()
    }

    pub fn is_closed(&self) -> bool {
        self.signal.load(Ordering::Relaxed)
    }

    pub async fn get_subscriptions(&self) -> AHashMap<HyperliquidWsChannel, AHashSet<String>> {
        let subscriptions = self.subscriptions.lock().await;
        subscriptions.clone()
    }
}

impl Debug for HyperliquidWebSocketClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HyperliquidWebSocketClient")
            .field("account_id", &self.account_id)
            .field("is_active", &self.is_active())
            .finish_non_exhaustive()
    }
}
