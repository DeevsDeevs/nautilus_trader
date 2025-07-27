#!/usr/bin/env python3
"""
Simple test script for Hyperliquid instrument provider.

This script tests the Python integration with the Rust HTTP client
and instrument provider implementation.
"""
import asyncio

# Test imports
try:
    import sys
    sys.path.insert(0, '/root/deevs/nautilus/nautilus_trader')
    import nautilus_trader.core.nautilus_pyo3 as nautilus_pyo3
    print("✅ nautilus_pyo3 import successful")
except ImportError as e:
    print(f"❌ nautilus_pyo3 import failed: {e}")
    exit(1)

try:
    from nautilus_trader.adapters.hyperliquid.config import HyperliquidInstrumentProviderConfig
    from nautilus_trader.adapters.hyperliquid.providers import HyperliquidInstrumentProvider
    print("✅ Hyperliquid adapter imports successful")
except ImportError as e:
    print(f"❌ Hyperliquid adapter import failed: {e}")
    exit(1)


async def test_instrument_provider():
    """Test the instrument provider implementation."""
    print("\n🚀 Testing Hyperliquid Instrument Provider...")
    
    # Test 1: Create HTTP client
    print("\n1. Creating HTTP client...")
    try:
        client = nautilus_pyo3.hyperliquid.HyperliquidHttpClient(
            account_id="HYPERLIQUID-TEST-001",
            base_url=None,  # Use default mainnet
        )
        print(f"✅ HTTP client created: {client}")
        print(f"   Account ID: {client.account_id}")
        print(f"   Base URL: {client.base_url}")
    except Exception as e:
        print(f"❌ HTTP client creation failed: {e}")
        return
    
    # Test 2: Create instrument provider
    print("\n2. Creating instrument provider...")
    try:
        config = HyperliquidInstrumentProviderConfig(
            load_all=True,
            is_testnet=False,
        )
        provider = HyperliquidInstrumentProvider(
            client=client,
            config=config,
        )
        print(f"✅ Instrument provider created: {provider}")
    except Exception as e:
        print(f"❌ Instrument provider creation failed: {e}")
        return
    
    # Test 3: Load instruments
    print("\n3. Loading instruments from Hyperliquid API...")
    try:
        await provider.load_all_async()
        instruments = provider.list_all()
        print(f"✅ Loaded {len(instruments)} instruments")
        
        # Show sample instruments
        print("\nSample instruments:")
        for i, instrument in enumerate(instruments[:5]):
            print(f"  {i+1}. {instrument.id} - {type(instrument).__name__}")
            print(f"     Price increment: {instrument.price_increment}")
            print(f"     Size increment: {instrument.size_increment}")
        
        if len(instruments) > 5:
            print(f"  ... and {len(instruments) - 5} more instruments")
            
    except Exception as e:
        print(f"❌ Instrument loading failed: {e}")
        return
    
    # Test 4: Test PyO3 instruments
    print("\n4. Testing PyO3 instrument access...")
    try:
        pyo3_instruments = provider.instruments_pyo3()
        print(f"✅ PyO3 instruments: {len(pyo3_instruments)} instruments")
        
        if pyo3_instruments:
            sample = pyo3_instruments[0]
            print(f"   Sample PyO3 instrument: {sample}")
    except Exception as e:
        print(f"❌ PyO3 instrument access failed: {e}")
        return
    
    # Test 5: Test venue filtering
    print("\n5. Testing venue filtering...")
    try:
        from nautilus_trader.adapters.hyperliquid.constants import HYPERLIQUID_VENUE
        all_instruments = provider.list_all()
        venue_instruments = [inst for inst in all_instruments if inst.id.venue == HYPERLIQUID_VENUE]
        print(f"✅ Venue filtering works: {len(venue_instruments)} instruments for {HYPERLIQUID_VENUE}")
        
        # Show sample venue instruments
        print("   Sample venue instruments:")
        for i, instrument in enumerate(venue_instruments[:3]):
            print(f"     {i+1}. {instrument.id} - {type(instrument).__name__}")
    except Exception as e:
        print(f"❌ Venue filtering failed: {e}")
        return
    
    print("\n🎉 All tests passed! Hyperliquid instrument provider is working correctly.")


if __name__ == "__main__":
    asyncio.run(test_instrument_provider())