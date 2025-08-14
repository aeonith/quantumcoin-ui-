import type { NextApiRequest, NextApiResponse } from "next";

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
        },
        timeout: 10000
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

    res.status(200).json({ 
      usd: btcUsd,
      timestamp: new Date().toISOString(),
      source: "CoinGecko"
    });

  } catch (error: any) {
    console.error("BTC price fetch error:", error);
    
    // Fallback to a reasonable default price if API fails
    const fallbackPrice = 95000; // Approximate BTC price
    
    res.status(200).json({ 
      usd: fallbackPrice,
      timestamp: new Date().toISOString(),
      source: "fallback",
      error: error.message
    });
  }
}
