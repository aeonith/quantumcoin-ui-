import fetch from "node-fetch";
const base = "http://localhost:8080";
(async ()=>{
  const s = await fetch(base+"/status").then(r=>r.json());
  if(!(s && typeof s.height === "number")) { console.error("status failed"); process.exit(1); }
  const b = await fetch(base+"/blocks?limit=3").then(r=>r.json());
  if(!Array.isArray(b)) { console.error("blocks failed"); process.exit(1); }
  console.log("explorer-api smoke ok");
})();
