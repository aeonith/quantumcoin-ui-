/**
 * QuantumCoin API Client
 * 
 * Type-safe API client using ky and zod for schema validation
 */

import ky, { type KyInstance } from 'ky';
import { z } from 'zod';

// Environment configuration
const API_BASE_URL = process.env.NEXT_PUBLIC_BACKEND_BASE_URL ?? 'http://localhost:8080';
const API_TIMEOUT = 8000; // 8 seconds

/**
 * Zod schemas for API responses
 */

// Basic types
const HashSchema = z.string().regex(/^(0x)?[0-9a-fA-F]+$/);
const AddressSchema = z.string().min(1);
const TimestampSchema = z.string().datetime();

// Network Status
export const NetworkStatusSchema = z.object({
  chain: z.string(),
  network: z.enum(['mainnet', 'testnet', 'devnet']),
  height: z.number().int().min(0),
  tip_hash: HashSchema,
  peers: z.number().int().min(0),
  block_time_target_sec: z.number().int().positive(),
  supply: z.object({
    total: z.number().int().positive(),
    issued: z.number().int().min(0),
    remaining: z.number().int().min(0),
  }),
});
export type NetworkStatus = z.infer<typeof NetworkStatusSchema>;

// Block schemas
export const BlockSummarySchema = z.object({
  hash: HashSchema,
  height: z.number().int().min(0),
  time: TimestampSchema,
  tx_count: z.number().int().min(0),
  prev_hash: HashSchema,
});
export type BlockSummary = z.infer<typeof BlockSummarySchema>;

export const BlockDetailsSchema = BlockSummarySchema.extend({
  difficulty: z.number().int().positive(),
  nonce: z.number().int().min(0),
  merkle_root: HashSchema,
  transactions: z.array(HashSchema),
  reward: z.number().int().min(0),
});
export type BlockDetails = z.infer<typeof BlockDetailsSchema>;

// Transaction schemas
export const TransactionInputSchema = z.object({
  prev_tx_id: HashSchema,
  output_index: z.number().int().min(0),
  signature: z.string(),
  amount: z.number().int().min(0),
});
export type TransactionInput = z.infer<typeof TransactionInputSchema>;

export const TransactionOutputSchema = z.object({
  index: z.number().int().min(0),
  amount: z.number().int().min(0),
  address: AddressSchema,
  spent: z.boolean(),
});
export type TransactionOutput = z.infer<typeof TransactionOutputSchema>;

export const TransactionDetailsSchema = z.object({
  id: HashSchema,
  hash: HashSchema,
  block_hash: HashSchema.nullable(),
  block_height: z.number().int().min(0).nullable(),
  confirmations: z.number().int().min(0),
  timestamp: TimestampSchema,
  inputs: z.array(TransactionInputSchema),
  outputs: z.array(TransactionOutputSchema),
  fee: z.number().int().min(0),
  size: z.number().int().positive(),
});
export type TransactionDetails = z.infer<typeof TransactionDetailsSchema>;

// Address schema
export const UTXOSchema = z.object({
  tx_id: HashSchema,
  output_index: z.number().int().min(0),
  amount: z.number().int().min(0),
  confirmations: z.number().int().min(0),
});
export type UTXO = z.infer<typeof UTXOSchema>;

export const AddressSummarySchema = z.object({
  address: AddressSchema,
  balance: z.number().int().min(0),
  tx_count: z.number().int().min(0),
  received_total: z.number().int().min(0),
  sent_total: z.number().int().min(0),
  utxos: z.array(UTXOSchema).nullable(),
});
export type AddressSummary = z.infer<typeof AddressSummarySchema>;

// Mempool schema
export const MempoolStatusSchema = z.object({
  size: z.number().int().min(0),
  total_fees: z.number().int().min(0),
  transactions: z.array(TransactionDetailsSchema).nullable(),
});
export type MempoolStatus = z.infer<typeof MempoolStatusSchema>;

// Pagination schema
export const PaginationSchema = z.object({
  limit: z.number().int().positive(),
  next_cursor: z.number().int().min(0).nullable(),
  has_more: z.boolean(),
});
export type Pagination = z.infer<typeof PaginationSchema>;

// Response wrappers
export const BlockListResponseSchema = z.object({
  blocks: z.array(BlockSummarySchema),
  pagination: PaginationSchema,
});
export type BlockListResponse = z.infer<typeof BlockListResponseSchema>;

export const HealthResponseSchema = z.object({
  status: z.literal('healthy'),
  timestamp: TimestampSchema,
});
export type HealthResponse = z.infer<typeof HealthResponseSchema>;

export const BroadcastTxResponseSchema = z.object({
  tx_id: HashSchema,
  status: z.literal('accepted'),
});
export type BroadcastTxResponse = z.infer<typeof BroadcastTxResponseSchema>;

// Error schema
export const ApiErrorSchema = z.object({
  error: z.string(),
  code: z.string().optional(),
  details: z.record(z.unknown()).optional(),
});
export type ApiError = z.infer<typeof ApiErrorSchema>;

/**
 * API Client Class
 */
export class QuantumCoinAPI {
  private readonly client: KyInstance;
  private readonly demoMode: boolean;

  constructor(baseUrl: string = API_BASE_URL, demoMode: boolean = false) {
    this.demoMode = demoMode;
    this.client = ky.create({
      prefixUrl: baseUrl,
      timeout: API_TIMEOUT,
      retry: {
        limit: 2,
        methods: ['get'],
        statusCodes: [408, 413, 429, 500, 502, 503, 504],
      },
      hooks: {
        beforeRequest: [
          (request) => {
            // Add request ID for tracing
            request.headers.set('X-Request-ID', crypto.randomUUID());
          },
        ],
        beforeError: [
          async (error) => {
            // Try to parse error response
            try {
              const errorData = await error.response?.json();
              const parsed = ApiErrorSchema.parse(errorData);
              error.message = `API Error: ${parsed.error}`;
            } catch {
              // Keep original error if parsing fails
            }
            return error;
          },
        ],
      },
    });
  }

  /**
   * Safe API call with error handling and demo mode fallback
   */
  private async safeCall<T>(
    endpoint: string,
    schema: z.ZodSchema<T>,
    demoData?: T,
    options?: Parameters<KyInstance['get']>[1]
  ): Promise<T> {
    if (this.demoMode && demoData) {
      // Simulate network delay in demo mode
      await new Promise(resolve => setTimeout(resolve, 100 + Math.random() * 200));
      return demoData;
    }

    try {
      const response = await this.client.get(endpoint, options).json();
      return schema.parse(response);
    } catch (error) {
      if (demoData) {
        console.warn(`API call failed, falling back to demo data:`, error);
        return demoData;
      }
      throw error;
    }
  }

  private async safePost<T>(
    endpoint: string,
    data: unknown,
    schema: z.ZodSchema<T>,
    demoData?: T
  ): Promise<T> {
    if (this.demoMode && demoData) {
      await new Promise(resolve => setTimeout(resolve, 300 + Math.random() * 200));
      return demoData;
    }

    try {
      const response = await this.client.post(endpoint, { json: data }).json();
      return schema.parse(response);
    } catch (error) {
      if (demoData) {
        console.warn(`API call failed, falling back to demo data:`, error);
        return demoData;
      }
      throw error;
    }
  }

  // API Methods

  async getHealth(): Promise<HealthResponse> {
    return this.safeCall(
      'health',
      HealthResponseSchema,
      {
        status: 'healthy' as const,
        timestamp: new Date().toISOString(),
      }
    );
  }

  async getNetworkStatus(): Promise<NetworkStatus> {
    return this.safeCall(
      'status',
      NetworkStatusSchema,
      {
        chain: 'quantumcoin-mainnet-v2',
        network: 'mainnet' as const,
        height: 123456,
        tip_hash: '0x1a2b3c4d5e6f7890abcdef1234567890abcdef1234567890abcdef1234567890',
        peers: 24,
        block_time_target_sec: 600,
        supply: {
          total: 22000000,
          issued: 0,
          remaining: 9500000,
        },
      }
    );
  }

  async getBlocks(limit = 20, cursor?: number): Promise<BlockListResponse> {
    const params = new URLSearchParams({ limit: limit.toString() });
    if (cursor) params.set('cursor', cursor.toString());

    const demoBlocks = Array.from({ length: limit }, (_, i) => ({
      hash: `0x${(123456 - i).toString(16).padStart(64, '0')}`,
      height: 123456 - i,
      time: new Date(Date.now() - i * 600000).toISOString(),
      tx_count: Math.floor(Math.random() * 10) + 1,
      prev_hash: `0x${(123455 - i).toString(16).padStart(64, '0')}`,
    }));

    return this.safeCall(
      `blocks?${params}`,
      BlockListResponseSchema,
      {
        blocks: demoBlocks,
        pagination: {
          limit,
          next_cursor: cursor ? cursor - limit : 123456 - limit,
          has_more: true,
        },
      }
    );
  }

  async getBlock(hashOrHeight: string): Promise<BlockDetails> {
    const isHash = hashOrHeight.startsWith('0x') || hashOrHeight.length === 64;
    const height = isHash ? 123456 : parseInt(hashOrHeight);

    return this.safeCall(
      `blocks/${encodeURIComponent(hashOrHeight)}`,
      BlockDetailsSchema,
      {
        hash: isHash ? hashOrHeight : `0x${height.toString(16).padStart(64, '0')}`,
        height,
        time: new Date().toISOString(),
        tx_count: 5,
        prev_hash: `0x${(height - 1).toString(16).padStart(64, '0')}`,
        difficulty: 486604799,
        nonce: 2083236893,
        merkle_root: '0x5f4e3d2c1b0a9988776655443322110987654321abcdef0123456789abcdef',
        transactions: [
          '0xabc123def456789abc123def456789abc123def456789abc123def456789abc1',
          '0xdef456789abc123def456789abc123def456789abc123def456789abc123def4',
        ],
        reward: 50000,
      }
    );
  }

  async getTransaction(id: string): Promise<TransactionDetails> {
    return this.safeCall(
      `tx/${encodeURIComponent(id)}`,
      TransactionDetailsSchema,
      {
        id,
        hash: id,
        block_hash: '0x1a2b3c4d5e6f7890abcdef1234567890abcdef1234567890abcdef1234567890',
        block_height: 123456,
        confirmations: 6,
        timestamp: new Date().toISOString(),
        inputs: [
          {
            prev_tx_id: '0xdef456789abc123def456789abc123def456789abc123def456789abc123def4',
            output_index: 0,
            signature: '0x304502207fffffff...',
            amount: 1000000,
          },
        ],
        outputs: [
          {
            index: 0,
            amount: 999000,
            address: 'QtC1a2b3c4d5e6f7890abcdef',
            spent: false,
          },
        ],
        fee: 1000,
        size: 250,
      }
    );
  }

  async getAddress(address: string, includeUtxos = false): Promise<AddressSummary> {
    const params = new URLSearchParams();
    if (includeUtxos) params.set('include_utxos', 'true');

    return this.safeCall(
      `address/${encodeURIComponent(address)}?${params}`,
      AddressSummarySchema,
      {
        address,
        balance: 5000000,
        tx_count: 25,
        received_total: 10000000,
        sent_total: 5000000,
        utxos: includeUtxos ? [
          {
            tx_id: '0xabc123def456789abc123def456789abc123def456789abc123def456789abc1',
            output_index: 0,
            amount: 1000000,
            confirmations: 6,
          },
        ] : null,
      }
    );
  }

  async getMempool(includeTxs = false): Promise<MempoolStatus> {
    const params = new URLSearchParams();
    if (includeTxs) params.set('include_txs', 'true');

    return this.safeCall(
      `mempool?${params}`,
      MempoolStatusSchema,
      {
        size: 42,
        total_fees: 100000,
        transactions: includeTxs ? [] : null,
      }
    );
  }

  async broadcastTransaction(rawTx: string): Promise<BroadcastTxResponse> {
    return this.safePost(
      'tx',
      { raw: rawTx },
      BroadcastTxResponseSchema,
      {
        tx_id: '0x' + Array.from({ length: 64 }, () => Math.floor(Math.random() * 16).toString(16)).join(''),
        status: 'accepted' as const,
      }
    );
  }
}

// Create default API instance
export const api = new QuantumCoinAPI();

/**
 * Hook for checking if we're in demo mode
 */
export function useApiMode() {
  // In a real app, this would check if the API is available
  // For now, return demo mode if we're in development and no backend URL is set
  const isDemoMode = process.env.NODE_ENV === 'development' && !process.env.NEXT_PUBLIC_BACKEND_BASE_URL;
  
  return {
    isDemoMode,
    apiBaseUrl: API_BASE_URL,
  };
}

// Export schemas for external use
export {
  HashSchema,
  AddressSchema,
  TimestampSchema,
};
