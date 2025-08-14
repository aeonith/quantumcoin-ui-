import type { NextApiRequest, NextApiResponse } from "next";

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  if (req.method !== "POST") {
    return res.status(405).json({ 
      ok: false, 
      error: "Method not allowed. Use POST." 
    });
  }

  const { qtcAddr, amountQtc } = req.body || {};
  const availableFloat = Number(process.env.EXCHANGE_AVAILABLE_FLOAT || "250000");

  // Validation
  if (!qtcAddr || !amountQtc || amountQtc <= 0) {
    return res.status(400).json({ 
      ok: false, 
      error: "Invalid request. Missing QTC address or amount." 
    });
  }

  if (availableFloat <= 0) {
    return res.status(200).json({ 
      ok: false, 
      error: "No QTC available for exchange. Supply must be mined first." 
    });
  }

  if (amountQtc > availableFloat) {
    return res.status(200).json({ 
      ok: false, 
      error: `Insufficient exchange supply. Available: ${availableFloat} QTC, Requested: ${amountQtc} QTC` 
    });
  }

  // Check if we have a backend API configured
  const backendApiBase = process.env.NEXT_PUBLIC_API_BASE || "";

  if (!backendApiBase) {
    // No backend configured - simulate success for UI testing
    console.log(`[SIMULATED] Would credit ${amountQtc} QTC to ${qtcAddr}`);
    
    return res.status(200).json({ 
      ok: true, 
      credited: amountQtc, 
      txid: null,
      simulated: true,
      message: "Credit simulated - backend not configured yet"
    });
  }

  try {
    // Call the actual Rust backend to credit QTC
    const backendResponse = await fetch(`${backendApiBase.replace(/\/$/, "")}/credit`, {
      method: "POST",
      headers: { 
        "Content-Type": "application/json",
        "User-Agent": "QuantumCoin-Exchange/1.0"
      },
      body: JSON.stringify({ 
        address: qtcAddr, 
        amount: amountQtc 
      }),
      timeout: 30000
    });

    if (!backendResponse.ok) {
      throw new Error(`Backend API error: ${backendResponse.status}`);
    }

    const backendResult = await backendResponse.json();
    
    if (!backendResult || backendResult.error) {
      throw new Error(backendResult?.error || "Backend credit failed");
    }

    return res.status(200).json({ 
      ok: true, 
      credited: backendResult.amount || amountQtc, 
      txid: backendResult.txid || null,
      blockHeight: backendResult.blockHeight,
      timestamp: new Date().toISOString()
    });

  } catch (error: any) {
    console.error("Credit QTC error:", error);
    
    return res.status(200).json({ 
      ok: false, 
      error: "Failed to credit QTC. Backend may be temporarily unavailable." 
    });
  }
}
