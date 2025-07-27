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
    sync::{Arc, LazyLock, Mutex},
};

use ahash::AHashMap;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Meta, SpotMeta};
use nautilus_model::{
    identifiers::AccountId,
    instruments::{Instrument, InstrumentAny},
};
use nautilus_network::ratelimiter::quota::Quota;
use ustr::Ustr;

use super::error::{HyperliquidHttpError, Result};
use crate::common::parse::{
    parse_instruments_from_meta, parse_instruments_from_spot_meta,
    parse_instruments_from_spot_meta_with_asset_ids,
};

/// Default Hyperliquid REST API rate limit: 20 requests per second.
/// Based on Hyperliquid API documentation.
pub static HYPERLIQUID_REST_QUOTA: LazyLock<Quota> =
    LazyLock::new(|| Quota::per_second(NonZeroU32::new(20).unwrap()));

pub struct HyperliquidHttpInnerClient {
    pub(crate) account_id: AccountId,
    pub(crate) base_url: BaseUrl,
    info_client: InfoClient,
}

impl Debug for HyperliquidHttpInnerClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HyperliquidHttpInnerClient")
            .field("account_id", &self.account_id)
            .finish_non_exhaustive()
    }
}


impl HyperliquidHttpInnerClient {
    pub async fn new(account_id: AccountId, base_url: Option<BaseUrl>) -> Result<Self> {
        let base_url = base_url.unwrap_or(BaseUrl::Mainnet);
        let info_client = InfoClient::new(None, Some(base_url))
            .await
            .map_err(HyperliquidHttpError::SdkError)?;

        Ok(Self {
            account_id,
            base_url,
            info_client,
        })
    }

    pub async fn meta(&self) -> Result<Meta> {
        self.info_client
            .meta()
            .await
            .map_err(HyperliquidHttpError::SdkError)
    }

    pub async fn spot_meta(&self) -> Result<SpotMeta> {
        self.info_client
            .spot_meta()
            .await
            .map_err(HyperliquidHttpError::SdkError)
    }

    pub async fn all_mids(&self) -> Result<AHashMap<String, String>> {
        let mids = self
            .info_client
            .all_mids()
            .await
            .map_err(HyperliquidHttpError::SdkError)?;

        Ok(mids.into_iter().collect())
    }
}

#[cfg_attr(
    feature = "python",
    pyo3::pyclass(module = "nautilus_trader.core.nautilus_pyo3.hyperliquid")
)]
pub struct HyperliquidHttpClient {
    pub(crate) inner: HyperliquidHttpInnerClient,
    instruments_cache: Arc<Mutex<AHashMap<Ustr, InstrumentAny>>>,
    pub(crate) cache_initialized: bool,
}

impl Debug for HyperliquidHttpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HyperliquidHttpClient")
            .field("inner", &self.inner)
            .finish_non_exhaustive()
    }
}

impl HyperliquidHttpClient {
    pub async fn new(account_id: AccountId, base_url: Option<BaseUrl>) -> Result<Self> {
        let inner = HyperliquidHttpInnerClient::new(account_id, base_url).await?;
        let instruments_cache = Arc::new(Mutex::new(AHashMap::new()));

        Ok(Self {
            inner,
            instruments_cache,
            cache_initialized: false,
        })
    }

    pub async fn meta(&self) -> Result<Meta> {
        self.inner.meta().await
    }

    pub async fn spot_meta(&self) -> Result<SpotMeta> {
        self.inner.spot_meta().await
    }

    pub async fn all_mids(&self) -> Result<AHashMap<String, String>> {
        self.inner.all_mids().await
    }

    pub async fn instruments(&mut self) -> Result<Vec<InstrumentAny>> {
        let mut instruments = Vec::new();

        let meta = self.inner.meta().await?;
        let perp_instruments = parse_instruments_from_meta(meta)?;
        instruments.extend(perp_instruments);

        let spot_meta = self.inner.spot_meta().await?;
        let spot_instruments = parse_instruments_from_spot_meta(spot_meta)?;
        instruments.extend(spot_instruments);

        self.add_cached_instruments(&instruments);

        Ok(instruments)
    }

    pub async fn instruments_with_asset_ids(
        &mut self,
    ) -> Result<(Vec<InstrumentAny>, AHashMap<Ustr, String>)> {
        let mut instruments = Vec::new();
        let mut asset_id_map = AHashMap::new();

        // Parse perp instruments
        let meta = self.inner.meta().await?;
        let perp_instruments = parse_instruments_from_meta(meta)?;

        // Add perp asset IDs (use coin name)
        for inst in &perp_instruments {
            let symbol_str = inst.symbol().to_string();
            let coin = symbol_str.split('-').next().unwrap_or(&symbol_str);
            asset_id_map.insert(inst.symbol().inner(), coin.to_string());
        }

        instruments.extend(perp_instruments);

        // Parse spot instruments with asset IDs
        let spot_meta = self.inner.spot_meta().await?;
        let (spot_instruments, spot_asset_ids) =
            parse_instruments_from_spot_meta_with_asset_ids(spot_meta)?;
        instruments.extend(spot_instruments);
        asset_id_map.extend(spot_asset_ids);

        self.add_cached_instruments(&instruments);

        Ok((instruments, asset_id_map))
    }



    #[must_use]
    pub const fn is_initialized(&self) -> bool {
        self.cache_initialized
    }

    #[must_use]
    pub fn get_cached_symbols(&self) -> Vec<String> {
        self.instruments_cache
            .lock()
            .unwrap()
            .keys()
            .map(std::string::ToString::to_string)
            .collect()
    }

    fn add_cached_instruments(&mut self, instruments: &[InstrumentAny]) {
        let mut cache = self.instruments_cache.lock().unwrap();
        for instrument in instruments {
            cache.insert(instrument.id().symbol.inner(), instrument.clone());
        }
        self.cache_initialized = true;
    }
}
