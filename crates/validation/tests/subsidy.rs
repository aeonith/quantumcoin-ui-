use qc_validation::*;

fn spec() -> ChainSpec { 
    toml::from_str(include_str!("../../../chain_spec.toml")).unwrap() 
}

#[test]
fn supply_never_exceeds_cap() {
    let spec = spec();
    let mut total: i128 = 0;
    let hal = spec.supply.halving_interval_blocks;
    
    // Calculate total emission over all eras
    for era in 0..64 {
        let sub = block_subsidy(&spec, era * hal);
        total += (sub as i128) * (hal as i128);
        
        // Early exit if subsidy becomes 0
        if sub == 0 {
            break;
        }
    }
    
    println!("Total emission: {} sats", total);
    println!("Supply cap: {} sats", spec.supply.max_supply_sats);
    
    // Verify total never exceeds cap
    assert!(total <= spec.supply.max_supply_sats as i128);
}

#[test]
fn block_subsidy_halving_schedule() {
    let spec = spec();
    let hal = spec.supply.halving_interval_blocks;
    
    // Test initial subsidy
    let s0 = block_subsidy(&spec, 0);
    assert!(s0 > 0);
    
    // Test first halving
    let s1 = block_subsidy(&spec, hal);
    assert_eq!(s1, s0 / 2);
    
    // Test second halving
    let s2 = block_subsidy(&spec, 2 * hal);
    assert_eq!(s2, s0 / 4);
    
    // Test that subsidy eventually becomes 0
    let s_late = block_subsidy(&spec, 64 * hal);
    assert_eq!(s_late, 0);
    
    println!("Era 0 subsidy: {} sats", s0);
    println!("Era 1 subsidy: {} sats", s1);
    println!("Era 2 subsidy: {} sats", s2);
}

#[test]
fn emission_rate_decreases() {
    let spec = spec();
    let hal = spec.supply.halving_interval_blocks;
    
    let mut prev_subsidy = block_subsidy(&spec, 0);
    
    for era in 1..10 {
        let current_subsidy = block_subsidy(&spec, era * hal);
        
        // Each era should have lower subsidy
        assert!(current_subsidy <= prev_subsidy);
        
        prev_subsidy = current_subsidy;
        
        if current_subsidy == 0 {
            break;
        }
    }
}
