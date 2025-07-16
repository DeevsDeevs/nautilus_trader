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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HyperliquidBookLevel {
    pub px: String,
    pub sz: String,
    pub n: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HyperliquidL2BookMsg {
    pub coin: String,
    pub time: u64,
    pub levels: Vec<Vec<HyperliquidBookLevel>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HyperliquidTradeMsg {
    pub coin: String,
    pub side: String,
    pub px: String,
    pub sz: String,
    pub time: u64,
    pub hash: String,
    pub tid: u64,
    pub users: (String, String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HyperliquidBboMsg {
    pub coin: String,
    pub time: u64,
    pub levels: Vec<Vec<HyperliquidBookLevel>>,
}
