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
    use futures_util::StreamExt;
    use hyperliquid_rust_sdk::BaseUrl;
    use nautilus_hyperliquid::{
        http::client::HyperliquidHttpClient, websocket::client::HyperliquidWebSocketClient,
    };
    use nautilus_model::{identifiers::AccountId, instruments::Instrument};

    // Initialize tracing for debug logs
    tracing_subscriber::fmt::init();

    println!("Testing structured Hyperliquid WebSocket client...");

    // Create account ID (dummy for public data)
    let account_id = AccountId::new("HYPERLIQUID-001");

    // Get instruments with asset IDs first
    println!("Fetching instruments...");
    let mut http_client = HyperliquidHttpClient::new(account_id, Some(BaseUrl::Mainnet)).await?;
    let (instruments, asset_id_map) = http_client.instruments_with_asset_ids().await?;

    // Try HYPE spot instrument (matches the SDK example using @107)
    let test_instrument = instruments
        .iter()
        .find(|inst| inst.symbol().as_str().contains("HYPE-USDC-SPOT"))
        .or_else(|| {
            instruments
                .iter()
                .find(|inst| inst.symbol().as_str().contains("PURR-USDC-SPOT"))
        })
        .or_else(|| {
            instruments
                .iter()
                .find(|inst| inst.symbol().as_str().contains("SPOT"))
        })
        .or_else(|| {
            instruments
                .iter()
                .find(|inst| inst.symbol().as_str() == "BTC-USDC-PERP")
        })
        .ok_or_else(|| anyhow::anyhow!("No test instrument found"))?;

    println!(
        "Using test instrument: {}",
        test_instrument.symbol().as_str()
    );

    // Print asset ID mappings for debugging
    println!("Asset ID mappings for spot instruments:");
    for (symbol, asset_id) in &asset_id_map {
        if symbol.to_string().contains("SPOT") {
            println!("  {} -> {}", symbol, asset_id);
        }
    }

    // Print the specific test instrument mapping
    if let Some(asset_id) = asset_id_map.get(&test_instrument.symbol().inner()) {
        println!(
            "Test instrument {} -> {}",
            test_instrument.symbol().as_str(),
            asset_id
        );
    }

    // Create the WebSocket client with instruments and asset IDs
    let mut client = HyperliquidWebSocketClient::new_with_asset_ids(
        account_id,
        Some(BaseUrl::Mainnet),
        instruments.clone(),
        asset_id_map,
    );

    // Connect to WebSocket
    println!("Connecting to Hyperliquid...");
    client.connect().await?;

    // Subscribe to test instrument orderbook
    println!("Subscribing to orderbook...");
    client.subscribe_order_book(test_instrument.id()).await?;

    // Get current subscriptions
    let subscriptions = client.get_subscriptions().await;
    println!("Active subscriptions: {:?}", subscriptions);

    // Start consuming the stream to see actual messages
    println!("Monitoring for WebSocket messages (10 seconds)...");

    let stream = client.stream();
    tokio::pin!(stream);
    let mut message_count = 0;

    let timeout = tokio::time::sleep(std::time::Duration::from_secs(10));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            msg = stream.next() => {
                if let Some(msg) = msg {
                    message_count += 1;
                    match msg {
                        nautilus_hyperliquid::websocket::enums::NautilusWsMessage::OrderBookDeltas(deltas) => {
                            println!("Message {}: Received OrderBookDeltas for {}",
                                message_count, deltas.instrument_id);
                            println!("  - {} deltas in batch", deltas.deltas.len());

                            // Show first few deltas
                            for (i, delta) in deltas.deltas.iter().take(5).enumerate() {
                                println!("    Delta {}: {} {:?} {} @ {} (flags: {})",
                                    i + 1,
                                    delta.action,
                                    delta.order.side,
                                    delta.order.size,
                                    delta.order.price,
                                    delta.flags
                                );
                            }
                            if deltas.deltas.len() > 5 {
                                println!("    ... and {} more deltas", deltas.deltas.len() - 5);
                            }
                        }
                        other => {
                            println!("Message {}: Received other message: {:?}", message_count, other);
                        }
                    }

                    if message_count >= 5 {
                        println!("Received {} messages, stopping...", message_count);
                        break;
                    }
                } else {
                    println!("Stream ended");
                    break;
                }
            }
            _ = &mut timeout => {
                println!("Timeout reached, stopping...");
                break;
            }
        }
    }

    println!("Total messages received: {}", message_count);
    Ok(())
}
