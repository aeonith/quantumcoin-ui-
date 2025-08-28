import type { Metadata } from 'next'
import '../src/styles/globals.css'

export const metadata: Metadata = {
  title: {
    template: '%s | QuantumCoin',
    default: 'QuantumCoin - Post-Quantum Cryptocurrency Revolution',
  },
  description: 'The world\'s first production-ready post-quantum cryptocurrency with AI-powered RevStop protection.',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <head>
        <meta name="viewport" content="width=device-width, initial-scale=1" />
      </head>
      <body>
        <nav className="fixed top-0 left-0 right-0 z-50 bg-slate-900/80 backdrop-blur-sm border-b border-slate-700/50">
          <div className="mx-auto max-w-7xl px-6 py-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <span className="text-2xl">⚛️</span>
                <span className="text-xl font-bold bg-gradient-to-r from-cyan-400 to-purple-400 bg-clip-text text-transparent">
                  QuantumCoin
                </span>
              </div>
              
              <div className="hidden md:flex items-center gap-6">
                <a href="/" className="text-slate-300 hover:text-cyan-400 transition-colors">Home</a>
                <a href="/wqc" className="text-slate-300 hover:text-purple-400 transition-colors">wQC Token</a>
                <a href="/learn" className="text-slate-300 hover:text-blue-400 transition-colors">Learn</a>
                <a href="/releases" className="text-slate-300 hover:text-emerald-400 transition-colors">Releases</a>
                <a href="/explorer" className="text-slate-300 hover:text-yellow-400 transition-colors">Explorer</a>
                <a href="https://github.com/aeonith/quantumcoin-ui-" target="_blank" 
                   className="rounded-lg bg-gradient-to-r from-cyan-500 to-blue-600 px-4 py-2 text-sm font-semibold text-white hover:scale-105 transition-all duration-300">
                  GitHub
                </a>
              </div>
            </div>
          </div>
        </nav>

        <div className="pt-16">
          {children}
        </div>
      </body>
    </html>
  )
}
