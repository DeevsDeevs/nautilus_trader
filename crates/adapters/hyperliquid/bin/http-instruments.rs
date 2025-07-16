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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use hyperliquid_rust_sdk::BaseUrl;
    use nautilus_hyperliquid::http::client::HyperliquidHttpClient;
    use nautilus_model::identifiers::AccountId;

    // Initialize tracing for debug logs
    tracing_subscriber::fmt::init();

    println!("Testing Hyperliquid HTTP client instrument fetching...");

    // Create account ID (dummy for public data)
    let account_id = AccountId::new("HYPERLIQUID-001");

    // Create the HTTP client
    let mut client = HyperliquidHttpClient::new(account_id, Some(BaseUrl::Mainnet)).await?;

    // Test meta fetching
    println!("Fetching meta...");
    let meta = client.meta().await?;
    println!("Meta universe assets: {}", meta.universe.len());
    for (i, asset) in meta.universe.iter().take(3).enumerate() {
        println!(
            "  Asset {}: name='{}' sz_decimals={}",
            i, asset.name, asset.sz_decimals
        );
    }

    // Test spot meta fetching
    println!("\nFetching spot meta...");
    let spot_meta = client.spot_meta().await?;
    println!("Spot meta universe assets: {}", spot_meta.universe.len());
    println!("Spot meta tokens: {}", spot_meta.tokens.len());
    for (i, spot_asset) in spot_meta.universe.iter().take(3).enumerate() {
        println!(
            "  Spot Asset {}: name='{}' tokens={:?}",
            i, spot_asset.name, spot_asset.tokens
        );
    }

    // Test all mids fetching
    println!("\nFetching all mids...");
    let mids = client.all_mids().await?;
    println!("All mids count: {}", mids.len());
    println!("Sample perp mids:");
    for (asset, price) in mids.iter().take(20) {
        if !asset.contains("/") && !asset.starts_with("@") {
            println!("  Perp: {} = {}", asset, price);
        }
    }
    println!("Sample spot mids:");
    for (asset, price) in mids.iter().take(20) {
        if asset.contains("/") {
            println!("  Spot: {} = {}", asset, price);
        }
    }

    // Test instrument parsing and caching
    println!("\nTesting instrument parsing and caching...");
    println!("Client initialized: {}", client.is_initialized());

    let instruments = client.instruments().await?;
    println!("Parsed {} instruments from meta", instruments.len());
    println!(
        "Client initialized after loading: {}",
        client.is_initialized()
    );
    println!(
        "Cached symbols count: {}",
        client.get_cached_symbols().len()
    );

    // Show first 5 parsed instruments
    for (i, instrument) in instruments.iter().take(5).enumerate() {
        match instrument {
            nautilus_model::instruments::InstrumentAny::CryptoPerpetual(perp) => {
                println!(
                    "  Perp {}: {} (price_increment: {}, size_increment: {})",
                    i, perp.id, perp.price_increment, perp.size_increment
                );
            }
            nautilus_model::instruments::InstrumentAny::CurrencyPair(spot) => {
                println!(
                    "  Spot {}: {} (price_increment: {}, size_increment: {})",
                    i, spot.id, spot.price_increment, spot.size_increment
                );
            }
            _ => println!("  Other {}: {:?}", i, instrument),
        }
    }

    // Show first few spot instruments
    println!("\\nFirst 5 spot instruments:");
    let mut spot_shown = 0;
    for (i, instrument) in instruments.iter().enumerate() {
        if spot_shown >= 5 {
            break;
        }
        if let nautilus_model::instruments::InstrumentAny::CurrencyPair(spot) = instrument {
            println!(
                "  Spot {}: {} (price_increment: {}, size_increment: {})",
                i, spot.id, spot.price_increment, spot.size_increment
            );
            spot_shown += 1;
        }
    }

    // Count total by type
    let total_perp_count = instruments
        .iter()
        .filter(|i| {
            matches!(
                i,
                nautilus_model::instruments::InstrumentAny::CryptoPerpetual(_)
            )
        })
        .count();
    let total_spot_count = instruments
        .iter()
        .filter(|i| {
            matches!(
                i,
                nautilus_model::instruments::InstrumentAny::CurrencyPair(_)
            )
        })
        .count();

    println!("\nTotal instruments parsed:");
    println!("  Perpetuals: {}", total_perp_count);
    println!("  Spot pairs: {}", total_spot_count);
    println!("  Total: {}", instruments.len());

    println!("\nHTTP client test completed successfully!");
    Ok(())
}
