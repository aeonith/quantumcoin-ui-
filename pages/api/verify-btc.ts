import type { NextApiRequest, NextApiResponse } from "next";

// Compute how many QTC to credit: BTC value in USD divided by QTC price.
function computeQtc(btc:number, btcUsd:number, qtcUsd?:number){
  const priceQtc = Number(process.env.QTC_USD_PRICE || qtcUsd || 0.45);
  const usd = btc * btcUsd;
  return Math.max(0, Math.floor((usd / priceQtc) * 100000000) / 100000000); // 8 decimal places
}

export default async function handler(req:NextApiRequest,res:NextApiResponse){
  const { txid } = req.query as { txid?: string };
  const receive = process.env.NEXT_PUBLIC_BTC_ADDRESS || "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh";
  
  if(!txid) return res.status(400).json({ ok:false, error:"Missing txid" });

  try{
    // Verify transaction via mempool.space API
    const txr = await fetch(`https://mempool.space/api/tx/${txid}`);
    if(!txr.ok) return res.status(200).json({ ok:false, error:"Transaction not found on blockchain" });
    const tx = await txr.json();

    // Sum outputs to our address
    let sats = 0;
    for(const o of tx.vout || []){
      const addr = o?.scriptpubkey_address;
      if(addr === receive) sats += Math.round((o?.value ?? 0));
    }
    const confs = tx.status?.confirmed ? (tx.status.block_height ? 1 : 0) : 0;
    const btc = sats / 1e8;

    if(sats<=0) return res.status(200).json({ ok:false, error:`No BTC sent to our address ${receive}` });
    if(btc < 0.001) return res.status(200).json({ ok:false, error:"Minimum 0.001 BTC required" });

    // Get current BTC price
    const priceRes = await fetch(`https://${req.headers.host}/api/btc-price`);
    const { usd: btcUsd } = await priceRes.json();
    
    if (!btcUsd) return res.status(200).json({ ok:false, error:"Unable to get BTC price" });

    const estimatedQtc = computeQtc(btc, btcUsd, Number(process.env.QTC_USD_PRICE));
    
    return res.status(200).json({ 
      ok:true, 
      btc, 
      sats, 
      confs,
      btcUsd,
      estimatedQtc,
      qtcPrice: Number(process.env.QTC_USD_PRICE || 0.45)
    });
  }catch(e:any){
    return res.status(200).json({ ok:false, error:"Blockchain verification failed" });
  }
}
