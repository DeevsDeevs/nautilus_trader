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

use hyperliquid_rust_sdk::BaseUrl;
use nautilus_core::python::to_pyvalue_err;
use nautilus_model::{
    identifiers::AccountId,
    python::instruments::instrument_any_to_pyobject,
};
use pyo3::{prelude::*, types::{PyDict, PyList}};

use crate::http::client::HyperliquidHttpClient;

#[pymethods]
impl HyperliquidHttpClient {
    #[new]
    #[pyo3(signature = (account_id, base_url=None))]
    fn py_new(account_id: String, base_url: Option<String>) -> PyResult<Self> {
        let account_id = AccountId::new(&account_id);
        let base_url = base_url.map(|url| {
            if url.contains("testnet") {
                BaseUrl::Testnet
            } else {
                BaseUrl::Mainnet
            }
        });

        pyo3_async_runtimes::tokio::get_runtime()
            .block_on(async { Self::new(account_id, base_url).await })
            .map_err(to_pyvalue_err)
    }

    #[pyo3(name = "request_instruments")]
    fn py_request_instruments<'py>(
        &mut self,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        // Use runtime to block on the async operation 
        let instruments = pyo3_async_runtimes::tokio::get_runtime()
            .block_on(async { self.instruments().await })
            .map_err(to_pyvalue_err)?;

        let py_instruments: PyResult<Vec<_>> = instruments
            .into_iter()
            .map(|inst| instrument_any_to_pyobject(py, inst))
            .collect();
        let pylist = PyList::new(py, py_instruments?)
            .unwrap()
            .into_any();
        Ok(pylist)
    }

    #[pyo3(name = "request_all_mids")]
    fn py_request_all_mids<'py>(
        &self,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        // Use runtime to block on the async operation
        let mids = pyo3_async_runtimes::tokio::get_runtime()
            .block_on(async { self.all_mids().await })
            .map_err(to_pyvalue_err)?;

        let py_dict = PyDict::new(py);
        for (symbol, mid) in mids {
            py_dict.set_item(symbol, mid)?;
        }
        Ok(py_dict.into_any())
    }

    #[getter]
    fn account_id(&self) -> String {
        self.inner.account_id.to_string()
    }

    #[getter]
    fn base_url(&self) -> String {
        match self.inner.base_url {
            BaseUrl::Mainnet => "https://api.hyperliquid.xyz".to_string(),
            BaseUrl::Testnet => "https://api.hyperliquid-testnet.xyz".to_string(),
            BaseUrl::Localhost => "http://localhost:3001".to_string(), // For SDK compatibility only
        }
    }



    fn __repr__(&self) -> String {
        format!(
            "HyperliquidHttpClient(account_id={}, base_url={})",
            self.account_id(),
            self.base_url()
        )
    }
}

