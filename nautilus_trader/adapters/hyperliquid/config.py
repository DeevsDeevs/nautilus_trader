# -------------------------------------------------------------------------------------------------
#  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
#  https://nautechsystems.io
#
#  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
#  You may not use this file except in compliance with the License.
#  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
#
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.
# -------------------------------------------------------------------------------------------------

from typing import Literal

from nautilus_trader.config import InstrumentProviderConfig
from nautilus_trader.config import LiveDataClientConfig


class HyperliquidInstrumentProviderConfig(InstrumentProviderConfig, frozen=True):
    """
    Configuration for ``HyperliquidInstrumentProvider``.

    Parameters
    ----------
    load_all : bool, default True
        If all available instruments should be loaded on initialization.
    base_url : str, optional
        The HTTP base URL override.
        If None then will use the default for the configured environment.
    is_testnet : bool, default False
        If the client is connecting to the Hyperliquid testnet.

    """

    load_all: bool = True
    base_url: str | None = None
    is_testnet: bool = False


class HyperliquidDataClientConfig(LiveDataClientConfig, frozen=True):
    """
    Configuration for ``HyperliquidDataClient``.

    Parameters
    ----------
    instrument_provider : HyperliquidInstrumentProviderConfig, optional
        The instrument provider configuration.
    base_url_http : str, optional
        The HTTP API base URL override.
        If None then will use the default for the configured environment.
    base_url_ws : str, optional 
        The WebSocket base URL override.
        If None then will use the default for the configured environment.
    is_testnet : bool, default False
        If the client is connecting to the Hyperliquid testnet.

    """

    instrument_provider: HyperliquidInstrumentProviderConfig | None = None
    base_url_http: str | None = None
    base_url_ws: str | None = None
    is_testnet: bool = False
