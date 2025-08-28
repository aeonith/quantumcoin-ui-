/// Compact target encoding/decoding (Bitcoin-style) for difficulty bits
pub fn bits_to_target(bits: u32) -> u128 {
    let exponent = (bits >> 24) as u32;
    let mantissa = bits & 0x007fffff;
    
    if exponent <= 3 {
        ((mantissa as u128) >> (8 * (3 - exponent)))
    } else {
        (mantissa as u128) << (8 * (exponent - 3))
    }
}

pub fn target_to_bits(target: u128) -> u32 {
    if target == 0 {
        return 0;
    }
    
    // Find the most significant byte
    let mut size = 0u32;
    let mut tmp = target;
    while tmp > 0 {
        tmp >>= 8;
        size += 1;
    }
    
    let mut compact: u32;
    if size <= 3 {
        let mantissa = (target << (8 * (3 - size))) as u32;
        compact = mantissa & 0x007fffff;
    } else {
        let mantissa = (target >> (8 * (size - 3))) as u32;
        compact = mantissa & 0x007fffff;
    }
    
    compact |= size << 24;
    compact
}

/// Calculate next difficulty target using simplified algorithm
pub fn next_difficulty_target(prev_target: u128, actual_timespan: u64, target_timespan: u64) -> u128 {
    // Clamp adjustment to 4x in either direction
    let adjusted_timespan = actual_timespan.clamp(target_timespan / 4, target_timespan * 4);
    
    // new_target = prev_target * actual_time / target_time
    let new_target = (prev_target as u128 * adjusted_timespan as u128) / target_timespan as u128;
    
    // Clamp to reasonable bounds
    new_target.clamp(1, u128::MAX >> 32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bits_target_roundtrip() {
        let original_bits = 0x1d00ffff;
        let target = bits_to_target(original_bits);
        let reconstructed_bits = target_to_bits(target);
        
        // Should be approximately equal (some precision loss is expected)
        assert!((original_bits as i64 - reconstructed_bits as i64).abs() < 256);
    }

    #[test]
    fn test_difficulty_adjustment() {
        let initial_target = bits_to_target(0x1d00ffff);
        
        // If blocks come too fast, difficulty should increase (target decrease)
        let faster_target = next_difficulty_target(initial_target, 300, 600);
        assert!(faster_target < initial_target);
        
        // If blocks come too slow, difficulty should decrease (target increase)  
        let slower_target = next_difficulty_target(initial_target, 1200, 600);
        assert!(slower_target > initial_target);
        
        // If timing is perfect, target should stay roughly the same
        let same_target = next_difficulty_target(initial_target, 600, 600);
        assert_eq!(same_target, initial_target);
    }

    #[test]
    fn test_target_bounds() {
        // Test minimum target
        let min_bits = target_to_bits(1);
        assert!(min_bits > 0);
        
        // Test maximum reasonable target
        let max_target = u128::MAX >> 32;
        let max_bits = target_to_bits(max_target);
        assert!(bits_to_target(max_bits) <= max_target);
    }
}
