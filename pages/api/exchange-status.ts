import type { NextApiRequest, NextApiResponse } from "next";

export default async function handler(_req:NextApiRequest,res:NextApiResponse){
  const float = Number(process.env.EXCHANGE_AVAILABLE_FLOAT || "50000");
  res.status(200).json({ 
    float, 
    enabled: float > 0,
    priceUSD: Number(process.env.QTC_USD_PRICE || "0.45"),
    updated: new Date().toISOString()
  });
}
