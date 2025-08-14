import NavBar from "@/components/NavBar";
import HeroTiles from "@/components/HeroTiles";
import Link from "next/link";

export default function Home() {
  return (
    <div className="min-h-screen bg-quantum-dark text-cyan-100">
      <NavBar />
      
      {/* Hero Section */}
      <main className="pt-20 pb-16">
        <section className="mx-auto max-w-6xl px-4 py-12 text-center">
          <div className="mb-8">
            <h1 className="text-4xl md:text-6xl font-bold text-cyan-300 mb-4">
              🚀 QuantumCoin™
            </h1>
            <p className="text-xl md:text-2xl opacity-90 mb-6">
              The Future of Quantum-Resistant Digital Currency
            </p>
            <p className="text-lg opacity-75 max-w-3xl mx-auto leading-relaxed">
              Built with post-quantum cryptography (Dilithium2) to resist attacks from both 
              classical and quantum computers. Featuring RevStop™ protection and a hard-capped 
              supply of 22 million coins.
            </p>
          </div>

          {/* Quick Stats */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12 max-w-4xl mx-auto">
            <div className="quantum-card text-center">
              <div className="text-2xl font-bold text-cyan-300">22M</div>
              <div className="text-sm opacity-75">Max Supply</div>
            </div>
            <div className="quantum-card text-center">
              <div className="text-2xl font-bold text-cyan-300">Dilithium2</div>
              <div className="text-sm opacity-75">Post-Quantum Crypto</div>
            </div>
            <div className="quantum-card text-center">
              <div className="text-2xl font-bold text-cyan-300">RevStop™</div>
              <div className="text-sm opacity-75">Transaction Protection</div>
            </div>
          </div>

          {/* Call to Action */}
          <div className="flex flex-col sm:flex-row gap-4 justify-center mb-12">
            <Link href="/wallet" className="quantum-button-primary text-center py-3 px-6">
              🔑 Generate Wallet
            </Link>
            <Link href="/exchange" className="quantum-button-secondary text-center py-3 px-6">
              💱 Buy with BTC
            </Link>
            <a 
              href="https://github.com/aeonith/quantumcoin-ui-" 
              target="_blank" 
              rel="noopener noreferrer"
              className="quantum-button-secondary text-center py-3 px-6"
            >
              📂 View Source
            </a>
          </div>
        </section>

        {/* Feature Tiles */}
        <HeroTiles />

        {/* Technology Section */}
        <section className="mx-auto max-w-4xl px-4 py-12">
          <div className="quantum-card">
            <h2 className="text-2xl font-bold text-cyan-300 mb-6 text-center">
              ⚙️ Why QuantumCoin™?
            </h2>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
              <div>
                <h3 className="text-lg font-semibold text-cyan-300 mb-3">🛡️ Post-Quantum Security</h3>
                <p className="text-sm opacity-80 leading-relaxed">
                  Uses NIST-approved Dilithium2 signature schemes to resist attacks from 
                  quantum computers that threaten Bitcoin and other traditional cryptocurrencies.
                </p>
              </div>
              
              <div>
                <h3 className="text-lg font-semibold text-cyan-300 mb-3">🔒 RevStop™ Protection</h3>
                <p className="text-sm opacity-80 leading-relaxed">
                  Optional irreversible wallet lock feature that allows users to halt all 
                  movement of coins permanently - an extra defense for long-term holders.
                </p>
              </div>
              
              <div>
                <h3 className="text-lg font-semibold text-cyan-300 mb-3">⚡ Hard-Capped Supply</h3>
                <p className="text-sm opacity-80 leading-relaxed">
                  Maximum supply of 22 million coins with mining rewards that halve every 2 years, 
                  ensuring controlled issuance over 66 years with no pre-mine.
                </p>
              </div>
              
              <div>
                <h3 className="text-lg font-semibold text-cyan-300 mb-3">🌐 Decentralized</h3>
                <p className="text-sm opacity-80 leading-relaxed">
                  Open-source, community-driven blockchain that anyone can mine, validate, 
                  or build upon without centralized control or intermediaries.
                </p>
              </div>
            </div>
          </div>
        </section>

        {/* Footer */}
        <footer className="text-center text-sm opacity-60 py-8">
          <p>&copy; 2025 QuantumCoin™. Built for the Next Era of Humanity.</p>
          <div className="mt-2 flex justify-center gap-4">
            <a href="https://t.me/+abzOvpCAUfwyYjIx" target="_blank" rel="noopener noreferrer" className="hover:text-cyan-300">
              💬 Telegram
            </a>
            <a href="https://github.com/aeonith/quantumcoin-ui-" target="_blank" rel="noopener noreferrer" className="hover:text-cyan-300">
              📂 GitHub
            </a>
          </div>
        </footer>
      </main>
    </div>
  );
}
