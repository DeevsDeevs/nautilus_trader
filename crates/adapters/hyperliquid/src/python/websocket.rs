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

//! PyO3 bindings for Hyperliquid WebSocket client.

use hyperliquid_rust_sdk::BaseUrl;
use nautilus_core::python::to_pyvalue_err;
use nautilus_model::identifiers::{AccountId, InstrumentId};
use pyo3::{prelude::*, types::PyDict};

use crate::websocket::client::HyperliquidWebSocketClient;

#[cfg(feature = "python")]
#[pymethods]
impl HyperliquidWebSocketClient {
    #[new]
    fn py_new(account_id: &str, base_url: Option<&str>) -> PyResult<Self> {
        let account_id = AccountId::from(account_id);
        let base_url = match base_url {
            Some("mainnet") | None => Some(BaseUrl::Mainnet),
            Some("testnet") => Some(BaseUrl::Testnet),
            Some(url) => return Err(to_pyvalue_err(format!("Invalid base_url: {}", url))),
        };

        Ok(Self::new(account_id, base_url, None))
    }

    #[pyo3(name = "connect")]
    fn py_connect<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let mut client = self.clone();
        
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client.connect().await.map_err(to_pyvalue_err)?;
            Ok(Python::with_gil(|py| py.None()))
        })
    }

    #[pyo3(name = "subscribe_order_book")]
    fn py_subscribe_order_book<'py>(
        &self,
        py: Python<'py>,
        instrument_id: &str,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        let instrument_id = InstrumentId::from(instrument_id);
        
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .subscribe_order_book(instrument_id)
                .await
                .map_err(to_pyvalue_err)?;
            Ok(Python::with_gil(|py| py.None()))
        })
    }

    #[pyo3(name = "is_closed")]
    fn py_is_closed(&self) -> bool {
        self.is_closed()
    }

    #[pyo3(name = "get_subscriptions")]
    fn py_get_subscriptions<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let subscriptions = client.get_subscriptions().await;
            
            Python::with_gil(|py| -> PyResult<_> {
                let py_dict = PyDict::new(py);
                for (channel, symbols) in subscriptions {
                    let channel_str = format!("{:?}", channel);
                    let symbols_vec: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
                    py_dict.set_item(channel_str, symbols_vec)?;
                }
                Ok(py_dict.into_any().unbind())
            })
        })
    }

    #[getter]
    fn account_id(&self) -> String {
        self.account_id.to_string()
    }

    #[getter]
    fn base_url(&self) -> String {
        match self.base_url {
            BaseUrl::Mainnet => "https://api.hyperliquid.xyz".to_string(),
            BaseUrl::Testnet => "https://api.hyperliquid-testnet.xyz".to_string(),
            BaseUrl::Localhost => "http://localhost:3001".to_string(),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "HyperliquidWebSocketClient(account_id={}, base_url={})",
            self.account_id(),
            self.base_url()
        )
    }
}