import type { NextApiRequest, NextApiResponse } from "next";

// Calculate QTC amount based on BTC value and MARKET-DRIVEN QTC price
function computeQtcAmount(btcAmount: number, btcUsdPrice: number, qtcMarketPrice: number): number {
  const usdValue = btcAmount * btcUsdPrice;
  return Math.max(0, Math.floor(usdValue / qtcMarketPrice)); // Return integer QTC based on market price
}

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  const { txid } = req.query as { txid?: string };
  const receiveAddress = process.env.NEXT_PUBLIC_BTC_ADDRESS || "";

  if (!txid) {
    return res.status(400).json({ 
      ok: false, 
      error: "Missing transaction ID" 
    });
  }

  if (!receiveAddress) {
    return res.status(500).json({ 
      ok: false, 
      error: "Server configuration error: BTC address not configured" 
    });
  }

  try {
    // Fetch transaction data from mempool.space
    const txResponse = await fetch(`https://mempool.space/api/tx/${txid}`, {
      headers: {
        "Accept": "application/json",
        "User-Agent": "QuantumCoin-Exchange/1.0"
      },
      timeout: 15000
    });

    if (!txResponse.ok) {
      if (txResponse.status === 404) {
        return res.status(200).json({ 
          ok: false, 
          error: "Transaction not found. Make sure the txid is correct and the transaction is broadcast." 
        });
      }
      throw new Error(`Mempool API error: ${txResponse.status}`);
    }

    const txData = await txResponse.json();

    // Calculate total satoshis sent to our address
    let totalSatoshis = 0;
    for (const output of txData.vout || []) {
      const outputAddress = output?.scriptpubkey_address;
      if (outputAddress === receiveAddress) {
        totalSatoshis += Math.round(output?.value ?? 0);
      }
    }

    const confirmations = txData.status?.confirmed ? 1 : 0;
    const btcAmount = totalSatoshis / 100000000; // Convert satoshis to BTC

    if (totalSatoshis <= 0) {
      return res.status(200).json({ 
        ok: false, 
        error: `No Bitcoin found sent to our address ${receiveAddress}. Please check the transaction.` 
      });
    }

    // Get current market-driven prices
    const priceResponse = await fetch(`${req.headers['x-forwarded-proto'] ?? 'https'}://${req.headers.host}/api/btc-price`);
    const priceData = await priceResponse.json();
    const btcUsdPrice = priceData.btcUsd;
    const qtcMarketPrice = priceData.qtcUsd;

    if (!btcUsdPrice || !qtcMarketPrice) {
      return res.status(200).json({ 
        ok: false, 
        error: "Unable to fetch market prices. Please try again later." 
      });
    }

    // Calculate QTC amount using MARKET PRICE
    const estimatedQtc = computeQtcAmount(btcAmount, btcUsdPrice, qtcMarketPrice);

    return res.status(200).json({ 
      ok: true, 
      btc: btcAmount,
      satoshis: totalSatoshis,
      confirmations,
      btcUsdPrice,
      qtcMarketPrice,
      estimatedQtc,
      usdValue: btcAmount * btcUsdPrice,
      scarcityLevel: priceData.scarcityLevel,
      demandLevel: priceData.demandLevel,
      priceSource: priceData.source,
      timestamp: new Date().toISOString()
    });

  } catch (error: any) {
    console.error("BTC verification error:", error);
    return res.status(200).json({ 
      ok: false, 
      error: "Transaction verification failed. Please check your txid and try again." 
    });
  }
}
