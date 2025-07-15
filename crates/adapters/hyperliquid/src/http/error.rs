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

use thiserror::Error;

#[derive(Debug, Error)]
pub enum HyperliquidHttpError {
    #[error("Missing credentials")]
    MissingCredentials,
    #[error("Hyperliquid error: {0}")]
    HyperliquidError(String),
    #[error("JSON error: {0}")]
    JsonError(String),
    #[error("SDK error: {0}")]
    SdkError(#[from] hyperliquid_rust_sdk::Error),
    #[error("General error: {0}")]
    General(String),
}

pub type Result<T> = std::result::Result<T, HyperliquidHttpError>;
