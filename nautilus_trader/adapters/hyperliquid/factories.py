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

import asyncio
from functools import lru_cache

from nautilus_trader.adapters.hyperliquid.config import HyperliquidDataClientConfig
from nautilus_trader.adapters.hyperliquid.config import HyperliquidInstrumentProviderConfig
from nautilus_trader.adapters.hyperliquid.data import HyperliquidDataClient
from nautilus_trader.adapters.hyperliquid.providers import HyperliquidInstrumentProvider
from nautilus_trader.cache.cache import Cache
from nautilus_trader.common.component import LiveClock
from nautilus_trader.common.component import MessageBus
from nautilus_trader.core import nautilus_pyo3
from nautilus_trader.live.factories import LiveDataClientFactory


@lru_cache(1)
def get_cached_hyperliquid_http_client(
    account_id: str,
    base_url: str | None = None,
) -> nautilus_pyo3.HyperliquidHttpClient:
    """
    Cache and return a Hyperliquid HTTP client with the given parameters.

    If a cached client with matching parameters already exists, the cached client will be returned.

    Parameters
    ----------
    account_id : str
        The account ID for the client.
    base_url : str, optional
        The base URL override.

    Returns
    -------
    nautilus_pyo3.HyperliquidHttpClient

    """
    return nautilus_pyo3.HyperliquidHttpClient(
        account_id=account_id,
        base_url=base_url,
    )


@lru_cache(1)
def get_cached_hyperliquid_instrument_provider(
    client: nautilus_pyo3.HyperliquidHttpClient,
    config: HyperliquidInstrumentProviderConfig | None = None,
) -> HyperliquidInstrumentProvider:
    """
    Cache and return a Hyperliquid instrument provider.

    If a cached provider already exists, then that provider will be returned.

    Parameters
    ----------
    client : nautilus_pyo3.HyperliquidHttpClient
        The Hyperliquid HTTP client.
    config : HyperliquidInstrumentProviderConfig, optional
        The instrument provider configuration.

    Returns
    -------
    HyperliquidInstrumentProvider

    """
    return HyperliquidInstrumentProvider(
        client=client,
        config=config,
    )


class HyperliquidLiveDataClientFactory(LiveDataClientFactory):
    """
    Provides a Hyperliquid live data client factory.
    """

    @staticmethod
    def create(  # type: ignore
        loop: asyncio.AbstractEventLoop,
        name: str,
        config: HyperliquidDataClientConfig,
        msgbus: MessageBus,
        cache: Cache,
        clock: LiveClock,
    ) -> HyperliquidDataClient:
        """
        Create a new Hyperliquid data client.

        Parameters
        ----------
        loop : asyncio.AbstractEventLoop
            The event loop for the client.
        name : str
            The custom client ID.
        config : HyperliquidDataClientConfig
            The client configuration.
        msgbus : MessageBus
            The message bus for the client.
        cache : Cache
            The cache for the client.
        clock : LiveClock
            The clock for the instrument provider.

        Returns
        -------
        HyperliquidDataClient

        """
        client: nautilus_pyo3.HyperliquidHttpClient = get_cached_hyperliquid_http_client(
            account_id=name,
            base_url=config.base_url_http,
        )
        provider = get_cached_hyperliquid_instrument_provider(
            client=client,
            config=config.instrument_provider,
        )
        return HyperliquidDataClient(
            loop=loop,
            client=client,
            msgbus=msgbus,
            cache=cache,
            clock=clock,
            instrument_provider=provider,
            config=config,
            name=name,
        )
