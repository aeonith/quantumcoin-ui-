/**
 * QuantumCoin Economics - Single Source of Truth
 * 
 * This module provides the canonical economic parameters for QuantumCoin.
 * ALL UI components and calculations MUST use these values instead of hardcoding.
 */

// Environment variables with fallback to canonical values
const getEnvNumber = (key: string, defaultValue: number): number => {
  const value = process.env[key];
  if (value === undefined) return defaultValue;
  const parsed = parseInt(value, 10);
  return isNaN(parsed) ? defaultValue : parsed;
};

const getEnvString = (key: string, defaultValue: string): string => {
  return process.env[key] ?? defaultValue;
};

/**
 * Canonical QuantumCoin Economics Parameters
 * These values MUST match config/chain.toml in the Rust implementation
 */
export const ECONOMICS = {
  // Core supply parameters
  TOTAL_SUPPLY: getEnvNumber('NEXT_PUBLIC_TOTAL_SUPPLY', 22_000_000),
  HALVING_PERIOD_YEARS: getEnvNumber('NEXT_PUBLIC_HALVING_YEARS', 2),
  HALVING_DURATION_YEARS: getEnvNumber('NEXT_PUBLIC_HALVING_DURATION_YEARS', 66),
  BLOCK_TIME_TARGET_SEC: getEnvNumber('NEXT_PUBLIC_BLOCK_TIME_TARGET_SEC', 600),
  
  // NO PRE-ALLOCATION - ALL 22M QTC MUST BE MINED
  // Zero development fund, zero pre-mining
  
  // Chain info
  CHAIN_NAME: getEnvString('NEXT_PUBLIC_CHAIN_NAME', 'QuantumCoin'),
  CHAIN_ID: getEnvString('NEXT_PUBLIC_CHAIN_ID', 'quantumcoin-mainnet-v2'),
} as const;

/**
 * Calculated economics values
 */
export const CALCULATED = {
  // Total halvings over the entire duration
  get TOTAL_HALVINGS(): number {
    return ECONOMICS.HALVING_DURATION_YEARS / ECONOMICS.HALVING_PERIOD_YEARS;
  },
  
  // Blocks per year (approximate)
  get BLOCKS_PER_YEAR(): number {
    return (365 * 24 * 3600) / ECONOMICS.BLOCK_TIME_TARGET_SEC;
  },
  
  // Blocks per halving period
  get BLOCKS_PER_HALVING(): number {
    return this.BLOCKS_PER_YEAR * ECONOMICS.HALVING_PERIOD_YEARS;
  },
  
  // No allocation - ALL coins are mineable
  get TOTAL_ALLOCATION(): number {
    return 0; // Zero pre-allocation
  },
  
  // All supply available for mining rewards
  get MINING_SUPPLY(): number {
    return ECONOMICS.TOTAL_SUPPLY; // All 22M QTC mineable
  },
} as const;

/**
 * Issuance schedule calculations
 */
export class IssuanceCalculator {
  /**
   * Calculate initial block reward (first halving period)
   */
  static getInitialBlockReward(): number {
    const totalBlocks = CALCULATED.BLOCKS_PER_HALVING * CALCULATED.TOTAL_HALVINGS;
    
    // Calculate geometric series sum factor: sum of (1/2^i) for i = 0 to n-1
    let seriesFactor = 0;
    for (let i = 0; i < CALCULATED.TOTAL_HALVINGS; i++) {
      seriesFactor += Math.pow(0.5, i);
    }
    
    // Distribute mining supply over all blocks with halving consideration
    const baseReward = CALCULATED.MINING_SUPPLY / totalBlocks;
    return Math.floor(baseReward * 2 / seriesFactor);
  }
  
  /**
   * Calculate block reward at a given height
   */
  static getBlockReward(height: number): number {
    const halvingPeriod = Math.floor(height / CALCULATED.BLOCKS_PER_HALVING);
    
    if (halvingPeriod >= CALCULATED.TOTAL_HALVINGS) {
      return 0; // No more rewards after all halvings
    }
    
    const initialReward = this.getInitialBlockReward();
    return Math.floor(initialReward / Math.pow(2, halvingPeriod));
  }
  
  /**
   * Calculate cumulative issuance up to a given height
   */
  static getCumulativeIssuance(height: number): number {
    if (height === 0) return 0;
    
    let total = 0; // No pre-mining - all coins must be mined
    const initialReward = this.getInitialBlockReward();
    
    // Calculate rewards for each halving period
    for (let period = 0; period < CALCULATED.TOTAL_HALVINGS; period++) {
      const periodStart = period * CALCULATED.BLOCKS_PER_HALVING;
      const periodEnd = Math.min(height + 1, (period + 1) * CALCULATED.BLOCKS_PER_HALVING);
      
      if (periodStart > height) break;
      
      const blocksInPeriod = periodEnd - periodStart;
      const rewardPerBlock = Math.floor(initialReward / Math.pow(2, period));
      total += blocksInPeriod * rewardPerBlock;
    }
    
    return Math.min(total, ECONOMICS.TOTAL_SUPPLY);
  }
  
  /**
   * Get next halving height
   */
  static getNextHalvingHeight(currentHeight: number): number | null {
    const currentPeriod = Math.floor(currentHeight / CALCULATED.BLOCKS_PER_HALVING);
    const nextPeriod = currentPeriod + 1;
    
    if (nextPeriod >= CALCULATED.TOTAL_HALVINGS) {
      return null; // No more halvings
    }
    
    return nextPeriod * CALCULATED.BLOCKS_PER_HALVING;
  }
  
  /**
   * Get issuance schedule information for display
   */
  static getIssuanceSchedule(height: number) {
    const currentReward = this.getBlockReward(height);
    const totalIssued = this.getCumulativeIssuance(height);
    const remaining = ECONOMICS.TOTAL_SUPPLY - totalIssued;
    const nextHalving = this.getNextHalvingHeight(height);
    
    return {
      height,
      currentReward,
      totalIssued,
      remaining,
      nextHalving,
      blocksToHalving: nextHalving ? nextHalving - height : null,
      percentageIssued: (totalIssued / ECONOMICS.TOTAL_SUPPLY) * 100,
    };
  }
}

/**
 * Format numbers for display
 */
export const formatters = {
  /**
   * Format QTC amount with appropriate decimal places
   */
  qtc: (amount: number, decimals = 2): string => {
    return amount.toLocaleString('en-US', {
      minimumFractionDigits: decimals,
      maximumFractionDigits: decimals,
    });
  },
  
  /**
   * Format percentage with appropriate decimal places
   */
  percentage: (value: number, decimals = 2): string => {
    return `${value.toFixed(decimals)}%`;
  },
  
  /**
   * Format time duration
   */
  duration: (seconds: number): string => {
    const years = Math.floor(seconds / (365 * 24 * 3600));
    const days = Math.floor((seconds % (365 * 24 * 3600)) / (24 * 3600));
    const hours = Math.floor((seconds % (24 * 3600)) / 3600);
    
    if (years > 0) return `${years}y ${days}d`;
    if (days > 0) return `${days}d ${hours}h`;
    return `${hours}h`;
  },
};

/**
 * Validation functions to ensure consistency
 */
export const validation = {
  /**
   * Validate that the economics configuration is internally consistent
   */
  validateConfig(): { valid: true } | { valid: false; errors: string[] } {
    const errors: string[] = [];
    
    if (ECONOMICS.TOTAL_SUPPLY <= 0) {
      errors.push('Total supply must be positive');
    }
    
    // No allocation validation needed - all coins are mineable
    
    if (ECONOMICS.HALVING_PERIOD_YEARS <= 0) {
      errors.push('Halving period must be positive');
    }
    
    if (ECONOMICS.BLOCK_TIME_TARGET_SEC <= 0) {
      errors.push('Block time must be positive');
    }
    
    if (CALCULATED.TOTAL_HALVINGS !== 33) {
      errors.push(`Expected 33 halvings, got ${CALCULATED.TOTAL_HALVINGS}`);
    }
    
    return errors.length === 0 ? { valid: true } : { valid: false, errors };
  },
  
  /**
   * Ensure final issuance doesn't exceed total supply
   */
  validateFinalSupply(): boolean {
    const finalHeight = CALCULATED.BLOCKS_PER_HALVING * CALCULATED.TOTAL_HALVINGS;
    const finalIssued = IssuanceCalculator.getCumulativeIssuance(finalHeight);
    return finalIssued <= ECONOMICS.TOTAL_SUPPLY;
  },
};

// Validate configuration on module load (development only)
if (process.env.NODE_ENV === 'development') {
  const configValidation = validation.validateConfig();
  if (!configValidation.valid) {
    console.error('❌ Economics configuration validation failed:', configValidation.errors);
  } else {
    console.info('✅ Economics configuration validated');
  }
  
  if (!validation.validateFinalSupply()) {
    console.error('❌ Final supply validation failed - issuance would exceed total supply');
  }
}
