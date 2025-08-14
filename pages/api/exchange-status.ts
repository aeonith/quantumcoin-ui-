import type { NextApiRequest, NextApiResponse } from "next";

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  try {
    const float = Number(process.env.EXCHANGE_AVAILABLE_FLOAT || "250000");
    const enabled = float > 0;

    res.status(200).json({ 
      float, 
      enabled,
      qtcUsdPrice: Number(process.env.QTC_USD_PRICE || "1.00")
    });
  } catch (error) {
    console.error("Exchange status error:", error);
    res.status(500).json({ 
      float: 0, 
      enabled: false, 
      error: "Internal server error" 
    });
  }
}
