import NavBar from "@/components/NavBar";
import HeroTiles from "@/components/HeroTiles";

export default function Home(){
  return (
    <main className="min-h-screen bg-[#061018] text-cyan-100">
      <NavBar/>
      <section className="mx-auto max-w-6xl px-4 py-10">
        <h1 className="text-3xl font-bold text-cyan-300 mb-2">QuantumCoin</h1>
        <p className="opacity-80 mb-8">Post-quantum cryptocurrency secured by Dilithium2.</p>
      </section>
      <HeroTiles/>
    </main>
  );
}
