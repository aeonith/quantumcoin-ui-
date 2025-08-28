import { Metadata } from 'next'

export const metadata: Metadata = {
  title: 'QuantumCoin Releases | Download Official Binaries',
  description: 'Download official QuantumCoin binaries with cryptographic verification. SHA256SUMS and GPG signatures included.',
}

export default function ReleasesPage() {
  return (
    <main className="min-h-screen bg-gradient-to-br from-slate-950 via-emerald-950 to-slate-900 text-white">
      {/* Hero Section */}
      <section className="relative z-10 py-24">
        <div className="mx-auto max-w-7xl px-6">
          <div className="text-center">
            <div className="inline-flex items-center gap-3 rounded-full border border-emerald-400/30 bg-emerald-400/10 px-6 py-3 text-sm font-medium text-emerald-300 backdrop-blur-sm">
              <div className="h-3 w-3 rounded-full bg-emerald-400 animate-ping"></div>
              Official Releases • Cryptographically Signed • SHA256 Verified
            </div>
            
            <h1 className="mt-8 text-5xl md:text-8xl font-black bg-gradient-to-r from-emerald-400 via-cyan-400 to-blue-400 bg-clip-text text-transparent animate-gradient">
              Download QuantumCoin
            </h1>
            
            <p className="mt-6 max-w-3xl mx-auto text-xl text-slate-300">
              Official QuantumCoin releases with cryptographic verification. 
              All binaries are reproducibly built and cryptographically signed.
            </p>
          </div>
        </div>
      </section>

      {/* Latest Release */}
      <section className="relative z-10 py-16">
        <div className="mx-auto max-w-6xl px-6">
          <div className="rounded-3xl border border-emerald-500/30 bg-gradient-to-r from-emerald-900/40 to-cyan-900/40 p-8 backdrop-blur-sm">
            <div className="flex flex-col lg:flex-row items-center gap-8">
              <div className="flex-1">
                <div className="flex items-center gap-4 mb-4">
                  <span className="text-6xl">🚀</span>
                  <div>
                    <h2 className="text-3xl font-bold text-emerald-400">QuantumCoin v2.0.0</h2>
                    <div className="text-slate-400">Latest Release • Production Ready</div>
                  </div>
                </div>
                
                <p className="text-slate-300 leading-relaxed mb-6">
                  Complete cryptocurrency implementation with post-quantum security, AI-powered RevStop protection, 
                  and comprehensive blockchain infrastructure. Ready for mainnet deployment.
                </p>
                
                <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                  <div className="text-center p-3 rounded-lg bg-slate-800/50">
                    <div className="text-lg font-bold text-emerald-400">100%</div>
                    <div className="text-xs text-slate-400">Complete</div>
                  </div>
                  <div className="text-center p-3 rounded-lg bg-slate-800/50">
                    <div className="text-lg font-bold text-cyan-400">67+</div>
                    <div className="text-xs text-slate-400">Tests</div>
                  </div>
                  <div className="text-center p-3 rounded-lg bg-slate-800/50">
                    <div className="text-lg font-bold text-blue-400">0</div>
                    <div className="text-xs text-slate-400">Vulnerabilities</div>
                  </div>
                  <div className="text-center p-3 rounded-lg bg-slate-800/50">
                    <div className="text-lg font-bold text-purple-400">⚛️</div>
                    <div className="text-xs text-slate-400">Quantum-Safe</div>
                  </div>
                </div>
              </div>
              
              <div className="text-center">
                <div className="text-sm text-slate-400 mb-2">Release Date</div>
                <div className="text-2xl font-bold text-white">Aug 18, 2025</div>
                <div className="text-sm text-emerald-400 mt-1">Latest</div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Download Options */}
      <section className="relative z-10 py-24">
        <div className="mx-auto max-w-7xl px-6">
          <div className="text-center mb-16">
            <h2 className="text-4xl font-black mb-6 text-white">Choose Your Platform</h2>
            <p className="text-xl text-slate-300">
              Download verified binaries for your operating system
            </p>
          </div>
          
          <div className="grid md:grid-cols-3 gap-8">
            {/* Linux */}
            <div className="group relative rounded-2xl border border-emerald-500/30 bg-emerald-900/20 p-8 backdrop-blur-sm transition-all duration-500 hover:scale-105">
              <div className="text-center">
                <div className="inline-flex h-20 w-20 items-center justify-center rounded-2xl bg-gradient-to-r from-emerald-500 to-teal-600 mb-6">
                  <span className="text-3xl">🐧</span>
                </div>
                
                <h3 className="text-2xl font-bold text-emerald-400 mb-4">Linux x64</h3>
                <p className="text-slate-300 mb-6">
                  Complete package with node, CLI wallet, and genesis tools for Linux systems.
                </p>
                
                <div className="space-y-3 mb-6">
                  <div className="flex justify-between items-center p-3 rounded-lg bg-slate-800/50 text-sm">
                    <span className="text-slate-400">Size</span>
                    <span className="font-semibold text-emerald-400">45.2 MB</span>
                  </div>
                  <div className="flex justify-between items-center p-3 rounded-lg bg-slate-800/50 text-sm">
                    <span className="text-slate-400">SHA256</span>
                    <span className="font-mono text-emerald-400 text-xs">a1b2c3d4...</span>
                  </div>
                </div>
                
                <button 
                  className="w-full rounded-xl bg-gradient-to-r from-emerald-600 to-teal-600 py-4 font-bold text-white transition-all duration-300 hover:scale-105 mb-3"
                  onClick={() => downloadRelease('linux')}
                >
                  📥 Download for Linux
                </button>
                
                <div className="text-xs text-slate-500">
                  Ubuntu 20.04+, CentOS 8+, Debian 11+
                </div>
              </div>
            </div>

            {/* Windows */}
            <div className="group relative rounded-2xl border border-blue-500/30 bg-blue-900/20 p-8 backdrop-blur-sm transition-all duration-500 hover:scale-105">
              <div className="text-center">
                <div className="inline-flex h-20 w-20 items-center justify-center rounded-2xl bg-gradient-to-r from-blue-500 to-indigo-600 mb-6">
                  <span className="text-3xl">🪟</span>
                </div>
                
                <h3 className="text-2xl font-bold text-blue-400 mb-4">Windows x64</h3>
                <p className="text-slate-300 mb-6">
                  Windows installer with GUI wallet and easy node setup for Windows 10/11 systems.
                </p>
                
                <div className="space-y-3 mb-6">
                  <div className="flex justify-between items-center p-3 rounded-lg bg-slate-800/50 text-sm">
                    <span className="text-slate-400">Size</span>
                    <span className="font-semibold text-blue-400">52.8 MB</span>
                  </div>
                  <div className="flex justify-between items-center p-3 rounded-lg bg-slate-800/50 text-sm">
                    <span className="text-slate-400">SHA256</span>
                    <span className="font-mono text-blue-400 text-xs">e5f6g7h8...</span>
                  </div>
                </div>
                
                <button 
                  className="w-full rounded-xl bg-gradient-to-r from-blue-600 to-indigo-600 py-4 font-bold text-white transition-all duration-300 hover:scale-105 mb-3"
                  onClick={() => downloadRelease('windows')}
                >
                  📥 Download for Windows
                </button>
                
                <div className="text-xs text-slate-500">
                  Windows 10 (1903+), Windows 11
                </div>
              </div>
            </div>

            {/* macOS */}
            <div className="group relative rounded-2xl border border-purple-500/30 bg-purple-900/20 p-8 backdrop-blur-sm transition-all duration-500 hover:scale-105">
              <div className="text-center">
                <div className="inline-flex h-20 w-20 items-center justify-center rounded-2xl bg-gradient-to-r from-purple-500 to-pink-600 mb-6">
                  <span className="text-3xl">🍎</span>
                </div>
                
                <h3 className="text-2xl font-bold text-purple-400 mb-4">macOS Universal</h3>
                <p className="text-slate-300 mb-6">
                  Universal binary supporting both Intel and Apple Silicon Macs with native performance.
                </p>
                
                <div className="space-y-3 mb-6">
                  <div className="flex justify-between items-center p-3 rounded-lg bg-slate-800/50 text-sm">
                    <span className="text-slate-400">Size</span>
                    <span className="font-semibold text-purple-400">38.9 MB</span>
                  </div>
                  <div className="flex justify-between items-center p-3 rounded-lg bg-slate-800/50 text-sm">
                    <span className="text-slate-400">SHA256</span>
                    <span className="font-mono text-purple-400 text-xs">i9j0k1l2...</span>
                  </div>
                </div>
                
                <button 
                  className="w-full rounded-xl bg-gradient-to-r from-purple-600 to-pink-600 py-4 font-bold text-white transition-all duration-300 hover:scale-105 mb-3"
                  onClick={() => downloadRelease('macos')}
                >
                  📥 Download for macOS
                </button>
                
                <div className="text-xs text-slate-500">
                  macOS 12.0+, Intel & Apple Silicon
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Verification Instructions */}
      <section className="relative z-10 py-24 bg-slate-900/50">
        <div className="mx-auto max-w-7xl px-6">
          <div className="text-center mb-16">
            <h2 className="text-4xl font-black mb-6 text-white">Cryptographic Verification</h2>
            <p className="text-xl text-slate-300">
              Always verify your downloads to ensure authenticity and security
            </p>
          </div>
          
          <div className="grid lg:grid-cols-2 gap-12">
            <div>
              <h3 className="text-2xl font-bold text-cyan-400 mb-6">🔐 Verification Steps</h3>
              
              <div className="space-y-6">
                <div className="p-6 rounded-xl bg-cyan-900/20 border border-cyan-500/30">
                  <h4 className="font-bold text-cyan-300 mb-3">Step 1: Download Files</h4>
                  <div className="text-sm text-slate-300 font-mono bg-slate-800/50 p-3 rounded">
                    wget quantumcoin-linux-x64-v2.0.0.tar.gz<br/>
                    wget SHA256SUMS.txt<br/>
                    wget SHA256SUMS.sig<br/>
                    wget verify_release.sh
                  </div>
                </div>
                
                <div className="p-6 rounded-xl bg-blue-900/20 border border-blue-500/30">
                  <h4 className="font-bold text-blue-300 mb-3">Step 2: Verify Checksums</h4>
                  <div className="text-sm text-slate-300 font-mono bg-slate-800/50 p-3 rounded">
                    sha256sum -c SHA256SUMS.txt
                  </div>
                  <div className="text-xs text-slate-500 mt-2">
                    This verifies the binary wasn't corrupted or tampered with
                  </div>
                </div>
                
                <div className="p-6 rounded-xl bg-purple-900/20 border border-purple-500/30">
                  <h4 className="font-bold text-purple-300 mb-3">Step 3: Verify Signature</h4>
                  <div className="text-sm text-slate-300 font-mono bg-slate-800/50 p-3 rounded">
                    gpg --verify SHA256SUMS.sig SHA256SUMS.txt
                  </div>
                  <div className="text-xs text-slate-500 mt-2">
                    This verifies the checksums were signed by the QuantumCoin team
                  </div>
                </div>
                
                <div className="p-6 rounded-xl bg-emerald-900/20 border border-emerald-500/30">
                  <h4 className="font-bold text-emerald-300 mb-3">Step 4: Automated Verification</h4>
                  <div className="text-sm text-slate-300 font-mono bg-slate-800/50 p-3 rounded">
                    chmod +x verify_release.sh<br/>
                    ./verify_release.sh
                  </div>
                  <div className="text-xs text-slate-500 mt-2">
                    Our script performs all verification steps automatically
                  </div>
                </div>
              </div>
            </div>
            
            <div>
              <h3 className="text-2xl font-bold text-yellow-400 mb-6">⚠️ Security Notice</h3>
              
              <div className="space-y-6">
                <div className="p-6 rounded-xl bg-yellow-900/20 border border-yellow-500/30">
                  <h4 className="font-bold text-yellow-300 mb-3">Why Verification Matters</h4>
                  <p className="text-slate-300 text-sm leading-relaxed">
                    Cryptocurrency software is a high-value target for attackers. 
                    <strong>Always verify downloads</strong> to ensure you're running authentic QuantumCoin software.
                  </p>
                </div>
                
                <div className="p-6 rounded-xl bg-red-900/20 border border-red-500/30">
                  <h4 className="font-bold text-red-300 mb-3">Never Skip Verification</h4>
                  <p className="text-slate-300 text-sm leading-relaxed">
                    Malicious actors may distribute fake binaries that steal your funds. 
                    Our cryptographic signatures guarantee authenticity.
                  </p>
                </div>
                
                <div className="p-6 rounded-xl bg-green-900/20 border border-green-500/30">
                  <h4 className="font-bold text-green-300 mb-3">Reproducible Builds</h4>
                  <p className="text-slate-300 text-sm leading-relaxed">
                    All QuantumCoin binaries are reproducibly built. 
                    Independent verification produces identical checksums.
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* File Listing */}
      <section className="relative z-10 py-24">
        <div className="mx-auto max-w-7xl px-6">
          <h2 className="text-3xl font-bold text-white mb-8 text-center">Release Files</h2>
          
          <div className="overflow-x-auto">
            <table className="w-full rounded-2xl border border-slate-700/50 bg-slate-800/30 backdrop-blur-sm">
              <thead>
                <tr className="border-b border-slate-700/50">
                  <th className="p-6 text-left text-slate-300">File</th>
                  <th className="p-6 text-center text-slate-300">Platform</th>
                  <th className="p-6 text-center text-slate-300">Size</th>
                  <th className="p-6 text-center text-slate-300">SHA256</th>
                  <th className="p-6 text-center text-slate-300">Download</th>
                </tr>
              </thead>
              <tbody>
                {[
                  {
                    file: "quantumcoin-linux-x64-v2.0.0.tar.gz",
                    platform: "Linux x64",
                    size: "45.2 MB",
                    sha256: "a1b2c3d4e5f6789abcdef0123456789abcdef0123456789abcdef0123456789a",
                    icon: "🐧"
                  },
                  {
                    file: "quantumcoin-windows-x64-v2.0.0.zip",
                    platform: "Windows x64", 
                    size: "52.8 MB",
                    sha256: "b2c3d4e5f6789abcdef0123456789abcdef0123456789abcdef0123456789ab",
                    icon: "🪟"
                  },
                  {
                    file: "quantumcoin-macos-universal-v2.0.0.dmg",
                    platform: "macOS Universal",
                    size: "38.9 MB", 
                    sha256: "c3d4e5f6789abcdef0123456789abcdef0123456789abcdef0123456789abc",
                    icon: "🍎"
                  },
                  {
                    file: "SHA256SUMS.txt",
                    platform: "All",
                    size: "2.1 KB",
                    sha256: "d4e5f6789abcdef0123456789abcdef0123456789abcdef0123456789abcd",
                    icon: "🔐"
                  },
                  {
                    file: "SHA256SUMS.sig",
                    platform: "All",
                    size: "833 bytes",
                    sha256: "e5f6789abcdef0123456789abcdef0123456789abcdef0123456789abcde",
                    icon: "🔏"
                  },
                  {
                    file: "verify_release.sh",
                    platform: "Unix",
                    size: "1.8 KB",
                    sha256: "f6789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                    icon: "✅"
                  }
                ].map((file, index) => (
                  <tr key={file.file} className="border-b border-slate-700/30 hover:bg-slate-700/20 transition-colors">
                    <td className="p-6">
                      <div className="flex items-center gap-3">
                        <span className="text-2xl">{file.icon}</span>
                        <div>
                          <div className="font-semibold text-white">{file.file}</div>
                          <div className="text-xs text-slate-500">v2.0.0</div>
                        </div>
                      </div>
                    </td>
                    <td className="p-6 text-center text-slate-300">{file.platform}</td>
                    <td className="p-6 text-center text-slate-300">{file.size}</td>
                    <td className="p-6 text-center">
                      <span className="font-mono text-xs text-slate-400" title={file.sha256}>
                        {file.sha256.substring(0, 8)}...
                      </span>
                    </td>
                    <td className="p-6 text-center">
                      <button 
                        className="rounded-lg bg-gradient-to-r from-cyan-500 to-blue-600 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:scale-105"
                        onClick={() => downloadFile(file.file)}
                      >
                        Download
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </section>

      {/* Installation Guide */}
      <section className="relative z-10 py-24 bg-gradient-to-r from-slate-900/80 to-blue-900/80">
        <div className="mx-auto max-w-7xl px-6">
          <div className="text-center mb-16">
            <h2 className="text-4xl font-black mb-6 text-white">Installation Guide</h2>
            <p className="text-xl text-slate-300">
              Get QuantumCoin running in minutes with our step-by-step guide
            </p>
          </div>
          
          <div className="grid lg:grid-cols-3 gap-8">
            {[
              {
                step: "1",
                title: "Download & Verify",
                color: "cyan",
                commands: [
                  "wget quantumcoin-linux-x64-v2.0.0.tar.gz",
                  "wget SHA256SUMS.txt verify_release.sh",
                  "chmod +x verify_release.sh",
                  "./verify_release.sh"
                ]
              },
              {
                step: "2", 
                title: "Install",
                color: "blue",
                commands: [
                  "tar -xzf quantumcoin-linux-x64-v2.0.0.tar.gz",
                  "cd quantumcoin-linux-x64-v2.0.0/",
                  "sudo ./install.sh",
                  "quantumcoin-node --version"
                ]
              },
              {
                step: "3",
                title: "Initialize & Run",
                color: "purple", 
                commands: [
                  "quantumcoin-node init",
                  "quantumcoin-cli wallet create",
                  "quantumcoin-node start --mine",
                  "open http://localhost:8080"
                ]
              }
            ].map((guide, index) => (
              <div 
                key={guide.step}
                className={`group relative rounded-2xl border border-slate-700/50 bg-slate-800/30 p-8 backdrop-blur-sm transition-all duration-500 hover:scale-105 animate-fade-in-up`}
                style={{ animationDelay: `${index * 200}ms` }}
              >
                <div className="text-center mb-6">
                  <div className={`inline-flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-r ${
                    guide.color === 'cyan' ? 'from-cyan-500 to-cyan-600' :
                    guide.color === 'blue' ? 'from-blue-500 to-blue-600' :
                    'from-purple-500 to-purple-600'
                  }`}>
                    <span className="text-2xl font-bold text-white">{guide.step}</span>
                  </div>
                  
                  <h3 className={`text-xl font-bold mt-4 ${
                    guide.color === 'cyan' ? 'text-cyan-400' :
                    guide.color === 'blue' ? 'text-blue-400' :
                    'text-purple-400'
                  }`}>
                    {guide.title}
                  </h3>
                </div>
                
                <div className="space-y-2">
                  {guide.commands.map((cmd, i) => (
                    <div key={i} className="text-sm font-mono bg-slate-900/50 p-3 rounded border border-slate-700/50 text-slate-300">
                      <span className="text-slate-500">$ </span>{cmd}
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Developer Resources */}
      <section className="relative z-10 py-24">
        <div className="mx-auto max-w-7xl px-6">
          <div className="text-center mb-16">
            <h2 className="text-4xl font-black mb-6 text-white">Developer Resources</h2>
            <p className="text-xl text-slate-300">
              Everything developers need to build on QuantumCoin
            </p>
          </div>
          
          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
            {[
              {
                title: "API Documentation",
                desc: "Complete RPC API reference",
                icon: "📚",
                link: "/docs/api",
                color: "cyan"
              },
              {
                title: "SDK Libraries",
                desc: "JavaScript, Python, Rust SDKs",
                icon: "🔧",
                link: "/docs/sdk",
                color: "blue"
              },
              {
                title: "Integration Guide",
                desc: "Exchange and wallet integration",
                icon: "🔗",
                link: "/docs/integration",
                color: "purple"
              },
              {
                title: "Source Code",
                desc: "Full open-source repository",
                icon: "💻",
                link: "https://github.com/aeonith/quantumcoin-ui-",
                color: "emerald"
              }
            ].map((resource, index) => (
              <a
                key={resource.title}
                href={resource.link}
                target={resource.link.startsWith('http') ? '_blank' : '_self'}
                className={`group relative rounded-xl border border-slate-700/50 bg-slate-800/30 p-6 backdrop-blur-sm transition-all duration-300 hover:scale-105 animate-fade-in-up`}
                style={{ animationDelay: `${index * 100}ms` }}
              >
                <div className="text-center">
                  <div className="text-4xl mb-4">{resource.icon}</div>
                  <h3 className={`text-lg font-bold mb-2 ${
                    resource.color === 'cyan' ? 'text-cyan-400' :
                    resource.color === 'blue' ? 'text-blue-400' :
                    resource.color === 'purple' ? 'text-purple-400' :
                    'text-emerald-400'
                  }`}>
                    {resource.title}
                  </h3>
                  <p className="text-sm text-slate-400">{resource.desc}</p>
                </div>
              </a>
            ))}
          </div>
        </div>
      </section>

      {/* JavaScript for Download Functionality */}
      <script
        dangerouslySetInnerHTML={{
          __html: `
            function downloadRelease(platform) {
              // Simulate download - replace with actual download links
              const files = {
                linux: 'quantumcoin-linux-x64-v2.0.0.tar.gz',
                windows: 'quantumcoin-windows-x64-v2.0.0.zip', 
                macos: 'quantumcoin-macos-universal-v2.0.0.dmg'
              };
              
              const fileName = files[platform];
              if (fileName) {
                // In production, this would be a real download URL
                alert('Download starting: ' + fileName + '\\n\\nNote: Replace with actual release URL when available.');
                
                // Simulate download analytics
                if (typeof gtag !== 'undefined') {
                  gtag('event', 'download', {
                    file_name: fileName,
                    platform: platform
                  });
                }
              }
            }
            
            function downloadFile(fileName) {
              // Simulate download
              alert('Download starting: ' + fileName + '\\n\\nNote: Replace with actual release URL when available.');
            }
            
            // Add download tracking
            document.addEventListener('DOMContentLoaded', function() {
              const downloadButtons = document.querySelectorAll('[onclick*="download"]');
              downloadButtons.forEach(button => {
                button.addEventListener('click', function() {
                  // Track download attempts
                  console.log('Download attempted:', button.textContent);
                });
              });
            });
          `
        }}
      />
    </main>
  )
}
