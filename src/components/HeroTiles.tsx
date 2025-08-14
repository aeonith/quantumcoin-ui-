import Link from "next/link";
const Tile=({href,title,desc}:{href:string;title:string;desc:string})=>(
  <Link href={href} className="rounded-xl p-5 bg-gradient-to-br from-[#0a1f2b] to-[#103042] border border-cyan-700/30 hover:border-cyan-400/60 transition">
    <div className="text-lg font-semibold text-cyan-300">{title}</div>
    <div className="text-sm opacity-80 mt-1">{desc}</div>
  </Link>
);
export default function HeroTiles(){
  return (
    <section className="mx-auto max-w-6xl px-4">
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        <Tile href="/wallet" title="Wallet" desc="Generate, back up, and manage your QTC address."/>
        <Tile href="/explorer" title="Explorer" desc="Search blocks, transactions, and addresses."/>
        <Tile href="/dashboard" title="Dashboard" desc="Balances, RevStop, AI Security & analytics."/>
        <Tile href="/mining" title="Mining" desc="Start mining or join a pool (coming soon)."/>
        <Tile href="/kyc" title="KYC" desc="Optional verification for exchange usage."/>
        <Tile href="/exchange" title="Exchange" desc="Buy QTC with BTC when supply is available."/>
      </div>
    </section>
  );
}
