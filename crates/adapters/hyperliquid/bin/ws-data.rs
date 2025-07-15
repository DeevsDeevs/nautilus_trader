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
    use nautilus_hyperliquid::websocket::client::HyperliquidWebSocketClient;
    use nautilus_model::identifiers::AccountId;

    // Initialize tracing for debug logs
    tracing_subscriber::fmt::init();

    println!("Testing structured Hyperliquid WebSocket client...");

    // Create account ID (dummy for public data)
    let account_id = AccountId::new("HYPERLIQUID-001");

    // Create the structured WebSocket client
    let mut client = HyperliquidWebSocketClient::new(
        account_id,
        Some(BaseUrl::Mainnet),
        None, // No instruments cache for this test
    );

    // Connect to WebSocket
    println!("Connecting to Hyperliquid...");
    client.connect().await?;

    // Subscribe to BTC orderbook
    println!("Subscribing to BTC orderbook...");
    client.subscribe_order_book("BTC".to_string()).await?;

    // Get current subscriptions
    let subscriptions = client.get_subscriptions().await;
    println!("Active subscriptions: {:?}", subscriptions);

    // Wait and monitor for actual messages
    println!("Monitoring for WebSocket messages (5 seconds)...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // Shutdown gracefully
    client.close().await?;

    println!("Test completed successfully!");
    Ok(())
}
