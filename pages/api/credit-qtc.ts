import type { NextApiRequest, NextApiResponse } from "next";

export default async function handler(req:NextApiRequest,res:NextApiResponse){
  if(req.method!=="POST") return res.status(405).json({ok:false,error:"POST only"});
  
  const { qtcAddr, amountQtc } = req.body || {};
  const float = Number(process.env.EXCHANGE_AVAILABLE_FLOAT || "50000");
  
  if(!qtcAddr || !amountQtc || amountQtc<=0) {
    return res.status(400).json({ok:false,error:"Invalid request: missing address or amount"});
  }
  
  if(float <= 0) {
    return res.status(200).json({ok:false,error:"No exchange supply available. QTC must be mined first."});
  }

  if(amountQtc > float) {
    return res.status(200).json({ok:false,error:`Insufficient supply. Max available: ${float} QTC`});
  }

  const backendUrl = process.env.NEXT_PUBLIC_API_BASE || "http://localhost:8080";
  
  try{
    // Try to credit via Rust backend
    const response = await fetch(`${backendUrl}/wallet/credit`, {
      method:"POST",
      headers:{ "Content-Type":"application/json" },
      body: JSON.stringify({ 
        address: qtcAddr, 
        amount: amountQtc,
        reason: "BTC_EXCHANGE"
      })
    });
    
    if (response.ok) {
      const result = await response.json();
      return res.status(200).json({ 
        ok:true, 
        credited: result.amount || amountQtc, 
        txid: result.txid || null,
        newBalance: result.newBalance || null
      });
    } else {
      throw new Error("Backend credit failed");
    }
    
  } catch (error) {
    // Demo mode when backend not available
    console.log("Backend not available, using demo mode:", error);
    return res.status(200).json({ 
      ok:true, 
      credited: amountQtc, 
      txid: `demo_${Date.now()}`,
      simulated: true,
      message: "Demo mode - will be real when backend is live"
    });
  }
}
