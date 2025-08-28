import { Metadata } from 'next'
import Link from 'next/link'

export const metadata: Metadata = {
  title: 'wQC - Wrapped QuantumCoin Token | Early Access to the Quantum Future',
  description: 'Get early access to QuantumCoin through wQC, our wrapped ERC-20 token. 1:1 redeemable for native QTC when mainnet launches.',
}

export default function WrappedTokenPage() {
  return (
    <main className="min-h-screen bg-gradient-to-br from-purple-950 via-indigo-950 to-slate-950 text-white">
      {/* Animated Background */}
      <div className="absolute inset-0 overflow-hidden">
        <div className="token-particles"></div>
        <div className="pulse-rings"></div>
      </div>

      {/* Hero Section */}
      <section className="relative z-10 py-24">
        <div className="mx-auto max-w-7xl px-6">
          <div className="text-center">
            <div className="inline-flex items-center gap-3 rounded-full border border-purple-400/30 bg-purple-400/10 px-6 py-3 text-sm font-medium text-purple-300 backdrop-blur-sm">
              <div className="h-3 w-3 rounded-full bg-emerald-400 animate-ping"></div>
              Live on Ethereum • ERC-20 • 1:1 Redeemable
            </div>
            
            <h1 className="mt-8 text-6xl md:text-8xl font-black">
              <span className="bg-gradient-to-r from-purple-400 via-pink-400 to-cyan-400 bg-clip-text text-transparent animate-gradient">
                wQC Token
              </span>
            </h1>
            
            <div className="mt-4 text-2xl md:text-4xl font-light text-slate-300">
              Your Gateway to the 
              <span className="font-bold text-purple-400"> Quantum-Safe Future</span>
            </div>
            
            <p className="mt-6 max-w-3xl mx-auto text-xl text-slate-400">
              wQC (Wrapped QuantumCoin) gives you early access to the world's first post-quantum cryptocurrency. 
              Each token is redeemable 1:1 for native QTC when the QuantumCoin mainnet launches.
            </p>
          </div>
        </div>
      </section>

      {/* Live Price Section */}
      <section className="relative z-10 py-16">
        <div className="mx-auto max-w-5xl px-6">
          <div className="rounded-3xl border border-purple-500/30 bg-gradient-to-r from-purple-900/40 to-pink-900/40 p-8 backdrop-blur-sm">
            <div className="grid lg:grid-cols-3 gap-8 items-center">
              <div className="lg:col-span-2">
                <h2 className="text-3xl font-bold mb-4 text-purple-300">Live Token Metrics</h2>
                
                <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                  <div className="text-center p-4 rounded-xl bg-slate-800/50">
                    <div className="text-3xl font-black text-emerald-400" id="wqc-price">
                      $0.00
                    </div>
                    <div className="text-sm text-slate-400">Current Price</div>
                  </div>
                  
                  <div className="text-center p-4 rounded-xl bg-slate-800/50">
                    <div className="text-3xl font-black text-cyan-400">
                      100,000
                    </div>
                    <div className="text-sm text-slate-400">Max Supply</div>
                  </div>
                  
                  <div className="text-center p-4 rounded-xl bg-slate-800/50">
                    <div className="text-3xl font-black text-purple-400" id="market-cap">
                      $0
                    </div>
                    <div className="text-sm text-slate-400">Market Cap</div>
                  </div>
                  
                  <div className="text-center p-4 rounded-xl bg-slate-800/50">
                    <div className="text-3xl font-black text-pink-400" id="volume-24h">
                      $0
                    </div>
                    <div className="text-sm text-slate-400">24h Volume</div>
                  </div>
                </div>
                
                <div className="mt-6 text-center">
                  <div className="text-sm text-slate-500">
                    Contract: <span className="font-mono text-purple-300" id="contract-address">Deploying Soon...</span>
                  </div>
                </div>
              </div>
              
              <div className="text-center">
                <div className="relative inline-block">
                  <div className="h-32 w-32 rounded-full bg-gradient-to-r from-purple-500 to-pink-500 animate-pulse mx-auto mb-4"></div>
                  <div className="absolute inset-0 flex items-center justify-center">
                    <span className="text-4xl font-black text-white">wQC</span>
                  </div>
                </div>
                
                <button 
                  className="mt-4 group relative overflow-hidden rounded-xl bg-gradient-to-r from-purple-600 to-pink-600 px-8 py-4 font-bold text-white transition-all duration-300 hover:scale-105 hover:shadow-2xl hover:shadow-purple-500/25"
                  onClick={() => window.open('https://app.uniswap.org', '_blank')}
                >
                  <span className="relative z-10">🦄 Trade on Uniswap</span>
                  <div className="absolute inset-0 bg-gradient-to-r from-pink-600 to-purple-600 opacity-0 transition-opacity duration-300 group-hover:opacity-100"></div>
                </button>
                
                <div className="mt-2 text-xs text-slate-500">
                  (Available after liquidity launch)
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* How wQC Works */}
      <section className="relative z-10 py-24">
        <div className="mx-auto max-w-7xl px-6">
          <div className="text-center mb-16">
            <h2 className="text-4xl md:text-6xl font-black mb-6">
              How <span className="bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">wQC Works</span>
            </h2>
            <p className="text-xl text-slate-300">
              The bridge between Ethereum liquidity and QuantumCoin's quantum-safe future
            </p>
          </div>
          
          <div className="grid lg:grid-cols-4 gap-8">
            {[
              {
                step: "1",
                title: "Buy wQC",
                desc: "Purchase wQC tokens on Uniswap using ETH or USDC",
                icon: "💎",
                color: "purple"
              },
              {
                step: "2", 
                title: "Hold & Trade",
                desc: "Trade freely on DEXs while waiting for mainnet launch",
                icon: "🔄",
                color: "pink"
              },
              {
                step: "3",
                title: "Mainnet Launch",
                desc: "QuantumCoin L1 launches with genesis block mining",
                icon: "🚀",
                color: "cyan"
              },
              {
                step: "4",
                title: "Redeem 1:1",
                desc: "Exchange your wQC for native QTC on the new blockchain",
                icon: "⚛️",
                color: "emerald"
              }
            ].map((step, index) => (
              <div 
                key={step.step}
                className="group relative rounded-2xl border border-slate-700/50 bg-slate-800/30 p-8 backdrop-blur-sm transition-all duration-500 hover:scale-105 animate-fade-in-up"
                style={{ animationDelay: `${index * 150}ms` }}
              >
                <div className="text-center">
                  <div className={`inline-flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-r mb-6 ${
                    step.color === 'purple' ? 'from-purple-500 to-purple-600' :
                    step.color === 'pink' ? 'from-pink-500 to-pink-600' :
                    step.color === 'cyan' ? 'from-cyan-500 to-cyan-600' :
                    'from-emerald-500 to-emerald-600'
                  }`}>
                    <span className="text-2xl">{step.icon}</span>
                  </div>
                  
                  <div className={`text-sm font-bold mb-2 ${
                    step.color === 'purple' ? 'text-purple-400' :
                    step.color === 'pink' ? 'text-pink-400' :
                    step.color === 'cyan' ? 'text-cyan-400' :
                    'text-emerald-400'
                  }`}>
                    Step {step.step}
                  </div>
                  
                  <h3 className="text-xl font-bold text-white mb-3">{step.title}</h3>
                  <p className="text-slate-400 text-sm leading-relaxed">{step.desc}</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Token Economics */}
      <section className="relative z-10 py-24 bg-slate-900/50">
        <div className="mx-auto max-w-7xl px-6">
          <div className="grid lg:grid-cols-2 gap-16 items-center">
            <div>
              <h2 className="text-4xl md:text-5xl font-black mb-6">
                <span className="bg-gradient-to-r from-emerald-400 to-cyan-400 bg-clip-text text-transparent">
                  Token Economics
                </span>
              </h2>
              
              <div className="space-y-6">
                <div className="p-6 rounded-xl bg-emerald-900/20 border border-emerald-500/30">
                  <h3 className="text-xl font-bold text-emerald-300 mb-3">Fixed Supply Model</h3>
                  <ul className="space-y-2 text-slate-300">
                    <li className="flex items-center gap-2">
                      <span className="text-emerald-400">✓</span>
                      <strong>100,000 wQC</strong> maximum supply - no additional minting
                    </li>
                    <li className="flex items-center gap-2">
                      <span className="text-emerald-400">✓</span>
                      <strong>1:1 redemption</strong> for native QTC tokens
                    </li>
                    <li className="flex items-center gap-2">
                      <span className="text-emerald-400">✓</span>
                      <strong>No dilution</strong> - supply is permanently fixed
                    </li>
                  </ul>
                </div>
                
                <div className="p-6 rounded-xl bg-cyan-900/20 border border-cyan-500/30">
                  <h3 className="text-xl font-bold text-cyan-300 mb-3">Redemption Mechanism</h3>
                  <ul className="space-y-2 text-slate-300">
                    <li className="flex items-center gap-2">
                      <span className="text-cyan-400">🔄</span>
                      Backed by <strong>mined QTC</strong> (no premine)
                    </li>
                    <li className="flex items-center gap-2">
                      <span className="text-cyan-400">⏰</span>
                      <strong>6-12 month</strong> redemption window
                    </li>
                    <li className="flex items-center gap-2">
                      <span className="text-cyan-400">🔒</span>
                      Smart contract enforced redemption
                    </li>
                  </ul>
                </div>
              </div>
            </div>
            
            <div className="relative">
              {/* Token Value Visualization */}
              <div className="relative mx-auto h-96 w-96">
                <div className="absolute inset-0 rounded-full bg-gradient-to-r from-purple-500/20 to-cyan-500/20 animate-spin-slow"></div>
                <div className="absolute inset-8 rounded-full bg-gradient-to-r from-pink-500/30 to-purple-500/30 animate-spin-reverse"></div>
                <div className="absolute inset-16 rounded-full bg-slate-900/90 flex items-center justify-center border border-purple-500/30">
                  <div className="text-center">
                    <div className="text-8xl mb-6 animate-pulse">💎</div>
                    <div className="text-3xl font-black text-purple-400 mb-2">wQC</div>
                    <div className="text-lg text-slate-400 mb-4">Wrapped Token</div>
                    <div className="text-sm text-emerald-400 font-semibold">
                      1 wQC = 1 QTC
                    </div>
                  </div>
                </div>
                
                {/* Floating Elements */}
                <div className="absolute top-1/4 -left-8 text-4xl animate-bounce animation-delay-200">⚛️</div>
                <div className="absolute top-3/4 -right-8 text-4xl animate-bounce animation-delay-400">🔮</div>
                <div className="absolute -top-8 right-1/4 text-4xl animate-bounce animation-delay-600">✨</div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Where to Buy */}
      <section className="relative z-10 py-24 bg-gradient-to-r from-slate-900/80 to-purple-900/80">
        <div className="mx-auto max-w-7xl px-6">
          <div className="text-center mb-16">
            <h2 className="text-4xl md:text-6xl font-black mb-6">
              <span className="bg-gradient-to-r from-pink-400 to-purple-400 bg-clip-text text-transparent">
                Where to Buy wQC
              </span>
            </h2>
            <p className="text-xl text-slate-300">
              Multiple ways to get your hands on the future of money
            </p>
          </div>
          
          <div className="grid md:grid-cols-3 gap-8">
            {/* Uniswap */}
            <div className="group relative rounded-2xl border border-pink-500/30 bg-pink-900/20 p-8 backdrop-blur-sm transition-all duration-500 hover:scale-105 hover:border-pink-400/50">
              <div className="text-center">
                <div className="inline-flex h-20 w-20 items-center justify-center rounded-2xl bg-gradient-to-r from-pink-500 to-purple-600 mb-6">
                  <span className="text-3xl">🦄</span>
                </div>
                
                <h3 className="text-2xl font-bold text-pink-400 mb-4">Uniswap V3</h3>
                <p className="text-slate-300 mb-6">
                  Trade wQC directly on the largest decentralized exchange with deep liquidity and optimal pricing.
                </p>
                
                <div className="space-y-3 mb-6">
                  <div className="flex justify-between items-center p-3 rounded-lg bg-slate-800/50">
                    <span className="text-slate-400">Pair</span>
                    <span className="font-semibold text-pink-300">wQC/USDC</span>
                  </div>
                  <div className="flex justify-between items-center p-3 rounded-lg bg-slate-800/50">
                    <span className="text-slate-400">Liquidity</span>
                    <span className="font-semibold text-emerald-400" id="uniswap-liquidity">$0</span>
                  </div>
                  <div className="flex justify-between items-center p-3 rounded-lg bg-slate-800/50">
                    <span className="text-slate-400">Fee Tier</span>
                    <span className="font-semibold text-cyan-400">0.3%</span>
                  </div>
                </div>
                
                <button 
                  className="w-full rounded-xl bg-gradient-to-r from-pink-600 to-purple-600 py-4 font-bold text-white transition-all duration-300 hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed"
                  disabled
                >
                  Trade on Uniswap (Soon)
                </button>
              </div>
            </div>

            {/* 1inch */}
            <div className="group relative rounded-2xl border border-cyan-500/30 bg-cyan-900/20 p-8 backdrop-blur-sm transition-all duration-500 hover:scale-105 hover:border-cyan-400/50">
              <div className="text-center">
                <div className="inline-flex h-20 w-20 items-center justify-center rounded-2xl bg-gradient-to-r from-cyan-500 to-blue-600 mb-6">
                  <span className="text-3xl">🔄</span>
                </div>
                
                <h3 className="text-2xl font-bold text-cyan-400 mb-4">1inch Exchange</h3>
                <p className="text-slate-300 mb-6">
                  Get the best prices across all DEXs with 1inch's intelligent routing and optimal swap execution.
                </p>
                
                <div className="space-y-3 mb-6">
                  <div className="flex justify-between items-center p-3 rounded-lg bg-slate-800/50">
                    <span className="text-slate-400">Best Price</span>
                    <span className="font-semibold text-emerald-400">✓ Guaranteed</span>
                  </div>
                  <div className="flex justify-between items-center p-3 rounded-lg bg-slate-800/50">
                    <span className="text-slate-400">Slippage</span>
                    <span className="font-semibold text-cyan-400">Minimized</span>
                  </div>
                  <div className="flex justify-between items-center p-3 rounded-lg bg-slate-800/50">
                    <span className="text-slate-400">Gas</span>
                    <span className="font-semibold text-blue-400">Optimized</span>
                  </div>
                </div>
                
                <button 
                  className="w-full rounded-xl bg-gradient-to-r from-cyan-600 to-blue-600 py-4 font-bold text-white transition-all duration-300 hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed"
                  disabled
                >
                  Swap on 1inch (Soon)
                </button>
              </div>
            </div>

            {/* Direct Purchase */}
            <div className="group relative rounded-2xl border border-emerald-500/30 bg-emerald-900/20 p-8 backdrop-blur-sm transition-all duration-500 hover:scale-105 hover:border-emerald-400/50">
              <div className="text-center">
                <div className="inline-flex h-20 w-20 items-center justify-center rounded-2xl bg-gradient-to-r from-emerald-500 to-teal-600 mb-6">
                  <span className="text-3xl">🏦</span>
                </div>
                
                <h3 className="text-2xl font-bold text-emerald-400 mb-4">Direct Purchase</h3>
                <p className="text-slate-300 mb-6">
                  Buy wQC directly from our smart contract with credit card or bank transfer integration.
                </p>
                
                <div className="space-y-3 mb-6">
                  <div className="flex justify-between items-center p-3 rounded-lg bg-slate-800/50">
                    <span className="text-slate-400">Payment</span>
                    <span className="font-semibold text-emerald-400">Card/Bank</span>
                  </div>
                  <div className="flex justify-between items-center p-3 rounded-lg bg-slate-800/50">
                    <span className="text-slate-400">KYC</span>
                    <span className="font-semibold text-emerald-400">Required</span>
                  </div>
                  <div className="flex justify-between items-center p-3 rounded-lg bg-slate-800/50">
                    <span className="text-slate-400">Fees</span>
                    <span className="font-semibold text-emerald-400">2.5%</span>
                  </div>
                </div>
                
                <button 
                  className="w-full rounded-xl bg-gradient-to-r from-emerald-600 to-teal-600 py-4 font-bold text-white transition-all duration-300 hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed"
                  disabled
                >
                  Buy Direct (Soon)
                </button>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Investment Thesis */}
      <section className="relative z-10 py-24">
        <div className="mx-auto max-w-7xl px-6">
          <div className="text-center mb-16">
            <h2 className="text-4xl md:text-6xl font-black mb-6">
              The <span className="bg-gradient-to-r from-yellow-400 to-orange-400 bg-clip-text text-transparent">Investment Thesis</span>
            </h2>
            <p className="text-xl text-slate-300">
              Why smart money is positioning for the quantum-safe future
            </p>
          </div>
          
          <div className="grid lg:grid-cols-2 gap-12">
            <div className="space-y-8">
              <div className="p-8 rounded-2xl border border-yellow-500/30 bg-yellow-900/20">
                <h3 className="text-2xl font-bold text-yellow-400 mb-4">🎯 First-Mover Advantage</h3>
                <p className="text-slate-300 leading-relaxed">
                  When quantum computers threaten existing cryptocurrencies, QuantumCoin will be the only major 
                  quantum-safe option. Early adopters position themselves for massive opportunity.
                </p>
              </div>
              
              <div className="p-8 rounded-2xl border border-orange-500/30 bg-orange-900/20">
                <h3 className="text-2xl font-bold text-orange-400 mb-4">📈 Scarcity Value</h3>
                <p className="text-slate-300 leading-relaxed">
                  Only 100,000 wQC tokens exist. With no additional minting possible, 
                  increasing demand drives up value. Limited supply meets unlimited potential.
                </p>
              </div>
              
              <div className="p-8 rounded-2xl border border-red-500/30 bg-red-900/20">
                <h3 className="text-2xl font-bold text-red-400 mb-4">⚠️ Quantum Timeline</h3>
                <p className="text-slate-300 leading-relaxed">
                  IBM and Google predict cryptographically relevant quantum computers within 10-15 years. 
                  When that happens, $2+ trillion in crypto assets become vulnerable overnight.
                </p>
              </div>
            </div>
            
            <div className="space-y-8">
              <div className="p-8 rounded-2xl border border-emerald-500/30 bg-emerald-900/20">
                <h3 className="text-2xl font-bold text-emerald-400 mb-4">🔒 Safety Net</h3>
                <p className="text-slate-300 leading-relaxed">
                  While other crypto holders face quantum uncertainty, QuantumCoin provides a quantum-safe 
                  store of value that remains secure regardless of quantum computer advancement.
                </p>
              </div>
              
              <div className="p-8 rounded-2xl border border-blue-500/30 bg-blue-900/20">
                <h3 className="text-2xl font-bold text-blue-400 mb-4">🚀 Technology Leadership</h3>
                <p className="text-slate-300 leading-relaxed">
                  QuantumCoin isn't just quantum-safe - it features AI-powered RevStop protection, 
                  advanced fee estimation, and cutting-edge blockchain technology.
                </p>
              </div>
              
              <div className="p-8 rounded-2xl border border-purple-500/30 bg-purple-900/20">
                <h3 className="text-2xl font-bold text-purple-400 mb-4">💡 Smart Strategy</h3>
                <p className="text-slate-300 leading-relaxed">
                  Diversify your crypto portfolio with the only major quantum-resistant option. 
                  Hedge against quantum risk while participating in cryptocurrency's future.
                </p>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Risk Disclosure */}
      <section className="relative z-10 py-16 bg-slate-800/30">
        <div className="mx-auto max-w-5xl px-6">
          <div className="p-8 rounded-2xl border border-yellow-500/30 bg-yellow-900/10">
            <h3 className="text-2xl font-bold text-yellow-400 mb-4">⚠️ Important Disclosures</h3>
            <div className="space-y-3 text-slate-300">
              <p>
                <strong>Educational Content:</strong> This website provides educational information about QuantumCoin technology. 
                No investment advice is provided.
              </p>
              <p>
                <strong>Technology Risk:</strong> QuantumCoin is experimental technology. Mainnet launch timeline 
                and redemption mechanisms are subject to development progress.
              </p>
              <p>
                <strong>Market Risk:</strong> wQC token prices are determined by market forces. No price guarantees or promises are made.
              </p>
              <p>
                <strong>Regulatory Risk:</strong> Cryptocurrency regulations vary by jurisdiction. 
                Ensure compliance with local laws before participating.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* CSS Animations */}
      <style jsx global>{`
        .token-particles {
          position: absolute;
          width: 100%;
          height: 100%;
          background-image: 
            radial-gradient(circle at 10% 20%, rgba(139, 92, 246, 0.15) 0%, transparent 50%),
            radial-gradient(circle at 80% 80%, rgba(236, 72, 153, 0.15) 0%, transparent 50%),
            radial-gradient(circle at 40% 40%, rgba(6, 182, 212, 0.15) 0%, transparent 50%);
          animation: token-float 25s ease-in-out infinite;
        }
        
        .pulse-rings {
          position: absolute;
          top: 50%;
          left: 50%;
          transform: translate(-50%, -50%);
          width: 300px;
          height: 300px;
        }
        
        .pulse-rings::before,
        .pulse-rings::after {
          content: '';
          position: absolute;
          border: 2px solid rgba(139, 92, 246, 0.3);
          border-radius: 50%;
          animation: pulse-ring 4s ease-out infinite;
        }
        
        .pulse-rings::after {
          animation-delay: 2s;
        }
        
        @keyframes token-float {
          0%, 100% { transform: translateY(0px) rotate(0deg); opacity: 1; }
          25% { transform: translateY(-30px) rotate(90deg); opacity: 0.8; }
          50% { transform: translateY(0px) rotate(180deg); opacity: 1; }
          75% { transform: translateY(20px) rotate(270deg); opacity: 0.9; }
        }
        
        @keyframes pulse-ring {
          0% {
            transform: scale(0.1);
            opacity: 1;
          }
          80%, 100% {
            transform: scale(1.2);
            opacity: 0;
          }
        }
        
        @keyframes bounce {
          0%, 20%, 53%, 80%, 100% {
            transform: translate3d(0, 0, 0);
          }
          40%, 43% {
            transform: translate3d(0, -20px, 0);
          }
          70% {
            transform: translate3d(0, -10px, 0);
          }
          90% {
            transform: translate3d(0, -4px, 0);
          }
        }
        
        .animate-bounce {
          animation: bounce 2s infinite;
        }
        
        /* Live Price Updates */
        #wqc-price.updating {
          animation: price-update 0.5s ease-in-out;
        }
        
        @keyframes price-update {
          0% { transform: scale(1); }
          50% { transform: scale(1.1); color: #10b981; }
          100% { transform: scale(1); }
        }
      `}</style>

      {/* Live Price Update Script */}
      <script
        dangerouslySetInnerHTML={{
          __html: `
            // Simulate live price updates (replace with real API)
            function updatePrices() {
              const priceElement = document.getElementById('wqc-price');
              const mcapElement = document.getElementById('market-cap');
              const volumeElement = document.getElementById('volume-24h');
              const liquidityElement = document.getElementById('uniswap-liquidity');
              
              // Simulate price data (replace with real Uniswap/CoinGecko API)
              const mockPrice = (Math.random() * 0.1 + 0.05).toFixed(4);
              const mockMcap = (parseFloat(mockPrice) * 100000).toFixed(0);
              const mockVolume = (Math.random() * 10000 + 1000).toFixed(0);
              const mockLiquidity = (Math.random() * 50000 + 10000).toFixed(0);
              
              if (priceElement) {
                priceElement.textContent = '$' + mockPrice;
                priceElement.classList.add('updating');
                setTimeout(() => priceElement.classList.remove('updating'), 500);
              }
              
              if (mcapElement) mcapElement.textContent = '$' + mockMcap;
              if (volumeElement) volumeElement.textContent = '$' + mockVolume;
              if (liquidityElement) liquidityElement.textContent = '$' + mockLiquidity;
            }
            
            // Update prices every 10 seconds
            setInterval(updatePrices, 10000);
            
            // Initial update
            setTimeout(updatePrices, 2000);
          `
        }}
      />
    </main>
  )
}
