import express from "express";
import fetch from "node-fetch";

const app = express();
app.use(express.json());
const PORT = process.env.PORT || 8080;
const NODE_RPC = process.env.NODE_RPC_URL || "http://node:8545";

async function rpc(method, params=[]){
  const r = await fetch(NODE_RPC, {method:"POST", headers:{'content-type':'application/json'},
    body: JSON.stringify({jsonrpc:"2.0", id:1, method, params})});
  if(!r.ok) throw new Error(`${r.status}`);
  const j = await r.json(); if(j.error) throw new Error(j.error.message);
  return j.result;
}

app.get("/status", async (_req,res)=>{
  try{
    const height = parseInt((await rpc("qc_blockNumber")).toString(),16);
    const peers  = parseInt((await rpc("qc_peerCount")).toString(),16);
    res.json({ height, peers });
  }catch(e){ res.status(503).json({ ok:false, error:String(e) }); }
});

app.get("/blocks", async (req,res)=>{
  const limit = Math.min(parseInt(req.query.limit||"10",10), 50);
  try{
    const head = parseInt((await rpc("qc_blockNumber")).toString(),16);
    const out=[];
    for(let n=head; n>Math.max(head-limit,0); n--){
      out.push(await rpc("qc_getBlockByNumber",[`0x${n.toString(16)}`, false]));
    }
    res.json(out);
  }catch(e){ res.status(503).json({ error:String(e) }); }
});

app.get("/tx/:hash", async (req,res)=>{
  try{ res.json({ hash:req.params.hash, note:"tx lookup not implemented yet" }) }
  catch(e){ res.status(404).json({ error:String(e) }); }
});

app.listen(PORT, ()=> console.log(`explorer-api on :${PORT} → ${NODE_RPC}`));
