import React, {createContext, useContext, useEffect, useState} from "react";
type RevCtx = { active: boolean; enable: () => void; disable: () => void; };
const Ctx = createContext<RevCtx | undefined>(undefined);

export const RevStopProvider: React.FC<{children: React.ReactNode}> = ({children}) => {
  const defaultOn = String(process.env.NEXT_PUBLIC_REVSTOP_DEFAULT_ON ?? process.env.REVSTOP_DEFAULT_ON ?? "true")==="true";
  const [active, setActive] = useState<boolean>(defaultOn);
  useEffect(()=>{ const v=localStorage.getItem("qc_revstop"); if(v!==null) setActive(v==="1"); else localStorage.setItem("qc_revstop", defaultOn?"1":"0");},[]);
  const enable = ()=>{localStorage.setItem("qc_revstop","1"); setActive(true);};
  const disable= ()=>{localStorage.setItem("qc_revstop","0"); setActive(false);};
  return <Ctx.Provider value={{active, enable, disable}}>{children}</Ctx.Provider>;
};
export const useRevStop=()=>{const v=useContext(Ctx); if(!v) throw new Error("useRevStop outside provider"); return v;};
