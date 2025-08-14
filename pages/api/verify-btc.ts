import type { NextApiRequest, NextApiResponse } from "next";

// Calculate QTC amount based on BTC value and QTC price
function computeQtcAmount(btcAmount: number, btcUsdPrice: number, qtcUsdPrice?: number): number {
  const qtcPrice = Number(process.env.QTC_USD_PRICE || qtcUsdPrice || 1.00);
  const usdValue = btcAmount * btcUsdPrice;
  return Math.max(0, Math.floor(usdValue / qtcPrice)); // Return integer QTC
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

    // Get current BTC price
    const priceResponse = await fetch(`${req.headers['x-forwarded-proto'] ?? 'https'}://${req.headers.host}/api/btc-price`);
    const priceData = await priceResponse.json();
    const btcUsdPrice = priceData.usd;

    if (!btcUsdPrice) {
      return res.status(200).json({ 
        ok: false, 
        error: "Unable to fetch BTC price. Please try again later." 
      });
    }

    // Calculate QTC amount
    const estimatedQtc = computeQtcAmount(btcAmount, btcUsdPrice);

    return res.status(200).json({ 
      ok: true, 
      btc: btcAmount,
      satoshis: totalSatoshis,
      confirmations,
      btcUsdPrice,
      estimatedQtc,
      usdValue: btcAmount * btcUsdPrice,
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
