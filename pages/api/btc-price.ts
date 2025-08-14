import type { NextApiRequest, NextApiResponse } from "next";

// Calculate market-driven QTC price based on supply and demand
function calculateMarketPrice(totalSupply: number, exchangeFloat: number, btcUsd: number): number {
  // Market-driven pricing formula
  const basePrice = 0.01; // Starting price in USD
  const scarcityMultiplier = Math.max(1, (22000000 - totalSupply) / 22000000 * 10); // Scarcity increases price
  const demandMultiplier = Math.max(0.5, (250000 - exchangeFloat) / 250000 * 2); // Low supply = higher price
  const btcInfluence = Math.min(2, btcUsd / 50000); // BTC price influences QTC
  
  const marketPrice = basePrice * scarcityMultiplier * demandMultiplier * btcInfluence;
  return Math.max(0.001, marketPrice); // Minimum price floor
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

    // Get current supply and exchange data for market pricing
    const totalSupply = 1250000; // This would come from blockchain in production
    const exchangeFloat = Number(process.env.EXCHANGE_AVAILABLE_FLOAT || "250000");
    
    // Calculate market-driven QTC price
    const qtcMarketPrice = calculateMarketPrice(totalSupply, exchangeFloat, btcUsd);

    res.status(200).json({ 
      btcUsd: btcUsd,
      qtcUsd: qtcMarketPrice,
      totalSupply,
      exchangeFloat,
      scarcityLevel: (22000000 - totalSupply) / 22000000,
      demandLevel: (250000 - exchangeFloat) / 250000,
      timestamp: new Date().toISOString(),
      source: "market-driven",
      priceFormula: "basePrice * scarcity * demand * btcInfluence"
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
