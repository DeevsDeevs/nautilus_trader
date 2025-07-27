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

use std::str::FromStr;

use anyhow::Result;
use hyperliquid_rust_sdk::L2BookData;
use nautilus_core::nanos::UnixNanos;
use nautilus_model::{
    data::{
        OrderBookDelta,
        deltas::OrderBookDeltas,
        order::{BookOrder, OrderId},
    },
    enums::{BookAction, OrderSide, RecordFlag},
    identifiers::InstrumentId,
    types::{Price, Quantity},
};

use super::{
    enums::{HyperliquidWsChannel, NautilusWsMessage},
    messages::HyperliquidL2BookMsg,
};

pub fn parse_l2_book_msg(
    msg: &HyperliquidL2BookMsg,
    instrument_id: InstrumentId,
    _price_precision: u8,
    _size_precision: u8,
    action: &BookAction,
    ts_init: UnixNanos,
) -> Result<OrderBookDeltas> {
    let ts_event = UnixNanos::from(msg.time * 1_000_000);

    let flags = if matches!(action, BookAction::Add) {
        RecordFlag::F_SNAPSHOT as u8
    } else {
        0
    };

    let mut deltas = Vec::new();

    // Process bids (levels[0])
    if !msg.levels.is_empty() && !msg.levels[0].is_empty() {
        for level in &msg.levels[0] {
            let price = Price::from_str(&level.px)
                .map_err(|e| anyhow::anyhow!("Price parse error: {}", e))?;
            let size = Quantity::from_str(&level.sz)
                .map_err(|e| anyhow::anyhow!("Quantity parse error: {}", e))?;

            let book_action = if matches!(action, BookAction::Update) && level.sz == "0" {
                BookAction::Delete
            } else {
                *action
            };

            let order = BookOrder::new(
                OrderSide::Buy,
                price,
                size,
                OrderId::from(0u64), // order_id not provided by Hyperliquid
            );

            let delta = OrderBookDelta::new(
                instrument_id,
                book_action,
                order,
                flags,
                0, // sequence not provided
                ts_event,
                ts_init,
            );

            deltas.push(delta);
        }
    }

    // Process asks (levels[1])
    if msg.levels.len() > 1 && !msg.levels[1].is_empty() {
        for level in &msg.levels[1] {
            let price = Price::from_str(&level.px)
                .map_err(|e| anyhow::anyhow!("Price parse error: {}", e))?;
            let size = Quantity::from_str(&level.sz)
                .map_err(|e| anyhow::anyhow!("Quantity parse error: {}", e))?;

            let book_action = if matches!(action, BookAction::Update) && level.sz == "0" {
                BookAction::Delete
            } else {
                *action
            };

            let order = BookOrder::new(
                OrderSide::Sell,
                price,
                size,
                OrderId::from(0u64), // order_id not provided by Hyperliquid
            );

            let delta = OrderBookDelta::new(
                instrument_id,
                book_action,
                order,
                flags,
                0, // sequence not provided
                ts_event,
                ts_init,
            );

            deltas.push(delta);
        }
    }

    Ok(OrderBookDeltas::new(instrument_id, deltas))
}

pub fn parse_l2_book_data(
    book_data: &L2BookData,
    instrument_id: InstrumentId,
    price_precision: u8,
    size_precision: u8,
    action: &BookAction,
    ts_init: UnixNanos,
) -> Result<OrderBookDeltas> {
    let msg = HyperliquidL2BookMsg {
        coin: book_data.coin.clone(),
        time: book_data.time,
        levels: book_data
            .levels
            .iter()
            .map(|level_vec| {
                level_vec
                    .iter()
                    .map(|level| super::messages::HyperliquidBookLevel {
                        px: level.px.clone(),
                        sz: level.sz.clone(),
                        n: level.n,
                    })
                    .collect()
            })
            .collect(),
    };

    parse_l2_book_msg(
        &msg,
        instrument_id,
        price_precision,
        size_precision,
        action,
        ts_init,
    )
}

pub fn parse_ws_message_data(
    channel: &HyperliquidWsChannel,
    data: serde_json::Value,
    instrument_id: &InstrumentId,
    price_precision: u8,
    size_precision: u8,
    ts_init: UnixNanos,
) -> Result<Option<NautilusWsMessage>> {
    match channel {
        HyperliquidWsChannel::L2Book => {
            let msg: HyperliquidL2BookMsg = serde_json::from_value(data)?;
            let deltas = parse_l2_book_msg(
                &msg,
                *instrument_id,
                price_precision,
                size_precision,
                &BookAction::Add, // Hyperliquid sends snapshots
                ts_init,
            )?;
            Ok(Some(NautilusWsMessage::OrderBookDeltas(deltas)))
        }
        _ => {
            tracing::warn!("Parsing not implemented for channel: {:?}", channel);
            Ok(None)
        }
    }
}
