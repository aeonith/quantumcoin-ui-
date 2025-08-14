import type { NextApiRequest, NextApiResponse } from "next";

// PERFECT MARKET-DRIVEN PRICING ENGINE
function calculatePerfectMarketPrice(totalSupply: number, exchangeFloat: number, btcUsd: number, tradingVolume: number = 1000): number {
  // BASE MARKET ECONOMICS
  const maxSupply = 22000000;
  const initialExchangeFloat = 250000;
  
  // SCARCITY PRESSURE (exponential curve as supply decreases)
  const remainingSupply = maxSupply - totalSupply;
  const scarcityRatio = remainingSupply / maxSupply;
  const scarcityPressure = Math.pow(1 - scarcityRatio, 2) * 15 + 1; // Exponential scarcity premium
  
  // DEMAND PRESSURE (exchange availability vs demand)
  const availabilityRatio = exchangeFloat / initialExchangeFloat;
  const demandPressure = Math.pow(1 / Math.max(0.1, availabilityRatio), 1.5); // Lower availability = higher demand
  
  // BTC CORRELATION (Bitcoin price influences crypto market)
  const btcInfluence = Math.max(0.5, Math.min(3, btcUsd / 50000));
  
  // TRADING VOLUME INFLUENCE (higher volume = price discovery)
  const volumeMultiplier = Math.max(0.8, Math.min(2, tradingVolume / 1000));
  
  // NETWORK EFFECT (more users = higher value)
  const networkEffect = Math.max(1, Math.log10(totalSupply / 1000) * 0.5);
  
  // TIME-BASED APPRECIATION (halving every 2 years like Bitcoin)
  const timeMultiplier = Math.pow(1.41, Math.floor((Date.now() - new Date('2025-01-01').getTime()) / (1000 * 60 * 60 * 24 * 365 * 2))); // 41% increase every 2 years
  
  // PERFECT MARKET FORMULA
  const basePrice = 0.005; // Lower base for realistic growth
  const marketPrice = basePrice * 
                     scarcityPressure * 
                     demandPressure * 
                     btcInfluence * 
                     volumeMultiplier * 
                     networkEffect * 
                     timeMultiplier;
  
  // PRICE FLOOR AND CEILING
  const priceFloor = 0.001; // Minimum viable price
  const priceCeiling = btcUsd * 0.001; // Never exceed 0.1% of BTC price
  
  return Math.max(priceFloor, Math.min(priceCeiling, marketPrice));
}

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  try {
    // Fetch BTC price from CoinGecko
    const response = await fetch(
      "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd",
      {
        headers: {
          "Accept": "application/json",
          "User-Agent": "QuantumCoin-Exchange/1.0"
        }
      }
    );

    if (!response.ok) {
      throw new Error(`CoinGecko API error: ${response.status}`);
    }

    const data = await response.json();
    const btcUsd = data?.bitcoin?.usd;

    if (typeof btcUsd !== 'number') {
      throw new Error("Invalid price data from CoinGecko");
    }

    // Get current supply and exchange data for PERFECT market pricing
    const totalSupply = 1250000; // This would come from blockchain in production
    const exchangeFloat = Number(process.env.EXCHANGE_AVAILABLE_FLOAT || "250000");
    const tradingVolume = 850 + Math.random() * 300; // Simulated trading volume
    
    // Calculate PERFECT market-driven QTC price
    const qtcMarketPrice = calculatePerfectMarketPrice(totalSupply, exchangeFloat, btcUsd, tradingVolume);

    res.status(200).json({ 
      btcUsd: btcUsd,
      qtcUsd: qtcMarketPrice,
      totalSupply,
      exchangeFloat,
      tradingVolume,
      scarcityLevel: (22000000 - totalSupply) / 22000000,
      demandLevel: (250000 - exchangeFloat) / 250000,
      btcInfluenceLevel: Math.min(3, btcUsd / 50000),
      networkEffect: Math.log10(totalSupply / 1000) * 0.5,
      timestamp: new Date().toISOString(),
      source: "perfect-market-engine",
      priceFormula: "basePrice × scarcity² × demand^1.5 × btcInfluence × volume × network × time",
      priceFactors: {
        scarcityPressure: Math.pow(1 - (22000000 - totalSupply) / 22000000, 2) * 15 + 1,
        demandPressure: Math.pow(1 / Math.max(0.1, exchangeFloat / 250000), 1.5),
        btcCorrelation: Math.max(0.5, Math.min(3, btcUsd / 50000)),
        volumeMultiplier: Math.max(0.8, Math.min(2, tradingVolume / 1000))
      }
    });

  } catch (error: any) {
    console.error("Price calculation error:", error);
    
    // Fallback pricing
    res.status(200).json({ 
      btcUsd: 95000,
      qtcUsd: 0.25, // Fallback market price
      totalSupply: 1250000,
      exchangeFloat: Number(process.env.EXCHANGE_AVAILABLE_FLOAT || "250000"),
      timestamp: new Date().toISOString(),
      source: "fallback",
      error: error.message
    });
  }
}
