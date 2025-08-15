import type { NextApiRequest, NextApiResponse } from "next";

export default async function handler(_req:NextApiRequest,res:NextApiResponse){
  try{
    const r = await fetch("https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd",{
      headers:{accept:"application/json"}
    });
    const j = await r.json();
    res.status(200).json({ 
      usd: j?.bitcoin?.usd ?? null,
      timestamp: new Date().toISOString()
    });
  }catch{
    res.status(200).json({ usd: null, error: "Price fetch failed" });
  }
}
