import type { NextApiRequest, NextApiResponse } from 'next';

interface PriceResponse {
  usd: number | null;
  timestamp: string;
  qtcUsd?: number;
  error?: string;
}

interface CoinGeckoResponse {
  bitcoin?: {
    usd?: number;
  };
}

export default async function handler(
  _req: NextApiRequest,
  res: NextApiResponse<PriceResponse>
): Promise<void> {
  try {
    const response = await fetch(
      'https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd',
      {
        headers: {
          accept: 'application/json',
        },
      }
    );
    
    const data: CoinGeckoResponse = await response.json();
    const btcPrice = data?.bitcoin?.usd ?? null;
    
    // Calculate QTC price (QTC is pegged at 1/2400 of BTC for now)
    const qtcPrice = btcPrice ? btcPrice / 2400 : 0.025;
    
    res.status(200).json({
      usd: btcPrice,
      qtcUsd: qtcPrice,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    console.error('BTC price fetch error:', error);
    res.status(500).json({
      usd: null,
      qtcUsd: 0.025,
      error: 'Price fetch failed',
      timestamp: new Date().toISOString(),
    });
  }
}
