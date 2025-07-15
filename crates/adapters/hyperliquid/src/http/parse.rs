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

#[cfg(test)]
mod tests {
    use hyperliquid_rust_sdk::{AssetMeta, Meta};
    use nautilus_model::identifiers::Symbol;
    use nautilus_model::instruments::{Instrument, InstrumentAny};

    use crate::common::parse::parse_instruments_from_meta;

    #[test]
    fn test_parse_perpetual_instruments() {
        let meta = Meta {
            universe: vec![
                AssetMeta {
                    name: "BTC".to_string(),
                    sz_decimals: 5,
                    max_leverage: 40,
                    only_isolated: None,
                },
                AssetMeta {
                    name: "ETH".to_string(),
                    sz_decimals: 4,
                    max_leverage: 25,
                    only_isolated: None,
                },
            ],
        };

        let instruments = parse_instruments_from_meta(meta).unwrap();
        assert_eq!(instruments.len(), 2);

        // Check BTC-USDC-PERP
        if let InstrumentAny::CryptoPerpetual(btc_perp) = &instruments[0] {
            assert_eq!(btc_perp.id().symbol, Symbol::from("BTC-USDC-PERP"));
            assert_eq!(btc_perp.price_precision, 1);
            assert_eq!(btc_perp.size_precision, 5);
            assert_eq!(btc_perp.base_currency.code, "BTC");
            assert_eq!(btc_perp.quote_currency.code, "USDC");
        } else {
            panic!("Expected CryptoPerpetual for BTC");
        }

        // Check ETH-USDC-PERP
        if let InstrumentAny::CryptoPerpetual(eth_perp) = &instruments[1] {
            assert_eq!(eth_perp.id().symbol, Symbol::from("ETH-USDC-PERP"));
            assert_eq!(eth_perp.price_precision, 2);
            assert_eq!(eth_perp.size_precision, 4);
            assert_eq!(eth_perp.base_currency.code, "ETH");
            assert_eq!(eth_perp.quote_currency.code, "USDC");
        } else {
            panic!("Expected CryptoPerpetual for ETH");
        }
    }

    #[test]
    fn test_parse_instrument_symbols() {
        use crate::common::parse::parse_instrument_id;
        use nautilus_model::identifiers::Venue;
        use ustr::Ustr;

        let symbol = Symbol::from("BTC-USDC-PERP");
        let instrument_id = parse_instrument_id(symbol);

        assert_eq!(instrument_id.symbol, symbol);
        assert_eq!(instrument_id.venue, Venue::new(Ustr::from("HYPERLIQUID")));
    }
}
