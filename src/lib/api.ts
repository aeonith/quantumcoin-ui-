// QuantumCoin API Integration Layer
// Handles communication between frontend and Rust backend

const API_BASE = process.env.NEXT_PUBLIC_API_BASE || 'https://quantumcoin-mainnet-api.vercel.app';

export interface WalletBalance {
  address: string;
  balance: number;
}

export interface Transaction {
  id: string;
  sender: string;
  recipient: string;
  amount: number;
  fee: number;
  timestamp: number;
  status: 'pending' | 'confirmed' | 'failed';
}

export interface BlockchainInfo {
  height: number;
  difficulty: string;
  totalSupply: number;
  hashRate: string;
}

// REAL Wallet API Functions - PRODUCTION CRYPTOCURRENCY SYSTEM
export const walletAPI = {
  // Generate a REAL quantum-resistant wallet using backend cryptography
  generateAddress: async (): Promise<{ address: string; publicKey: string; success: boolean }> => {
    if (!API_BASE) {
      throw new Error('Backend required for real wallet generation. This is not a demo.');
    }

    try {
      const response = await fetch(`${API_BASE}/wallet/generate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'same-origin'
      });
      
      if (!response.ok) {
        throw new Error(`Wallet generation failed: ${response.status}`);
      }
      
      const data = await response.json();
      
      if (!data.success || !data.address) {
        throw new Error('Invalid wallet generation response');
      }
      
      return {
        address: data.address,
        publicKey: data.public_key,
        success: true
      };
    } catch (error) {
      console.error('REAL wallet generation error:', error);
      throw new Error('Failed to generate quantum-resistant wallet. Backend connection required.');
    }
  },

  // Get wallet balance from backend
  getBalance: async (address: string): Promise<number> => {
    if (!API_BASE || !address) {
      return parseFloat(localStorage.getItem("qc_wallet_balance") || "0");
    }

    try {
      const response = await fetch(`${API_BASE}/balance/${address}`);
      const data = await response.json();
      
      // Update local storage for caching
      if (data.balance !== undefined) {
        localStorage.setItem("qc_wallet_balance", data.balance.toString());
      }
      
      return data.balance || 0;
    } catch (error) {
      console.error('Error fetching balance:', error);
      return parseFloat(localStorage.getItem("qc_wallet_balance") || "0");
    }
  },

  // Send a transaction
  sendTransaction: async (sender: string, recipient: string, amount: number, fee: number = 0.001): Promise<{ success: boolean; txId?: string; error?: string }> => {
    if (!API_BASE) {
      // Simulate transaction for testing
      return {
        success: true,
        txId: 'sim_' + Date.now(),
        error: undefined
      };
    }

    try {
      const response = await fetch(`${API_BASE}/transaction`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          sender,
          recipient,
          amount: Math.floor(amount * 1000000), // Convert to satoshis
          fee: Math.floor(fee * 1000000)
        })
      });

      const data = await response.json();
      
      if (response.ok) {
        return { success: true, txId: data.txId };
      } else {
        return { success: false, error: data.error || 'Transaction failed' };
      }
    } catch (error) {
      console.error('Error sending transaction:', error);
      return { success: false, error: 'Network error' };
    }
  }
};

// Blockchain API Functions
export const blockchainAPI = {
  // Get blockchain information
  getInfo: async (): Promise<BlockchainInfo | null> => {
    if (!API_BASE) {
      // Return simulated data
      return {
        height: 12547,
        difficulty: "0x1d00ffff",
        totalSupply: 1250000,
        hashRate: "1.2 TH/s"
      };
    }

    try {
      const response = await fetch(`${API_BASE}/blockchain`);
      const data = await response.json();
      return data;
    } catch (error) {
      console.error('Error fetching blockchain info:', error);
      return null;
    }
  },

  // Get transaction history for an address
  getTransactionHistory: async (address: string): Promise<Transaction[]> => {
    if (!API_BASE || !address) {
      return [];
    }

    try {
      const response = await fetch(`${API_BASE}/transactions/${address}`);
      const data = await response.json();
      return data.transactions || [];
    } catch (error) {
      console.error('Error fetching transaction history:', error);
      return [];
    }
  }
};

// RevStop API Functions
export const revStopAPI = {
  // Activate RevStop for a wallet
  activate: async (address: string): Promise<{ success: boolean; error?: string }> => {
    if (!API_BASE) {
      // Simulate activation
      localStorage.setItem(`qc_revstop_${address}`, 'permanent');
      return { success: true };
    }

    try {
      const response = await fetch(`${API_BASE}/revstop/activate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ address })
      });

      const data = await response.json();
      
      if (response.ok) {
        return { success: true };
      } else {
        return { success: false, error: data.error || 'RevStop activation failed' };
      }
    } catch (error) {
      console.error('Error activating RevStop:', error);
      return { success: false, error: 'Network error' };
    }
  },

  // Check RevStop status for a wallet
  getStatus: async (address: string): Promise<{ active: boolean; permanent: boolean }> => {
    if (!API_BASE || !address) {
      const permanent = localStorage.getItem(`qc_revstop_${address}`) === 'permanent';
      const temporary = localStorage.getItem("qc_revstop") === "1";
      return { active: permanent || temporary, permanent };
    }

    try {
      const response = await fetch(`${API_BASE}/revstop/status/${address}`);
      const data = await response.json();
      return { active: data.active || false, permanent: data.permanent || false };
    } catch (error) {
      console.error('Error checking RevStop status:', error);
      return { active: false, permanent: false };
    }
  }
};

// Utility function to check backend connectivity
export const checkBackendHealth = async (): Promise<boolean> => {
  if (!API_BASE) return false;

  try {
    const response = await fetch(`${API_BASE}/`, { timeout: 5000 });
    return response.ok;
  } catch (error) {
    console.error('Backend health check failed:', error);
    return false;
  }
};

// Export a unified API object
export const quantumAPI = {
  wallet: walletAPI,
  blockchain: blockchainAPI,
  revstop: revStopAPI,
  checkHealth: checkBackendHealth
};
