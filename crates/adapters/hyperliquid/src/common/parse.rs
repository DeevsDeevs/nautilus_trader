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

use std::{str::FromStr, sync::LazyLock};

use ahash::AHashMap;
use hyperliquid_rust_sdk::{AssetMeta, Meta, SpotAssetMeta, SpotMeta};
use nautilus_core::nanos::UnixNanos;
use nautilus_model::{
    currencies::CURRENCY_MAP,
    enums::CurrencyType,
    identifiers::{InstrumentId, Symbol, Venue},
    instruments::{CryptoPerpetual, CurrencyPair, Instrument, InstrumentAny},
    types::{Currency, Price, Quantity},
};
use rust_decimal::Decimal;
use ustr::Ustr;

use super::enums::HyperliquidInstrumentType;
use crate::http::error::{HyperliquidHttpError, Result};

static HYPERLIQUID_VENUE: LazyLock<Venue> = LazyLock::new(|| Venue::new(Ustr::from("HYPERLIQUID")));

pub fn parse_instruments_from_meta(meta: Meta) -> Result<Vec<InstrumentAny>> {
    let mut instruments = Vec::new();
    let ts_init = UnixNanos::default();

    for asset in meta.universe {
        if let Some(instrument) =
            parse_instrument_any(&asset, HyperliquidInstrumentType::Perp, ts_init)?
        {
            instruments.push(instrument);
        }
    }

    Ok(instruments)
}

pub fn parse_instruments_from_spot_meta(spot_meta: SpotMeta) -> Result<Vec<InstrumentAny>> {
    let mut instruments = Vec::new();
    let ts_init = UnixNanos::default();

    for spot_asset in &spot_meta.universe {
        if let Some(instrument) = parse_spot_instrument(spot_asset, &spot_meta, ts_init)? {
            instruments.push(instrument);
        }
    }

    Ok(instruments)
}

fn parse_instrument_any(
    asset: &AssetMeta,
    instrument_type: HyperliquidInstrumentType,
    ts_init: UnixNanos,
) -> Result<Option<InstrumentAny>> {
    match instrument_type {
        HyperliquidInstrumentType::Perp => parse_perpetual_instrument(asset, ts_init),
        HyperliquidInstrumentType::Spot => Ok(None),
    }
}

fn parse_perpetual_instrument(
    asset: &AssetMeta,
    ts_init: UnixNanos,
) -> Result<Option<InstrumentAny>> {
    let symbol = parse_symbol(&asset.name, HyperliquidInstrumentType::Perp)?;
    let instrument_id = parse_instrument_id(symbol);

    let base_currency = get_or_create_currency(&asset.name);
    let quote_currency = get_or_create_currency("USDC");

    let size_increment = calculate_size_increment(asset.sz_decimals)?;

    let price_precision = (6u32.saturating_sub(asset.sz_decimals)) as u8;
    let price_increment = calculate_price_increment(price_precision)?;

    let instrument = CryptoPerpetual::new(
        instrument_id,
        symbol,
        base_currency,
        quote_currency,
        base_currency,
        false,
        price_precision,
        asset.sz_decimals as u8, // size_precision
        price_increment,
        size_increment,
        None,                 // multiplier
        Some(size_increment), // lot_size
        None,                 // max_quantity
        None,                 // min_quantity
        None,                 // max_notional
        None,                 // min_notional
        None,                 // max_price
        None,                 // min_price
        None,                 // margin_init
        None,                 // margin_maint
        None,                 // maker_fee
        None,                 // taker_fee
        ts_init,
        ts_init,
    );

    Ok(Some(InstrumentAny::CryptoPerpetual(instrument)))
}

fn parse_spot_instrument(
    spot_asset: &SpotAssetMeta,
    spot_meta: &SpotMeta,
    ts_init: UnixNanos,
) -> Result<Option<InstrumentAny>> {
    let token_1 = spot_meta
        .tokens
        .iter()
        .find(|t| t.index == spot_asset.tokens[0])
        .ok_or_else(|| {
            HyperliquidHttpError::General(format!("Token index {} not found", spot_asset.tokens[0]))
        })?;
    let token_2 = spot_meta
        .tokens
        .iter()
        .find(|t| t.index == spot_asset.tokens[1])
        .ok_or_else(|| {
            HyperliquidHttpError::General(format!("Token index {} not found", spot_asset.tokens[1]))
        })?;

    let token_1_name = &token_1.name;
    let token_2_name = &token_2.name;
    let token_1_sz_decimals = token_1.sz_decimals;

    let symbol = parse_spot_symbol(token_1_name, token_2_name)?;
    let instrument_id = parse_instrument_id(symbol);

    let base_currency = get_or_create_currency(token_1_name);
    let quote_currency = get_or_create_currency(token_2_name);

    let price_precision = (8u32.saturating_sub(token_1_sz_decimals as u32)) as u8;
    let price_increment = calculate_price_increment(price_precision)?;

    let size_increment = calculate_size_increment(token_1_sz_decimals as u32)?;

    let instrument = CurrencyPair::new(
        instrument_id,
        symbol,
        base_currency,
        quote_currency,
        price_precision,
        token_1_sz_decimals,
        price_increment,
        size_increment,
        Some(size_increment), // lot_size
        None,                 // max_quantity
        None,                 // min_quantity
        None,                 // max_notional
        None,                 // min_notional
        None,                 // max_price
        None,                 // min_price
        None,                 // margin_init
        None,                 // margin_maint
        None,                 // maker_fee
        None,                 // taker_fee
        ts_init,
        ts_init,
    );

    Ok(Some(InstrumentAny::CurrencyPair(instrument)))
}

pub fn parse_instruments_from_spot_meta_with_asset_ids(
    spot_meta: SpotMeta,
) -> Result<(Vec<InstrumentAny>, AHashMap<Ustr, String>)> {
    let mut instruments = Vec::new();
    let mut asset_id_map = AHashMap::new();
    let ts_init = UnixNanos::default();

    for spot_asset in &spot_meta.universe {
        if let Some(instrument) = parse_spot_instrument(spot_asset, &spot_meta, ts_init)? {
            let symbol_ustr = instrument.symbol().inner();
            let asset_id = format!("@{}", spot_asset.index);
            asset_id_map.insert(symbol_ustr, asset_id);
            instruments.push(instrument);
        }
    }

    Ok((instruments, asset_id_map))
}

fn parse_symbol(name: &str, instrument_type: HyperliquidInstrumentType) -> Result<Symbol> {
    let symbol_str = match instrument_type {
        HyperliquidInstrumentType::Perp => format!("{}-USDC-PERP", name),
        HyperliquidInstrumentType::Spot => {
            format!("{}-USDC-SPOT", name)
        }
    };
    Ok(Symbol::from_str_unchecked(&symbol_str))
}

fn parse_spot_symbol(base: &str, quote: &str) -> Result<Symbol> {
    let symbol_str = format!("{}-{}-SPOT", base, quote);
    Ok(Symbol::from_str_unchecked(&symbol_str))
}

#[must_use]
pub fn parse_instrument_id(symbol: Symbol) -> InstrumentId {
    InstrumentId::new(symbol, *HYPERLIQUID_VENUE)
}

fn get_or_create_currency(code: &str) -> Currency {
    CURRENCY_MAP
        .lock()
        .unwrap()
        .get(code)
        .copied()
        .unwrap_or_else(|| Currency::new(code, 8, 0, code, CurrencyType::Crypto))
}

fn calculate_size_increment(sz_decimals: u32) -> Result<Quantity> {
    let increment = Decimal::new(1, sz_decimals);
    Quantity::from_str(&increment.to_string()).map_err(|e| {
        HyperliquidHttpError::General(format!(
            "Failed to calculate size increment from {} decimals: {}",
            sz_decimals, e
        ))
    })
}

fn calculate_price_increment(price_precision: u8) -> Result<Price> {
    let increment = Decimal::new(1, price_precision as u32);
    Price::from_str(&increment.to_string()).map_err(|e| {
        HyperliquidHttpError::General(format!(
            "Failed to calculate price increment from {} precision: {}",
            price_precision, e
        ))
    })
}
