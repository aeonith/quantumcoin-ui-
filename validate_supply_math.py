#!/usr/bin/env python3
"""
QuantumCoin Supply Validation Script
Validates that halving schedule doesn't exceed max supply cap
"""

def validate_supply_schedule():
    # Chain parameters
    MAX_SUPPLY_SATS = 2200000000000000  # 22,000,000 × 100,000,000
    HALVING_INTERVAL = 105120           # blocks (~2 years)
    INITIAL_BLOCK_REWARD = 5000000000   # 50 QTC in satoshis
    
    print("🔍 QuantumCoin Supply Schedule Validation")
    print("=" * 50)
    print(f"Max Supply: {MAX_SUPPLY_SATS:,} sats ({MAX_SUPPLY_SATS / 1e8:,.0f} QTC)")
    print(f"Halving Interval: {HALVING_INTERVAL:,} blocks")
    print(f"Initial Reward: {INITIAL_BLOCK_REWARD:,} sats ({INITIAL_BLOCK_REWARD / 1e8:.0f} QTC)")
    print()
    
    total_supply = 0
    current_reward = INITIAL_BLOCK_REWARD
    halving_count = 0
    
    print("Halving Schedule:")
    print("-" * 70)
    print(f"{'Era':<3} {'Blocks':<15} {'Reward (QTC)':<12} {'Era Supply (QTC)':<20} {'Total (QTC)':<15}")
    print("-" * 70)
    
    while current_reward > 0 and total_supply < MAX_SUPPLY_SATS:
        # Calculate supply for this era
        era_supply = HALVING_INTERVAL * current_reward
        total_supply += era_supply
        
        print(f"{halving_count + 1:<3} "
              f"{HALVING_INTERVAL:,} blocks    "
              f"{current_reward / 1e8:<11.2f} "
              f"{era_supply / 1e8:>19,.0f} "
              f"{total_supply / 1e8:>14,.0f}")
        
        # Halve the reward
        current_reward //= 2
        halving_count += 1
        
        # Safety check to prevent infinite loop
        if halving_count > 50:
            print("⚠️  Stopping at 50 halvings (safety limit)")
            break
    
    print("-" * 70)
    print(f"Final Total Supply: {total_supply:,} sats ({total_supply / 1e8:,.0f} QTC)")
    print(f"Max Supply Cap:     {MAX_SUPPLY_SATS:,} sats ({MAX_SUPPLY_SATS / 1e8:,.0f} QTC)")
    print(f"Difference:         {MAX_SUPPLY_SATS - total_supply:,} sats ({(MAX_SUPPLY_SATS - total_supply) / 1e8:,.2f} QTC)")
    print(f"Total Halvings:     {halving_count}")
    print()
    
    # Validation results
    if total_supply <= MAX_SUPPLY_SATS:
        print("✅ VALIDATION PASSED: Total supply does not exceed cap")
        utilization = (total_supply / MAX_SUPPLY_SATS) * 100
        print(f"📊 Supply Utilization: {utilization:.2f}%")
    else:
        print("❌ VALIDATION FAILED: Total supply exceeds cap!")
        excess = total_supply - MAX_SUPPLY_SATS
        print(f"⚠️  Excess: {excess:,} sats ({excess / 1e8:,.2f} QTC)")
        return False
    
    # Time calculation
    years = (halving_count * HALVING_INTERVAL * 10) / (60 * 24 * 365.25)  # 10 min blocks
    print(f"⏰ Total Distribution Period: {years:.1f} years")
    
    return True

def validate_single_halving():
    """Validate that a single halving period doesn't exceed reasonable bounds"""
    HALVING_INTERVAL = 105120
    INITIAL_REWARD = 5000000000
    
    single_era_supply = HALVING_INTERVAL * INITIAL_REWARD
    single_era_qtc = single_era_supply / 1e8
    
    print(f"\n🔬 Single Era Analysis:")
    print(f"Blocks per era: {HALVING_INTERVAL:,}")
    print(f"Reward per block: {INITIAL_REWARD / 1e8:.0f} QTC")
    print(f"Total per era: {single_era_qtc:,.0f} QTC")
    
    # Time calculation
    era_minutes = HALVING_INTERVAL * 10  # 10 min per block
    era_days = era_minutes / (60 * 24)
    era_years = era_days / 365.25
    
    print(f"Era duration: {era_years:.2f} years ({era_days:.0f} days)")
    
    return single_era_qtc

if __name__ == "__main__":
    try:
        # Run validations
        validate_single_halving()
        success = validate_supply_schedule()
        
        if success:
            print("\n🎯 RESULT: QuantumCoin supply schedule is mathematically sound!")
            print("✅ Ready for production deployment")
        else:
            print("\n❌ RESULT: Supply schedule validation failed!")
            print("⚠️  Fix required before deployment")
            
    except Exception as e:
        print(f"\n💥 Error during validation: {e}")
        print("⚠️  Manual review required")
