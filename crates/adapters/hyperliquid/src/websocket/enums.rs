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

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum HyperliquidWsChannel {
    #[serde(rename = "l2Book")]
    #[strum(serialize = "l2Book")]
    L2Book,
    Trades,
    Bbo,
    AllMids,
    Candles,
    User,
    Orders,
    UserFills,
    UserFunding,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HyperliquidSubscriptionArg {
    pub channel: HyperliquidWsChannel,
    pub coin: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NautilusWsMessage {
    OrderBookDeltas(nautilus_model::data::deltas::OrderBookDeltas),
    TradeTicks(Vec<nautilus_model::data::TradeTick>),
    QuoteTicks(Vec<nautilus_model::data::QuoteTick>),
    Heartbeat,
    Error(String),
}
