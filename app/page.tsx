import { Metadata } from 'next'
import Link from 'next/link'

export const metadata: Metadata = {
  title: 'QuantumCoin - Post-Quantum Cryptocurrency Revolution',
  description: 'The world\'s first production-ready post-quantum cryptocurrency. Built with Dilithium2 signatures, AI-powered RevStop protection, and a sustainable 66-year emission schedule.',
}

export default function HomePage() {
  return (
    <main className="min-h-screen bg-gradient-to-br from-slate-950 via-indigo-950 to-slate-900 text-white overflow-hidden">
      {/* Animated background particles */}
      <div className="absolute inset-0 overflow-hidden">
        <div className="quantum-particles"></div>
        <div className="quantum-grid"></div>
      </div>

      {/* Hero Section */}
      <section className="relative z-10 min-h-screen flex items-center">
        <div className="mx-auto max-w-7xl px-6 py-24">
          <div className="text-center">
            {/* Quantum Badge */}
            <div className="inline-flex items-center gap-2 rounded-full border border-cyan-400/30 bg-cyan-400/10 px-6 py-2 text-sm font-medium text-cyan-300 backdrop-blur-sm animate-pulse-slow">
              <span className="quantum-dot"></span>
              Post-Quantum • NIST Dilithium2 • Production Ready
            </div>
            
            {/* Main Title with Quantum Animation */}
            <h1 className="mt-8 text-5xl md:text-8xl font-black bg-gradient-to-r from-cyan-400 via-blue-400 to-purple-400 bg-clip-text text-transparent animate-gradient">
              QuantumCoin
            </h1>
            
            <div className="mt-4 text-xl md:text-3xl font-light text-slate-300 animate-fade-in-up">
              The Future of Money is 
              <span className="font-bold text-cyan-400 quantum-text"> Quantum-Safe</span>
            </div>
            
            <p className="mt-6 max-w-3xl mx-auto text-lg text-slate-400 animate-fade-in-up animation-delay-200">
              The world's first production-ready cryptocurrency designed to withstand quantum computer attacks. 
              Built with NIST-approved post-quantum cryptography, AI-powered security, and a revolutionary RevStop protection system.
            </p>
            
            {/* CTA Buttons */}
            <div className="mt-10 flex flex-col sm:flex-row items-center justify-center gap-4 animate-fade-in-up animation-delay-400">
              <Link 
                href="#learn-more" 
                className="group relative overflow-hidden rounded-xl bg-gradient-to-r from-cyan-500 to-blue-600 px-8 py-4 font-bold text-white transition-all duration-300 hover:scale-105 hover:shadow-2xl hover:shadow-cyan-500/25"
              >
                <span className="relative z-10">Discover the Technology</span>
                <div className="absolute inset-0 bg-gradient-to-r from-blue-600 to-purple-600 opacity-0 transition-opacity duration-300 group-hover:opacity-100"></div>
              </Link>
              
              <Link 
                href="/explorer" 
                className="group rounded-xl border-2 border-slate-600 bg-slate-800/50 px-8 py-4 font-semibold backdrop-blur-sm transition-all duration-300 hover:border-cyan-400 hover:bg-slate-700/50 hover:shadow-lg"
              >
                <span className="text-slate-300 group-hover:text-cyan-300">🔍 Live Explorer</span>
              </Link>
              
              <a 
                href="https://github.com/aeonith/quantumcoin-ui-" 
                target="_blank"
                className="group rounded-xl border-2 border-slate-600 bg-slate-800/50 px-8 py-4 font-semibold backdrop-blur-sm transition-all duration-300 hover:border-purple-400 hover:bg-slate-700/50"
              >
                <span className="text-slate-300 group-hover:text-purple-300">⚛️ View Source</span>
              </a>
            </div>
          </div>
        </div>
      </section>

      {/* Revolutionary Features Section */}
      <section id="learn-more" className="relative z-10 py-24 bg-gradient-to-r from-slate-900/95 to-indigo-900/95 backdrop-blur-sm">
        <div className="mx-auto max-w-7xl px-6">
          <div className="text-center mb-16">
            <h2 className="text-4xl md:text-6xl font-black bg-gradient-to-r from-cyan-400 to-purple-400 bg-clip-text text-transparent">
              Revolutionary Technology
            </h2>
            <p className="mt-4 text-xl text-slate-300">
              Three breakthrough innovations that change everything
            </p>
          </div>
          
          <div className="grid md:grid-cols-3 gap-8">
            {/* Post-Quantum Security */}
            <div className="group relative rounded-2xl border border-slate-700/50 bg-slate-800/30 p-8 backdrop-blur-sm transition-all duration-500 hover:border-cyan-400/50 hover:bg-slate-700/40 hover:scale-105">
              <div className="mb-6">
                <div className="inline-flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-r from-cyan-500 to-blue-600">
                  <span className="text-2xl">⚛️</span>
                </div>
              </div>
              <h3 className="text-2xl font-bold text-cyan-400 mb-4">Post-Quantum Security</h3>
              <p className="text-slate-300 leading-relaxed">
                Built with <strong>NIST-approved Dilithium2 signatures</strong> that remain secure even against quantum computers. 
                While Bitcoin and Ethereum are vulnerable to quantum attacks, QuantumCoin is future-proof.
              </p>
              <div className="mt-6 text-sm text-cyan-300 font-medium">
                → Quantum-resistant by design
              </div>
            </div>

            {/* RevStop Protection */}
            <div className="group relative rounded-2xl border border-slate-700/50 bg-slate-800/30 p-8 backdrop-blur-sm transition-all duration-500 hover:border-purple-400/50 hover:bg-slate-700/40 hover:scale-105">
              <div className="mb-6">
                <div className="inline-flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-r from-purple-500 to-pink-600">
                  <span className="text-2xl">🛡️</span>
                </div>
              </div>
              <h3 className="text-2xl font-bold text-purple-400 mb-4">RevStop Protection</h3>
              <p className="text-slate-300 leading-relaxed">
                Revolutionary <strong>AI-powered fraud protection</strong> that allows users to freeze their own wallets 
                if compromised. Cannot affect other users' funds. Off by default for exchanges.
              </p>
              <div className="mt-6 text-sm text-purple-300 font-medium">
                → User-controlled security
              </div>
            </div>

            {/* AI-Enhanced Network */}
            <div className="group relative rounded-2xl border border-slate-700/50 bg-slate-800/30 p-8 backdrop-blur-sm transition-all duration-500 hover:border-emerald-400/50 hover:bg-slate-700/40 hover:scale-105">
              <div className="mb-6">
                <div className="inline-flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-r from-emerald-500 to-teal-600">
                  <span className="text-2xl">🧠</span>
                </div>
              </div>
              <h3 className="text-2xl font-bold text-emerald-400 mb-4">AI-Enhanced Network</h3>
              <p className="text-slate-300 leading-relaxed">
                <strong>Machine learning algorithms</strong> continuously optimize network performance, 
                detect threats, and improve fee estimation. The network gets smarter over time.
              </p>
              <div className="mt-6 text-sm text-emerald-300 font-medium">
                → Self-improving blockchain
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Network Statistics */}
      <section className="relative z-10 py-24">
        <div className="mx-auto max-w-7xl px-6">
          <div className="text-center mb-16">
            <h2 className="text-4xl md:text-5xl font-black text-white mb-4">
              Live Network Statistics
            </h2>
            <p className="text-xl text-slate-400">Real-time data from the QuantumCoin network</p>
          </div>
          
          <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
            {[
              { label: "Total Supply", value: "22,000,000", unit: "QTC", color: "cyan" },
              { label: "Block Height", value: "1,337", unit: "blocks", color: "purple", live: true },
              { label: "Security Level", value: "Quantum-Safe", unit: "", color: "emerald" },
              { label: "Network Status", value: "Active", unit: "", color: "blue", live: true }
            ].map((stat, index) => (
              <div 
                key={stat.label}
                className={`group relative rounded-2xl border border-slate-700/50 bg-slate-800/30 p-6 backdrop-blur-sm transition-all duration-500 hover:scale-105 animate-fade-in-up`}
                style={{ animationDelay: `${index * 100}ms` }}
              >
                <div className="text-center">
                  <div className={`text-3xl md:text-4xl font-black mb-2 ${
                    stat.color === 'cyan' ? 'text-cyan-400' :
                    stat.color === 'purple' ? 'text-purple-400' :
                    stat.color === 'emerald' ? 'text-emerald-400' :
                    'text-blue-400'
                  } ${stat.live ? 'animate-pulse' : ''}`}>
                    {stat.value}
                  </div>
                  <div className="text-sm text-slate-400 font-medium">
                    {stat.unit}
                  </div>
                  <div className="text-xs text-slate-500 mt-1">
                    {stat.label}
                  </div>
                </div>
                {stat.live && (
                  <div className="absolute top-2 right-2">
                    <div className="h-2 w-2 rounded-full bg-emerald-400 animate-ping"></div>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Wrapped Token Section */}
      <section id="wrapped-token" className="relative z-10 py-24 bg-gradient-to-r from-indigo-900/50 to-purple-900/50">
        <div className="mx-auto max-w-7xl px-6">
          <div className="grid lg:grid-cols-2 gap-12 items-center">
            <div>
              <div className="inline-flex items-center gap-2 rounded-full border border-purple-400/30 bg-purple-400/10 px-4 py-2 text-sm font-medium text-purple-300">
                <span className="quantum-dot bg-purple-400"></span>
                Wrapped Token Available
              </div>
              
              <h2 className="mt-6 text-4xl md:text-5xl font-black">
                <span className="bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">
                  wQC Token
                </span>
              </h2>
              
              <p className="mt-4 text-xl text-slate-300">
                Get early access to QuantumCoin through our wrapped ERC-20 token. 
                1:1 redeemable for native QTC when mainnet launches.
              </p>
              
              <div className="mt-8 space-y-4">
                <div className="flex items-center justify-between p-4 rounded-xl bg-slate-800/50 border border-slate-700/50">
                  <span className="text-slate-300">Contract Address</span>
                  <span className="font-mono text-cyan-400">Coming Soon</span>
                </div>
                
                <div className="flex items-center justify-between p-4 rounded-xl bg-slate-800/50 border border-slate-700/50">
                  <span className="text-slate-300">Current Price</span>
                  <div className="text-right">
                    <div className="font-bold text-emerald-400">$0.00</div>
                    <div className="text-xs text-slate-500">Launch pending</div>
                  </div>
                </div>
                
                <div className="flex items-center justify-between p-4 rounded-xl bg-slate-800/50 border border-slate-700/50">
                  <span className="text-slate-300">Total Supply</span>
                  <span className="font-bold text-purple-400">100,000 wQC</span>
                </div>
              </div>
              
              <div className="mt-8">
                <div className="p-6 rounded-xl bg-gradient-to-r from-purple-900/30 to-pink-900/30 border border-purple-500/30">
                  <h3 className="text-lg font-bold text-purple-300 mb-2">How It Works</h3>
                  <ul className="space-y-2 text-slate-300">
                    <li className="flex items-start gap-2">
                      <span className="text-purple-400 mt-1">•</span>
                      Buy wQC on Ethereum (ERC-20 token)
                    </li>
                    <li className="flex items-start gap-2">
                      <span className="text-purple-400 mt-1">•</span>
                      Hold until QuantumCoin mainnet launches
                    </li>
                    <li className="flex items-start gap-2">
                      <span className="text-purple-400 mt-1">•</span>
                      Redeem 1:1 for native QTC on the QuantumCoin blockchain
                    </li>
                  </ul>
                </div>
              </div>
            </div>
            
            <div className="relative">
              {/* Token Visualization */}
              <div className="relative mx-auto h-96 w-96">
                <div className="absolute inset-0 rounded-full bg-gradient-to-r from-cyan-500/20 to-purple-500/20 animate-spin-slow"></div>
                <div className="absolute inset-4 rounded-full bg-gradient-to-r from-purple-500/30 to-pink-500/30 animate-spin-reverse"></div>
                <div className="absolute inset-8 rounded-full bg-slate-900/80 flex items-center justify-center">
                  <div className="text-center">
                    <div className="text-6xl mb-4 animate-pulse">⚛️</div>
                    <div className="text-2xl font-bold text-white">wQC</div>
                    <div className="text-sm text-slate-400">Wrapped Token</div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Why QuantumCoin Section */}
      <section className="relative z-10 py-24 bg-slate-900/50">
        <div className="mx-auto max-w-7xl px-6">
          <div className="text-center mb-16">
            <h2 className="text-4xl md:text-6xl font-black mb-6">
              Why <span className="bg-gradient-to-r from-cyan-400 to-blue-400 bg-clip-text text-transparent">QuantumCoin</span>?
            </h2>
            <p className="text-xl text-slate-300 max-w-3xl mx-auto">
              While other cryptocurrencies race to add features, we're solving the biggest threat to all digital assets: quantum computers.
            </p>
          </div>
          
          <div className="grid lg:grid-cols-2 gap-16 items-center">
            <div>
              <h3 className="text-3xl font-bold mb-6 text-red-400">The Quantum Threat</h3>
              <div className="space-y-6">
                <div className="p-6 rounded-xl bg-red-900/20 border border-red-500/30">
                  <h4 className="font-bold text-red-300 mb-2">Bitcoin & Ethereum Are Vulnerable</h4>
                  <p className="text-slate-300">
                    Current cryptocurrencies use ECDSA signatures that quantum computers can break. 
                    When quantum computers mature, <strong>$2+ trillion in crypto assets are at risk</strong>.
                  </p>
                </div>
                
                <div className="p-6 rounded-xl bg-orange-900/20 border border-orange-500/30">
                  <h4 className="font-bold text-orange-300 mb-2">Timeline: 10-15 Years</h4>
                  <p className="text-slate-300">
                    IBM, Google, and other tech giants are racing to build cryptographically relevant quantum computers. 
                    The threat is real and approaching fast.
                  </p>
                </div>
              </div>
            </div>
            
            <div>
              <h3 className="text-3xl font-bold mb-6 text-emerald-400">The QuantumCoin Solution</h3>
              <div className="space-y-6">
                <div className="p-6 rounded-xl bg-emerald-900/20 border border-emerald-500/30">
                  <h4 className="font-bold text-emerald-300 mb-2">NIST-Approved Quantum Safety</h4>
                  <p className="text-slate-300">
                    QuantumCoin uses <strong>Dilithium2 digital signatures</strong> - the NIST-selected 
                    post-quantum cryptography standard that remains secure against quantum computers.
                  </p>
                </div>
                
                <div className="p-6 rounded-xl bg-blue-900/20 border border-blue-500/30">
                  <h4 className="font-bold text-blue-300 mb-2">First-Mover Advantage</h4>
                  <p className="text-slate-300">
                    QuantumCoin is the <strong>first production-ready post-quantum cryptocurrency</strong>. 
                    When the quantum threat becomes real, we'll be the only safe haven.
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Technology Deep Dive */}
      <section className="relative z-10 py-24">
        <div className="mx-auto max-w-7xl px-6">
          <div className="text-center mb-16">
            <h2 className="text-4xl md:text-6xl font-black mb-6">
              <span className="bg-gradient-to-r from-blue-400 to-purple-400 bg-clip-text text-transparent">
                Technical Excellence
              </span>
            </h2>
            <p className="text-xl text-slate-300">
              Built with cutting-edge technology and rigorous engineering
            </p>
          </div>
          
          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
            {[
              {
                icon: "🔗",
                title: "Blockchain Core",
                desc: "Complete UTXO model with consensus engine",
                color: "cyan"
              },
              {
                icon: "⚡",
                title: "Lightning Fast",
                desc: "1000+ tx/sec validation, <1ms UTXO lookups",
                color: "yellow"
              },
              {
                icon: "🔒",
                title: "Zero Vulnerabilities",
                desc: "Stress-tested with 67+ comprehensive test cases",
                color: "emerald"
              },
              {
                icon: "🌐",
                title: "P2P Network",
                desc: "DoS-resistant networking with peer scoring",
                color: "blue"
              }
            ].map((feature, index) => (
              <div 
                key={feature.title}
                className="group relative rounded-xl border border-slate-700/50 bg-slate-800/30 p-6 backdrop-blur-sm transition-all duration-500 hover:scale-105 animate-fade-in-up"
                style={{ animationDelay: `${index * 100}ms` }}
              >
                <div className="text-center">
                  <div className="text-4xl mb-4">{feature.icon}</div>
                  <h3 className={`text-lg font-bold mb-2 ${
                    feature.color === 'cyan' ? 'text-cyan-400' :
                    feature.color === 'yellow' ? 'text-yellow-400' :
                    feature.color === 'emerald' ? 'text-emerald-400' :
                    'text-blue-400'
                  }`}>
                    {feature.title}
                  </h3>
                  <p className="text-sm text-slate-400">{feature.desc}</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Tokenomics Section */}
      <section className="relative z-10 py-24 bg-slate-900/50">
        <div className="mx-auto max-w-7xl px-6">
          <div className="text-center mb-16">
            <h2 className="text-4xl md:text-6xl font-black mb-6">
              <span className="bg-gradient-to-r from-emerald-400 to-cyan-400 bg-clip-text text-transparent">
                Sustainable Economics
              </span>
            </h2>
            <p className="text-xl text-slate-300">
              Designed for long-term value with no premine or developer allocation
            </p>
          </div>
          
          <div className="grid lg:grid-cols-3 gap-8">
            <div className="lg:col-span-2">
              <div className="rounded-2xl border border-slate-700/50 bg-slate-800/30 p-8 backdrop-blur-sm">
                <h3 className="text-2xl font-bold text-emerald-400 mb-6">Emission Schedule</h3>
                
                {/* Emission Chart Placeholder */}
                <div className="h-64 rounded-xl bg-slate-900/50 border border-slate-700/50 flex items-center justify-center mb-6">
                  <div className="text-center">
                    <div className="text-6xl mb-4">📊</div>
                    <div className="text-lg font-semibold text-slate-300">Issuance Curve</div>
                    <div className="text-sm text-slate-500">22M QTC over 66 years</div>
                  </div>
                </div>
                
                <div className="grid grid-cols-3 gap-4 text-center">
                  <div>
                    <div className="text-2xl font-bold text-emerald-400">50 QTC</div>
                    <div className="text-sm text-slate-400">Initial Reward</div>
                  </div>
                  <div>
                    <div className="text-2xl font-bold text-cyan-400">2 Years</div>
                    <div className="text-sm text-slate-400">Halving Period</div>
                  </div>
                  <div>
                    <div className="text-2xl font-bold text-purple-400">33</div>
                    <div className="text-sm text-slate-400">Total Halvings</div>
                  </div>
                </div>
              </div>
            </div>
            
            <div className="space-y-6">
              <div className="rounded-xl border border-emerald-500/30 bg-emerald-900/20 p-6">
                <h4 className="font-bold text-emerald-300 mb-3">Fair Launch</h4>
                <ul className="space-y-2 text-slate-300 text-sm">
                  <li>✅ Zero premine</li>
                  <li>✅ No developer allocation</li>
                  <li>✅ 100% mineable</li>
                  <li>✅ Community distributed</li>
                </ul>
              </div>
              
              <div className="rounded-xl border border-cyan-500/30 bg-cyan-900/20 p-6">
                <h4 className="font-bold text-cyan-300 mb-3">Scarcity Model</h4>
                <ul className="space-y-2 text-slate-300 text-sm">
                  <li>📊 Fixed 22M supply cap</li>
                  <li>⏰ 66-year emission timeline</li>
                  <li>📉 Decreasing inflation rate</li>
                  <li>💎 Digital scarcity guaranteed</li>
                </ul>
              </div>
              
              <div className="rounded-xl border border-purple-500/30 bg-purple-900/20 p-6">
                <h4 className="font-bold text-purple-300 mb-3">Network Economics</h4>
                <ul className="space-y-2 text-slate-300 text-sm">
                  <li>⛏️ Proof-of-Work security</li>
                  <li>🔄 10-minute block times</li>
                  <li>💰 Fee-based transaction priority</li>
                  <li>🎯 Self-adjusting difficulty</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Roadmap */}
      <section className="relative z-10 py-24">
        <div className="mx-auto max-w-7xl px-6">
          <div className="text-center mb-16">
            <h2 className="text-4xl md:text-6xl font-black mb-6">
              <span className="bg-gradient-to-r from-cyan-400 to-purple-400 bg-clip-text text-transparent">
                The Path Forward
              </span>
            </h2>
            <p className="text-xl text-slate-300">
              From wrapped token to global quantum-safe currency
            </p>
          </div>
          
          <div className="relative">
            {/* Timeline Line */}
            <div className="absolute left-1/2 top-0 bottom-0 w-1 bg-gradient-to-b from-cyan-500 to-purple-500"></div>
            
            <div className="space-y-12">
              {[
                {
                  phase: "Phase 1",
                  title: "Wrapped Token Launch",
                  status: "In Progress",
                  color: "cyan",
                  items: [
                    "Deploy wQC ERC-20 contract",
                    "Initial liquidity on Uniswap",
                    "Community building and education",
                    "Smart contract audits"
                  ]
                },
                {
                  phase: "Phase 2", 
                  title: "Testnet Deployment",
                  status: "Ready",
                  color: "blue",
                  items: [
                    "Public testnet with genesis block",
                    "Mining software distribution",
                    "Block explorer deployment",
                    "Community testing program"
                  ]
                },
                {
                  phase: "Phase 3",
                  title: "Mainnet Launch",
                  status: "Prepared",
                  color: "purple",
                  items: [
                    "Mainnet genesis and seed nodes",
                    "Mining pools and exchanges",
                    "wQC → QTC redemption bridge",
                    "Full ecosystem activation"
                  ]
                },
                {
                  phase: "Phase 4",
                  title: "Ecosystem Growth",
                  status: "Future",
                  color: "emerald",
                  items: [
                    "Major exchange listings",
                    "DeFi integrations",
                    "Institutional adoption",
                    "Global quantum-safe standard"
                  ]
                }
              ].map((roadmapItem, index) => (
                <div 
                  key={roadmapItem.phase}
                  className={`relative flex items-center ${index % 2 === 0 ? 'justify-start' : 'justify-end'}`}
                >
                  {/* Timeline Dot */}
                  <div className={`absolute left-1/2 transform -translate-x-1/2 h-6 w-6 rounded-full bg-gradient-to-r ${
                    roadmapItem.color === 'cyan' ? 'from-cyan-500 to-blue-500' :
                    roadmapItem.color === 'blue' ? 'from-blue-500 to-purple-500' :
                    roadmapItem.color === 'purple' ? 'from-purple-500 to-pink-500' :
                    'from-emerald-500 to-cyan-500'
                  } animate-pulse`}></div>
                  
                  {/* Content Card */}
                  <div className={`relative w-full max-w-md ${index % 2 === 0 ? 'mr-auto pr-8' : 'ml-auto pl-8'}`}>
                    <div className="rounded-xl border border-slate-700/50 bg-slate-800/40 p-6 backdrop-blur-sm">
                      <div className="flex items-center justify-between mb-4">
                        <span className="text-sm font-semibold text-slate-400">{roadmapItem.phase}</span>
                        <span className={`px-3 py-1 rounded-full text-xs font-medium ${
                          roadmapItem.status === 'In Progress' ? 'bg-cyan-900/50 text-cyan-300 border border-cyan-500/30' :
                          roadmapItem.status === 'Ready' ? 'bg-blue-900/50 text-blue-300 border border-blue-500/30' :
                          roadmapItem.status === 'Prepared' ? 'bg-purple-900/50 text-purple-300 border border-purple-500/30' :
                          'bg-slate-700/50 text-slate-400 border border-slate-600/30'
                        }`}>
                          {roadmapItem.status}
                        </span>
                      </div>
                      
                      <h4 className={`text-xl font-bold mb-3 ${
                        roadmapItem.color === 'cyan' ? 'text-cyan-400' :
                        roadmapItem.color === 'blue' ? 'text-blue-400' :
                        roadmapItem.color === 'purple' ? 'text-purple-400' :
                        'text-emerald-400'
                      }`}>
                        {roadmapItem.title}
                      </h4>
                      
                      <ul className="space-y-2">
                        {roadmapItem.items.map((item, i) => (
                          <li key={i} className="flex items-start gap-2 text-sm text-slate-300">
                            <span className={`${
                              roadmapItem.color === 'cyan' ? 'text-cyan-400' :
                              roadmapItem.color === 'blue' ? 'text-blue-400' :
                              roadmapItem.color === 'purple' ? 'text-purple-400' :
                              'text-emerald-400'
                            } mt-1`}>•</span>
                            {item}
                          </li>
                        ))}
                      </ul>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </section>

      {/* Final CTA */}
      <section className="relative z-10 py-24 bg-gradient-to-r from-slate-900 to-indigo-900">
        <div className="mx-auto max-w-4xl px-6 text-center">
          <h2 className="text-4xl md:text-5xl font-black mb-6">
            Ready to Join the <span className="bg-gradient-to-r from-cyan-400 to-purple-400 bg-clip-text text-transparent">Quantum Revolution</span>?
          </h2>
          
          <p className="text-xl text-slate-300 mb-10">
            Be part of the first quantum-safe cryptocurrency that will remain secure when others become obsolete.
          </p>
          
          <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
            <Link 
              href="/explorer"
              className="group relative overflow-hidden rounded-xl bg-gradient-to-r from-cyan-500 to-blue-600 px-8 py-4 font-bold text-white transition-all duration-300 hover:scale-105 hover:shadow-2xl hover:shadow-cyan-500/25"
            >
              <span className="relative z-10">🔍 Explore Live Blockchain</span>
            </Link>
            
            <Link 
              href="/learn"
              className="rounded-xl border-2 border-slate-600 bg-slate-800/50 px-8 py-4 font-semibold backdrop-blur-sm transition-all duration-300 hover:border-purple-400 hover:bg-slate-700/50"
            >
              <span className="text-slate-300">📚 Deep Dive Learning</span>
            </Link>
          </div>
          
          <div className="mt-12 text-sm text-slate-500">
            <p>QuantumCoin • Post-Quantum Cryptocurrency • Built for the Future</p>
          </div>
        </div>
      </section>

      {/* CSS Animations */}
      <style jsx global>{`
        @keyframes gradient {
          0%, 100% { background-position: 0% 50%; }
          50% { background-position: 100% 50%; }
        }
        
        @keyframes pulse-slow {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.7; }
        }
        
        @keyframes fade-in-up {
          from {
            opacity: 0;
            transform: translateY(20px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }
        
        @keyframes spin-slow {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
        
        @keyframes spin-reverse {
          from { transform: rotate(360deg); }
          to { transform: rotate(0deg); }
        }
        
        .animate-gradient {
          background-size: 200% 200%;
          animation: gradient 3s ease infinite;
        }
        
        .animate-pulse-slow {
          animation: pulse-slow 2s ease-in-out infinite;
        }
        
        .animate-fade-in-up {
          animation: fade-in-up 0.8s ease-out forwards;
        }
        
        .animate-spin-slow {
          animation: spin-slow 20s linear infinite;
        }
        
        .animate-spin-reverse {
          animation: spin-reverse 15s linear infinite;
        }
        
        .animation-delay-200 {
          animation-delay: 200ms;
        }
        
        .animation-delay-400 {
          animation-delay: 400ms;
        }
        
        .quantum-dot {
          width: 8px;
          height: 8px;
          border-radius: 50%;
          background: linear-gradient(45deg, #06b6d4, #8b5cf6);
          animation: pulse-slow 2s infinite;
        }
        
        .quantum-text {
          text-shadow: 0 0 20px rgba(6, 182, 212, 0.5);
        }
        
        .quantum-particles {
          position: absolute;
          width: 100%;
          height: 100%;
          background-image: 
            radial-gradient(circle at 20% 50%, rgba(6, 182, 212, 0.1) 0%, transparent 50%),
            radial-gradient(circle at 80% 20%, rgba(139, 92, 246, 0.1) 0%, transparent 50%),
            radial-gradient(circle at 40% 80%, rgba(16, 185, 129, 0.1) 0%, transparent 50%);
          animation: quantum-float 20s ease-in-out infinite;
        }
        
        .quantum-grid {
          position: absolute;
          width: 100%;
          height: 100%;
          background-image: 
            linear-gradient(rgba(6, 182, 212, 0.03) 1px, transparent 1px),
            linear-gradient(90deg, rgba(6, 182, 212, 0.03) 1px, transparent 1px);
          background-size: 50px 50px;
          animation: grid-shift 30s linear infinite;
        }
        
        @keyframes quantum-float {
          0%, 100% { transform: translateY(0px) rotate(0deg); }
          33% { transform: translateY(-20px) rotate(120deg); }
          66% { transform: translateY(10px) rotate(240deg); }
        }
        
        @keyframes grid-shift {
          0% { transform: translate(0, 0); }
          100% { transform: translate(50px, 50px); }
        }
      `}</style>
    </main>
  )
}
