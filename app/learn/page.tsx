import { Metadata } from 'next'
import Link from 'next/link'

export const metadata: Metadata = {
  title: 'Learn About QuantumCoin | Post-Quantum Cryptocurrency Education',
  description: 'Deep dive into QuantumCoin technology: post-quantum cryptography, RevStop protection, tokenomics, and the quantum computing threat to existing cryptocurrencies.',
}

export default function LearnPage() {
  return (
    <main className="min-h-screen bg-gradient-to-br from-slate-950 via-blue-950 to-indigo-950 text-white">
      {/* Animated Background */}
      <div className="absolute inset-0 overflow-hidden">
        <div className="learning-particles"></div>
        <div className="knowledge-grid"></div>
      </div>

      {/* Hero Section */}
      <section className="relative z-10 py-24">
        <div className="mx-auto max-w-7xl px-6">
          <div className="text-center">
            <div className="inline-flex items-center gap-3 rounded-full border border-blue-400/30 bg-blue-400/10 px-6 py-3 text-sm font-medium text-blue-300 backdrop-blur-sm">
              <span className="quantum-dot bg-blue-400"></span>
              Education Center • Technical Deep Dive • Investor Resources
            </div>
            
            <h1 className="mt-8 text-5xl md:text-8xl font-black bg-gradient-to-r from-blue-400 via-cyan-400 to-emerald-400 bg-clip-text text-transparent animate-gradient">
              Learn QuantumCoin
            </h1>
            
            <p className="mt-6 max-w-3xl mx-auto text-xl text-slate-300">
              Understand the technology, economics, and revolutionary innovations that make QuantumCoin 
              the future of digital currency in a post-quantum world.
            </p>
          </div>
        </div>
      </section>

      {/* Navigation */}
      <section className="relative z-10 py-12">
        <div className="mx-auto max-w-5xl px-6">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            {[
              { id: "quantum-threat", title: "Quantum Threat", icon: "⚠️", color: "red" },
              { id: "revstop", title: "RevStop Tech", icon: "🛡️", color: "purple" },
              { id: "tokenomics", title: "Tokenomics", icon: "💰", color: "emerald" },
              { id: "technology", title: "Technology", icon: "⚛️", color: "cyan" }
            ].map((section, index) => (
              <a 
                key={section.id}
                href={`#${section.id}`}
                className={`group relative rounded-xl border border-slate-700/50 bg-slate-800/30 p-6 backdrop-blur-sm transition-all duration-300 hover:scale-105 text-center animate-fade-in-up`}
                style={{ animationDelay: `${index * 100}ms` }}
              >
                <div className="text-3xl mb-2">{section.icon}</div>
                <div className={`text-sm font-semibold ${
                  section.color === 'red' ? 'text-red-400' :
                  section.color === 'purple' ? 'text-purple-400' :
                  section.color === 'emerald' ? 'text-emerald-400' :
                  'text-cyan-400'
                }`}>
                  {section.title}
                </div>
              </a>
            ))}
          </div>
        </div>
      </section>

      {/* The Quantum Threat */}
      <section id="quantum-threat" className="relative z-10 py-24 bg-gradient-to-r from-red-900/30 to-orange-900/30">
        <div className="mx-auto max-w-7xl px-6">
          <div className="text-center mb-16">
            <h2 className="text-4xl md:text-6xl font-black mb-6">
              <span className="bg-gradient-to-r from-red-400 to-orange-400 bg-clip-text text-transparent">
                The Coming Quantum Storm
              </span>
            </h2>
            <p className="text-xl text-slate-300">
              Why the entire cryptocurrency industry faces an existential threat
            </p>
          </div>
          
          <div className="grid lg:grid-cols-2 gap-16">
            <div>
              <h3 className="text-3xl font-bold text-red-400 mb-8">The Problem</h3>
              
              <div className="space-y-6">
                <div className="p-6 rounded-xl bg-red-900/30 border border-red-500/30">
                  <h4 className="text-xl font-bold text-red-300 mb-3">Bitcoin & Ethereum Are Vulnerable</h4>
                  <p className="text-slate-300 leading-relaxed">
                    All major cryptocurrencies use <strong>ECDSA digital signatures</strong> that can be broken 
                    by sufficiently powerful quantum computers using <strong>Shor's algorithm</strong>.
                  </p>
                </div>
                
                <div className="p-6 rounded-xl bg-orange-900/30 border border-orange-500/30">
                  <h4 className="text-xl font-bold text-orange-300 mb-3">$2+ Trillion at Risk</h4>
                  <p className="text-slate-300 leading-relaxed">
                    The entire cryptocurrency market cap becomes vulnerable when quantum computers 
                    can crack private keys in minutes instead of billions of years.
                  </p>
                </div>
                
                <div className="p-6 rounded-xl bg-yellow-900/30 border border-yellow-500/30">
                  <h4 className="text-xl font-bold text-yellow-300 mb-3">Timeline: 10-15 Years</h4>
                  <p className="text-slate-300 leading-relaxed">
                    IBM's quantum roadmap targets 100,000+ qubit systems by 2033. 
                    Google's Sycamore already demonstrates quantum supremacy in specific tasks.
                  </p>
                </div>
              </div>
            </div>
            
            <div>
              <h3 className="text-3xl font-bold text-emerald-400 mb-8">The Solution</h3>
              
              <div className="space-y-6">
                <div className="p-6 rounded-xl bg-emerald-900/30 border border-emerald-500/30">
                  <h4 className="text-xl font-bold text-emerald-300 mb-3">Post-Quantum Cryptography</h4>
                  <p className="text-slate-300 leading-relaxed">
                    QuantumCoin uses <strong>Dilithium2 digital signatures</strong> - selected by NIST 
                    as the standard for post-quantum public-key cryptography.
                  </p>
                </div>
                
                <div className="p-6 rounded-xl bg-cyan-900/30 border border-cyan-500/30">
                  <h4 className="text-xl font-bold text-cyan-300 mb-3">Quantum-Resistant Foundation</h4>
                  <p className="text-slate-300 leading-relaxed">
                    Built from the ground up with quantum resistance in mind. Every cryptographic 
                    operation uses algorithms that remain secure against quantum attacks.
                  </p>
                </div>
                
                <div className="p-6 rounded-xl bg-blue-900/30 border border-blue-500/30">
                  <h4 className="text-xl font-bold text-blue-300 mb-3">Future-Proof Investment</h4>
                  <p className="text-slate-300 leading-relaxed">
                    When quantum computers threaten Bitcoin and Ethereum, QuantumCoin holders will have 
                    the only major quantum-safe store of value.
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* RevStop Technology */}
      <section id="revstop" className="relative z-10 py-24">
        <div className="mx-auto max-w-7xl px-6">
          <div className="text-center mb-16">
            <h2 className="text-4xl md:text-6xl font-black mb-6">
              <span className="bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">
                RevStop Protection
              </span>
            </h2>
            <p className="text-xl text-slate-300">
              Revolutionary AI-powered security that protects users without compromising decentralization
            </p>
          </div>
          
          <div className="grid lg:grid-cols-3 gap-8">
            {/* How RevStop Works */}
            <div className="lg:col-span-2">
              <div className="rounded-2xl border border-purple-500/30 bg-purple-900/20 p-8">
                <h3 className="text-2xl font-bold text-purple-400 mb-6">How RevStop Works</h3>
                
                <div className="space-y-6">
                  <div className="flex items-start gap-4">
                    <div className="flex h-10 w-10 items-center justify-center rounded-full bg-purple-600 text-white font-bold">1</div>
                    <div>
                      <h4 className="font-bold text-purple-300 mb-2">User-Controlled Activation</h4>
                      <p className="text-slate-300">
                        Wallet owners can enable RevStop protection on their own addresses. 
                        It's <strong>off by default</strong> and requires explicit activation.
                      </p>
                    </div>
                  </div>
                  
                  <div className="flex items-start gap-4">
                    <div className="flex h-10 w-10 items-center justify-center rounded-full bg-purple-600 text-white font-bold">2</div>
                    <div>
                      <h4 className="font-bold text-purple-300 mb-2">AI Threat Detection</h4>
                      <p className="text-slate-300">
                        Advanced machine learning monitors transaction patterns and detects anomalies 
                        that might indicate <strong>compromised keys or quantum attacks</strong>.
                      </p>
                    </div>
                  </div>
                  
                  <div className="flex items-start gap-4">
                    <div className="flex h-10 w-10 items-center justify-center rounded-full bg-purple-600 text-white font-bold">3</div>
                    <div>
                      <h4 className="font-bold text-purple-300 mb-2">Emergency Freeze</h4>
                      <p className="text-slate-300">
                        If threats are detected, the system can <strong>temporarily freeze the wallet</strong> 
                        to prevent theft, giving the owner time to secure their keys.
                      </p>
                    </div>
                  </div>
                  
                  <div className="flex items-start gap-4">
                    <div className="flex h-10 w-10 items-center justify-center rounded-full bg-purple-600 text-white font-bold">4</div>
                    <div>
                      <h4 className="font-bold text-purple-300 mb-2">Recovery & Restoration</h4>
                      <p className="text-slate-300">
                        Users can restore access with proper authentication. 
                        <strong>Only the wallet owner</strong> can unfreeze their funds.
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            
            {/* Key Features */}
            <div className="space-y-6">
              <div className="rounded-xl border border-emerald-500/30 bg-emerald-900/20 p-6">
                <h4 className="font-bold text-emerald-300 mb-3">✅ What RevStop IS</h4>
                <ul className="space-y-2 text-slate-300 text-sm">
                  <li>User-controlled wallet protection</li>
                  <li>AI-powered threat detection</li>
                  <li>Quantum attack mitigation</li>
                  <li>Fraud prevention system</li>
                  <li>Emergency account freezing</li>
                </ul>
              </div>
              
              <div className="rounded-xl border border-red-500/30 bg-red-900/20 p-6">
                <h4 className="font-bold text-red-300 mb-3">❌ What RevStop is NOT</h4>
                <ul className="space-y-2 text-slate-300 text-sm">
                  <li>Cannot freeze other users' wallets</li>
                  <li>Cannot seize or confiscate funds</li>
                  <li>Not a central authority control</li>
                  <li>Not enabled on exchanges by default</li>
                  <li>Not a global kill switch</li>
                </ul>
              </div>
              
              <div className="rounded-xl border border-cyan-500/30 bg-cyan-900/20 p-6">
                <h4 className="font-bold text-cyan-300 mb-3">🏦 Exchange Integration</h4>
                <ul className="space-y-2 text-slate-300 text-sm">
                  <li>Disabled by default for exchanges</li>
                  <li>Standard UTXO deposit/withdrawal</li>
                  <li>No interference with trading</li>
                  <li>Compliance-friendly design</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Tokenomics Deep Dive */}
      <section id="tokenomics" className="relative z-10 py-24 bg-slate-900/50">
        <div className="mx-auto max-w-7xl px-6">
          <div className="text-center mb-16">
            <h2 className="text-4xl md:text-6xl font-black mb-6">
              <span className="bg-gradient-to-r from-emerald-400 to-cyan-400 bg-clip-text text-transparent">
                Tokenomics Mastery
              </span>
            </h2>
            <p className="text-xl text-slate-300">
              Designed for sustainable long-term value with mathematical precision
            </p>
          </div>
          
          <div className="grid lg:grid-cols-3 gap-8">
            {/* Supply Schedule */}
            <div className="lg:col-span-2">
              <div className="rounded-2xl border border-emerald-500/30 bg-emerald-900/20 p-8">
                <h3 className="text-2xl font-bold text-emerald-400 mb-6">Supply & Emission Schedule</h3>
                
                <div className="grid md:grid-cols-3 gap-6 mb-8">
                  <div className="text-center p-4 rounded-xl bg-slate-800/50">
                    <div className="text-4xl font-black text-emerald-400 mb-2">22M</div>
                    <div className="text-sm text-slate-400">Maximum Supply</div>
                    <div className="text-xs text-slate-500">Hard-coded limit</div>
                  </div>
                  
                  <div className="text-center p-4 rounded-xl bg-slate-800/50">
                    <div className="text-4xl font-black text-cyan-400 mb-2">66</div>
                    <div className="text-sm text-slate-400">Years</div>
                    <div className="text-xs text-slate-500">Total emission period</div>
                  </div>
                  
                  <div className="text-center p-4 rounded-xl bg-slate-800/50">
                    <div className="text-4xl font-black text-purple-400 mb-2">33</div>
                    <div className="text-sm text-slate-400">Halvings</div>
                    <div className="text-xs text-slate-500">Every 2 years</div>
                  </div>
                </div>
                
                {/* Emission Chart */}
                <div className="h-64 rounded-xl bg-slate-900/50 border border-slate-700/50 p-6">
                  <div className="h-full flex items-end justify-between gap-2">
                    {Array.from({ length: 12 }, (_, i) => {
                      const height = Math.max(10, 100 - (i * 8)); // Decreasing heights
                      return (
                        <div 
                          key={i} 
                          className="bg-gradient-to-t from-emerald-500 to-cyan-400 rounded-t animate-fade-in-up"
                          style={{ 
                            height: `${height}%`, 
                            width: '7%',
                            animationDelay: `${i * 100}ms`
                          }}
                        ></div>
                      );
                    })}
                  </div>
                  <div className="mt-2 text-center text-sm text-slate-400">
                    Emission decreases every 2 years → Long-term scarcity
                  </div>
                </div>
              </div>
            </div>
            
            {/* Economic Principles */}
            <div className="space-y-6">
              <div className="rounded-xl border border-blue-500/30 bg-blue-900/20 p-6">
                <h4 className="font-bold text-blue-300 mb-3">🎯 Fair Launch</h4>
                <ul className="space-y-2 text-slate-300 text-sm">
                  <li>✅ <strong>Zero premine</strong> - no founder allocation</li>
                  <li>✅ <strong>No ICO</strong> - community distributed</li>
                  <li>✅ <strong>100% mineable</strong> - proof of work secured</li>
                  <li>✅ <strong>Transparent</strong> - all code open source</li>
                </ul>
              </div>
              
              <div className="rounded-xl border border-purple-500/30 bg-purple-900/20 p-6">
                <h4 className="font-bold text-purple-300 mb-3">💎 Scarcity Design</h4>
                <ul className="space-y-2 text-slate-300 text-sm">
                  <li>📊 <strong>Decreasing inflation</strong> over time</li>
                  <li>⏰ <strong>Predictable schedule</strong> - no surprises</li>
                  <li>🔒 <strong>Mathematical certainty</strong> - code enforced</li>
                  <li>💰 <strong>Digital gold</strong> - store of value</li>
                </ul>
              </div>
              
              <div className="rounded-xl border border-cyan-500/30 bg-cyan-900/20 p-6">
                <h4 className="font-bold text-cyan-300 mb-3">⚡ Network Economics</h4>
                <ul className="space-y-2 text-slate-300 text-sm">
                  <li>⛏️ <strong>Proof of Work</strong> - battle-tested security</li>
                  <li>🎯 <strong>10-minute blocks</strong> - optimal for payments</li>
                  <li>💸 <strong>Market-based fees</strong> - efficient allocation</li>
                  <li>🔄 <strong>Self-adjusting difficulty</strong> - stable block times</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Technology Deep Dive */}
      <section id="technology" className="relative z-10 py-24">
        <div className="mx-auto max-w-7xl px-6">
          <div className="text-center mb-16">
            <h2 className="text-4xl md:text-6xl font-black mb-6">
              <span className="bg-gradient-to-r from-cyan-400 to-blue-400 bg-clip-text text-transparent">
                Technology Stack
              </span>
            </h2>
            <p className="text-xl text-slate-300">
              Cutting-edge architecture designed for the quantum era
            </p>
          </div>
          
          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
            {[
              {
                title: "Post-Quantum Crypto",
                icon: "⚛️",
                color: "cyan",
                features: [
                  "NIST Dilithium2 signatures",
                  "Blake3 hash functions", 
                  "Quantum-resistant keys",
                  "Future-proof algorithms"
                ]
              },
              {
                title: "Blockchain Core",
                icon: "🔗",
                color: "blue",
                features: [
                  "UTXO transaction model",
                  "Proof-of-Work consensus",
                  "Smart difficulty adjustment",
                  "Merkle tree validation"
                ]
              },
              {
                title: "AI Integration",
                icon: "🧠",
                color: "purple",
                features: [
                  "Threat detection AI",
                  "Fee estimation ML",
                  "Network optimization",
                  "Behavioral analysis"
                ]
              },
              {
                title: "Network Protocol",
                icon: "🌐",
                color: "emerald",
                features: [
                  "P2P mesh networking",
                  "DoS attack protection",
                  "Peer discovery system",
                  "Message propagation"
                ]
              },
              {
                title: "Database Layer",
                icon: "💾",
                color: "indigo",
                features: [
                  "ACID transactions",
                  "WAL crash safety",
                  "UTXO indexing",
                  "Performance optimization"
                ]
              },
              {
                title: "Developer Tools",
                icon: "🛠️",
                color: "pink",
                features: [
                  "RPC API server",
                  "CLI wallet tools",
                  "Block explorer",
                  "Integration libraries"
                ]
              }
            ].map((tech, index) => (
              <div 
                key={tech.title}
                className={`group relative rounded-2xl border border-slate-700/50 bg-slate-800/30 p-8 backdrop-blur-sm transition-all duration-500 hover:scale-105 animate-fade-in-up`}
                style={{ animationDelay: `${index * 100}ms` }}
              >
                <div className="text-center">
                  <div className={`inline-flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-r mb-6 ${
                    tech.color === 'cyan' ? 'from-cyan-500 to-cyan-600' :
                    tech.color === 'blue' ? 'from-blue-500 to-blue-600' :
                    tech.color === 'purple' ? 'from-purple-500 to-purple-600' :
                    tech.color === 'emerald' ? 'from-emerald-500 to-emerald-600' :
                    tech.color === 'indigo' ? 'from-indigo-500 to-indigo-600' :
                    'from-pink-500 to-pink-600'
                  }`}>
                    <span className="text-2xl">{tech.icon}</span>
                  </div>
                  
                  <h3 className={`text-xl font-bold mb-4 ${
                    tech.color === 'cyan' ? 'text-cyan-400' :
                    tech.color === 'blue' ? 'text-blue-400' :
                    tech.color === 'purple' ? 'text-purple-400' :
                    tech.color === 'emerald' ? 'text-emerald-400' :
                    tech.color === 'indigo' ? 'text-indigo-400' :
                    'text-pink-400'
                  }`}>
                    {tech.title}
                  </h3>
                  
                  <ul className="space-y-2 text-left">
                    {tech.features.map((feature, i) => (
                      <li key={i} className="flex items-center gap-2 text-sm text-slate-300">
                        <span className={`${
                          tech.color === 'cyan' ? 'text-cyan-400' :
                          tech.color === 'blue' ? 'text-blue-400' :
                          tech.color === 'purple' ? 'text-purple-400' :
                          tech.color === 'emerald' ? 'text-emerald-400' :
                          tech.color === 'indigo' ? 'text-indigo-400' :
                          'text-pink-400'
                        }`}>•</span>
                        {feature}
                      </li>
                    ))}
                  </ul>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Comparison Table */}
      <section className="relative z-10 py-24 bg-gradient-to-r from-slate-900/80 to-indigo-900/80">
        <div className="mx-auto max-w-7xl px-6">
          <div className="text-center mb-16">
            <h2 className="text-4xl md:text-6xl font-black mb-6">
              <span className="bg-gradient-to-r from-yellow-400 to-orange-400 bg-clip-text text-transparent">
                Cryptocurrency Comparison
              </span>
            </h2>
            <p className="text-xl text-slate-300">
              See how QuantumCoin stacks up against other major cryptocurrencies
            </p>
          </div>
          
          <div className="overflow-x-auto">
            <table className="w-full rounded-2xl border border-slate-700/50 bg-slate-800/30 backdrop-blur-sm">
              <thead>
                <tr className="border-b border-slate-700/50">
                  <th className="p-6 text-left text-slate-300">Feature</th>
                  <th className="p-6 text-center text-orange-400">Bitcoin</th>
                  <th className="p-6 text-center text-blue-400">Ethereum</th>
                  <th className="p-6 text-center text-purple-400">QuantumCoin</th>
                </tr>
              </thead>
              <tbody>
                {[
                  {
                    feature: "Quantum Resistance",
                    bitcoin: "❌ Vulnerable",
                    ethereum: "❌ Vulnerable", 
                    quantumcoin: "✅ Dilithium2"
                  },
                  {
                    feature: "AI Protection",
                    bitcoin: "❌ None",
                    ethereum: "❌ None",
                    quantumcoin: "✅ RevStop AI"
                  },
                  {
                    feature: "Block Time",
                    bitcoin: "~10 minutes",
                    ethereum: "~12 seconds",
                    quantumcoin: "✅ ~10 minutes"
                  },
                  {
                    feature: "Supply Cap",
                    bitcoin: "21M BTC",
                    ethereum: "♾️ Unlimited",
                    quantumcoin: "✅ 22M QTC"
                  },
                  {
                    feature: "Consensus",
                    bitcoin: "Proof of Work",
                    ethereum: "Proof of Stake",
                    quantumcoin: "✅ Proof of Work"
                  },
                  {
                    feature: "Energy Use",
                    bitcoin: "⚠️ High",
                    ethereum: "✅ Low",
                    quantumcoin: "✅ Efficient"
                  },
                  {
                    feature: "Smart Contracts",
                    bitcoin: "⚠️ Limited",
                    ethereum: "✅ Full",
                    quantumcoin: "🔮 Planned"
                  },
                  {
                    feature: "User Protection",
                    bitcoin: "❌ None",
                    ethereum: "❌ None",
                    quantumcoin: "✅ RevStop"
                  }
                ].map((row, index) => (
                  <tr key={row.feature} className={`border-b border-slate-700/30 hover:bg-slate-700/20 animate-fade-in-up`} style={{ animationDelay: `${index * 50}ms` }}>
                    <td className="p-6 font-semibold text-slate-200">{row.feature}</td>
                    <td className="p-6 text-center text-slate-400">{row.bitcoin}</td>
                    <td className="p-6 text-center text-slate-400">{row.ethereum}</td>
                    <td className="p-6 text-center font-semibold text-purple-400">{row.quantumcoin}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </section>

      {/* FAQ Section */}
      <section className="relative z-10 py-24">
        <div className="mx-auto max-w-5xl px-6">
          <div className="text-center mb-16">
            <h2 className="text-4xl md:text-5xl font-black mb-6">
              <span className="bg-gradient-to-r from-cyan-400 to-purple-400 bg-clip-text text-transparent">
                Frequently Asked Questions
              </span>
            </h2>
          </div>
          
          <div className="space-y-6">
            {[
              {
                q: "When will the QuantumCoin mainnet launch?",
                a: "The QuantumCoin mainnet is in final preparation phase. All core technology is complete and stress-tested. Launch timeline depends on final security audits and testnet validation."
              },
              {
                q: "How does wQC redemption work?",
                a: "When mainnet launches, we'll mine the first 100,000 QTC through Proof-of-Work (no premine). wQC holders can then redeem 1:1 for native QTC using a smart contract bridge."
              },
              {
                q: "What makes QuantumCoin quantum-safe?",
                a: "QuantumCoin uses Dilithium2 digital signatures, selected by NIST as the standard for post-quantum cryptography. These signatures remain secure even against quantum computers."
              },
              {
                q: "Can RevStop be used to seize funds?",
                a: "No. RevStop only allows wallet owners to freeze their own addresses. It cannot affect other users' funds and is disabled by default on exchanges."
              },
              {
                q: "How is this different from other 'quantum-resistant' projects?",
                a: "QuantumCoin is the first complete, production-ready implementation with NIST-approved cryptography, AI integration, and comprehensive testing. Most others are theoretical or incomplete."
              }
            ].map((faq, index) => (
              <div 
                key={index}
                className={`p-6 rounded-xl border border-slate-700/50 bg-slate-800/30 backdrop-blur-sm animate-fade-in-up`}
                style={{ animationDelay: `${index * 100}ms` }}
              >
                <h3 className="text-xl font-bold text-cyan-400 mb-3">{faq.q}</h3>
                <p className="text-slate-300 leading-relaxed">{faq.a}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Call to Action */}
      <section className="relative z-10 py-24 bg-gradient-to-r from-purple-900 to-indigo-900">
        <div className="mx-auto max-w-4xl px-6 text-center">
          <h2 className="text-4xl md:text-5xl font-black mb-6">
            Ready to <span className="bg-gradient-to-r from-purple-400 to-cyan-400 bg-clip-text text-transparent">Secure Your Future</span>?
          </h2>
          
          <p className="text-xl text-slate-300 mb-10">
            Join the quantum revolution and protect your wealth from the coming quantum storm.
          </p>
          
          <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
            <Link 
              href="/explorer"
              className="group relative overflow-hidden rounded-xl bg-gradient-to-r from-cyan-500 to-blue-600 px-8 py-4 font-bold text-white transition-all duration-300 hover:scale-105 hover:shadow-2xl hover:shadow-cyan-500/25"
            >
              <span className="relative z-10">🔍 Explore Live Blockchain</span>
            </Link>
            
            <a 
              href="https://github.com/aeonith/quantumcoin-ui-"
              target="_blank"
              className="rounded-xl border-2 border-slate-600 bg-slate-800/50 px-8 py-4 font-semibold backdrop-blur-sm transition-all duration-300 hover:border-purple-400 hover:bg-slate-700/50"
            >
              <span className="text-slate-300">📚 Technical Documentation</span>
            </a>
          </div>
        </div>
      </section>

      {/* Global Styles */}
      <style jsx global>{`
        .learning-particles {
          position: absolute;
          width: 100%;
          height: 100%;
          background-image: 
            radial-gradient(circle at 15% 40%, rgba(59, 130, 246, 0.1) 0%, transparent 50%),
            radial-gradient(circle at 85% 60%, rgba(139, 92, 246, 0.1) 0%, transparent 50%),
            radial-gradient(circle at 50% 20%, rgba(16, 185, 129, 0.1) 0%, transparent 50%);
          animation: learning-float 30s ease-in-out infinite;
        }
        
        .knowledge-grid {
          position: absolute;
          width: 100%;
          height: 100%;
          background-image: 
            linear-gradient(rgba(59, 130, 246, 0.02) 1px, transparent 1px),
            linear-gradient(90deg, rgba(59, 130, 246, 0.02) 1px, transparent 1px);
          background-size: 60px 60px;
          animation: grid-pulse 20s linear infinite;
        }
        
        @keyframes learning-float {
          0%, 100% { 
            transform: translateY(0px) rotate(0deg);
            opacity: 0.8;
          }
          33% { 
            transform: translateY(-40px) rotate(120deg);
            opacity: 1;
          }
          66% { 
            transform: translateY(20px) rotate(240deg);
            opacity: 0.9;
          }
        }
        
        @keyframes grid-pulse {
          0%, 100% { opacity: 0.5; }
          50% { opacity: 1; }
        }
      `}</style>
    </main>
  )
}
