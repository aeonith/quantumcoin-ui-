import NavBar from "@/components/NavBar";
import WalletCard from "@/components/WalletCard";
import { useRevStop } from "@/context/RevStopContext";

export default function Wallet(){
  const { active, enable, disable } = useRevStop();
  return (
    <main className="min-h-screen bg-[#061018] text-cyan-100">
      <NavBar/>
      <div className="mx-auto max-w-6xl px-4 py-8">
        <div className="mb-6 flex items-center gap-3">
          <span className="text-cyan-300 font-semibold">RevStop™ Protection:</span>
          <span className={`px-3 py-1 rounded ${active?"bg-green-700/50 text-green-200":"bg-red-800/50 text-red-200"}`}>
            {active ? "ACTIVE" : "DISABLED"}
          </span>
          {active ? (
            <button onClick={disable} className="px-3 py-1 rounded bg-red-600">Disable</button>
          ) : (
            <button onClick={enable} className="px-3 py-1 rounded bg-cyan-500 text-black">Enable</button>
          )}
        </div>
        <WalletCard/>
      </div>
    </main>
  );
}
