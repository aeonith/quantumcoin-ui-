import Link from "next/link";
import { useState } from "react";
import { useAuth } from "@/context/AuthContext";

export default function NavBar(){
  const { user, logout } = useAuth();
  const [open, setOpen] = useState(false);
  const Item = ({href, children}:{href:string; children:any}) => <Link href={href} className="px-3 py-2 hover:opacity-80">{children}</Link>;

  return (
    <header className="w-full border-b border-cyan-700/30 bg-[#07121a] text-cyan-200">
      <div className="mx-auto max-w-6xl px-4 h-14 flex items-center justify-between">
        <Link href="/" className="font-semibold text-cyan-300">⚛️ QuantumCoin</Link>
        <nav className="hidden md:flex items-center gap-1">
          <Item href="/wallet">Wallet</Item>
          <Item href="/explorer">Explorer</Item>
          <Item href="/dashboard">Dashboard</Item>
          <Item href="/mining">Mining</Item>
          <Item href="/kyc">KYC</Item>
          <Item href="/exchange">Exchange</Item>
          {!user ? (
            <>
              <Item href="/login">Login</Item>
              <Link href="/register" className="ml-2 px-4 py-2 rounded bg-cyan-500 text-black">Create account</Link>
            </>
          ) : (
            <>
              <span className="px-3 text-sm opacity-75">Hi, {user.email}</span>
              <button onClick={logout} className="px-3 py-2 rounded bg-red-600 text-white">Logout</button>
            </>
          )}
        </nav>
        <button className="md:hidden p-2" onClick={()=>setOpen(!open)}>☰</button>
      </div>
      {open && (
        <div className="md:hidden px-4 pb-3 flex flex-col gap-2 bg-[#081821]">
          {["/wallet","/explorer","/dashboard","/mining","/kyc","/exchange"].map(h=> <Item key={h} href={h}>{h.replace("/","").toUpperCase()}</Item>)}
          {!user ? (<><Item href="/login">LOGIN</Item><Item href="/register">CREATE ACCOUNT</Item></>) :
            (<button onClick={logout} className="self-start px-3 py-2 rounded bg-red-600 text-white">Logout</button>)}
        </div>
      )}
    </header>
  );
}
