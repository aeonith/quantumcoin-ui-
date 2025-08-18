'use client';

import React, { useState, useEffect } from 'react';

interface ExplorerStats {
  height: number;
  totalSupply: number;
  difficulty: string;
  hashRate: string;
  peers: number;
  mempool: number;
  lastBlockTime: number;
}

interface Block {
  hash: string;
  height: number;
  timestamp: number;
  transactions: number;
  size: number;
}

const ExplorerStats: React.FC = () => {
  const [stats, setStats] = useState<ExplorerStats | null>(null);
  const [recentBlocks, setRecentBlocks] = useState<Block[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const API_BASE = process.env.NEXT_PUBLIC_API_BASE || 'https://quantumcoin-mainnet-api.vercel.app';

  useEffect(() => {
    const fetchExplorerData = async () => {
      try {
        setLoading(true);
        setError(null);

        // Fetch network stats
        const statsResponse = await fetch(`${API_BASE}/explorer/stats`);
        if (!statsResponse.ok) {
          throw new Error(`Stats API failed: ${statsResponse.status}`);
        }
        const statsData = await statsResponse.json();
        setStats(statsData);

        // Fetch recent blocks
        const blocksResponse = await fetch(`${API_BASE}/explorer/blocks?limit=5`);
        if (!blocksResponse.ok) {
          throw new Error(`Blocks API failed: ${blocksResponse.status}`);
        }
        const blocksData = await blocksResponse.json();
        setRecentBlocks(blocksData.blocks || []);

      } catch (err) {
        console.error('Explorer data fetch error:', err);
        setError(err instanceof Error ? err.message : 'Unknown error');
        
        // Fallback to demo data if backend unavailable
        setStats({
          height: 0,
          totalSupply: 0,
          difficulty: 'N/A',
          hashRate: 'N/A',
          peers: 0,
          mempool: 0,
          lastBlockTime: Date.now() / 1000
        });
        setRecentBlocks([]);
      } finally {
        setLoading(false);
      }
    };

    fetchExplorerData();
    
    // Refresh every 30 seconds
    const interval = setInterval(fetchExplorerData, 30000);
    return () => clearInterval(interval);
  }, [API_BASE]);

  if (loading) {
    return (
      <div className="explorer-stats loading">
        <div className="stats-grid">
          <div className="stat-card">
            <h3>Block Height</h3>
            <div className="loading-placeholder">Loading...</div>
          </div>
          <div className="stat-card">
            <h3>Network Hash Rate</h3>
            <div className="loading-placeholder">Loading...</div>
          </div>
          <div className="stat-card">
            <h3>Connected Peers</h3>
            <div className="loading-placeholder">Loading...</div>
          </div>
          <div className="stat-card">
            <h3>Mempool Size</h3>
            <div className="loading-placeholder">Loading...</div>
          </div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="explorer-error">
        <h3>⚠️ Explorer Backend Connection Error</h3>
        <p>Cannot connect to QuantumCoin mainnet API: {error}</p>
        <p>Expected endpoint: <code>{API_BASE}/explorer/stats</code></p>
        <button onClick={() => window.location.reload()}>Retry Connection</button>
      </div>
    );
  }

  return (
    <div className="explorer-stats">
      <div className="stats-grid">
        <div className="stat-card">
          <h3>Block Height</h3>
          <div className="stat-value">
            {stats?.height?.toLocaleString() || '0'}
          </div>
          <div className="stat-subtitle">Latest Block</div>
        </div>
        
        <div className="stat-card">
          <h3>Network Hash Rate</h3>
          <div className="stat-value">
            {stats?.hashRate || 'N/A'}
          </div>
          <div className="stat-subtitle">Mining Power</div>
        </div>
        
        <div className="stat-card">
          <h3>Connected Peers</h3>
          <div className="stat-value">
            {stats?.peers || '0'}
          </div>
          <div className="stat-subtitle">Network Nodes</div>
        </div>
        
        <div className="stat-card">
          <h3>Mempool Size</h3>
          <div className="stat-value">
            {stats?.mempool || '0'}
          </div>
          <div className="stat-subtitle">Pending Transactions</div>
        </div>
      </div>

      <div className="recent-blocks">
        <h3>Recent Blocks</h3>
        {recentBlocks.length > 0 ? (
          <div className="blocks-list">
            {recentBlocks.map((block) => (
              <div key={block.hash} className="block-item">
                <div className="block-height">#{block.height}</div>
                <div className="block-hash">{block.hash.substring(0, 16)}...</div>
                <div className="block-time">
                  {new Date(block.timestamp * 1000).toLocaleTimeString()}
                </div>
                <div className="block-txs">{block.transactions} txs</div>
              </div>
            ))}
          </div>
        ) : (
          <div className="no-blocks">
            <p>No recent blocks available</p>
            <p>Mainnet may be starting up or backend connection needed</p>
          </div>
        )}
      </div>

      <style jsx>{`
        .explorer-stats {
          padding: 2rem;
          background: linear-gradient(135deg, #1e1e2e 0%, #2d1b69 100%);
          border-radius: 12px;
          color: white;
        }
        
        .stats-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 1.5rem;
          margin-bottom: 2rem;
        }
        
        .stat-card {
          background: rgba(255, 255, 255, 0.1);
          border-radius: 8px;
          padding: 1.5rem;
          text-align: center;
          border: 1px solid rgba(255, 255, 255, 0.2);
        }
        
        .stat-card h3 {
          margin: 0 0 1rem 0;
          font-size: 0.9rem;
          color: #a0a0ff;
          text-transform: uppercase;
          letter-spacing: 1px;
        }
        
        .stat-value {
          font-size: 2rem;
          font-weight: bold;
          margin-bottom: 0.5rem;
          color: #00ff88;
        }
        
        .stat-subtitle {
          font-size: 0.8rem;
          color: rgba(255, 255, 255, 0.7);
        }
        
        .loading-placeholder {
          background: linear-gradient(90deg, rgba(255,255,255,0.1) 25%, rgba(255,255,255,0.2) 50%, rgba(255,255,255,0.1) 75%);
          background-size: 200% 100%;
          animation: loading 1.5s infinite;
          border-radius: 4px;
          height: 2rem;
          margin: 0.5rem 0;
        }
        
        @keyframes loading {
          0% { background-position: 200% 0; }
          100% { background-position: -200% 0; }
        }
        
        .recent-blocks h3 {
          margin-bottom: 1rem;
          color: #a0a0ff;
        }
        
        .blocks-list {
          display: flex;
          flex-direction: column;
          gap: 0.5rem;
        }
        
        .block-item {
          display: grid;
          grid-template-columns: auto 1fr auto auto;
          gap: 1rem;
          padding: 0.75rem;
          background: rgba(255, 255, 255, 0.05);
          border-radius: 6px;
          font-family: 'Courier New', monospace;
          font-size: 0.9rem;
        }
        
        .block-height {
          color: #00ff88;
          font-weight: bold;
        }
        
        .block-hash {
          color: #a0a0ff;
        }
        
        .block-time {
          color: rgba(255, 255, 255, 0.8);
        }
        
        .block-txs {
          color: #ffaa00;
        }
        
        .no-blocks {
          text-align: center;
          padding: 2rem;
          color: rgba(255, 255, 255, 0.6);
        }
        
        .explorer-error {
          padding: 2rem;
          background: #ff4444;
          border-radius: 8px;
          color: white;
          text-align: center;
        }
        
        .explorer-error button {
          margin-top: 1rem;
          padding: 0.5rem 1rem;
          background: rgba(255, 255, 255, 0.2);
          border: none;
          border-radius: 4px;
          color: white;
          cursor: pointer;
        }
        
        .explorer-error code {
          background: rgba(0, 0, 0, 0.3);
          padding: 0.2rem 0.4rem;
          border-radius: 3px;
          font-family: monospace;
        }
      `}</style>
    </div>
  );
};

export default ExplorerStats;
