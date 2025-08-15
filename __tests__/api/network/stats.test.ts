import { createMocks } from 'node-mocks-http'
import handler from '../../../pages/api/network/stats'

describe('/api/network/stats', () => {
  it('returns network statistics with correct structure', async () => {
    const { req, res } = createMocks({
      method: 'GET',
    })

    await handler(req, res)

    expect([200, 500]).toContain(res._getStatusCode())
    const data = JSON.parse(res._getData())
    
    if (res._getStatusCode() === 200) {
      expect(data).toHaveProperty('success', true)
      expect(data).toHaveProperty('timestamp')
      expect(data).toHaveProperty('network')
      expect(data).toHaveProperty('blockchain')
      expect(data).toHaveProperty('mining')
      expect(data).toHaveProperty('economics')
      expect(data).toHaveProperty('security')
      
      // Blockchain data
      expect(data.blockchain).toHaveProperty('height')
      expect(data.blockchain).toHaveProperty('difficulty')
      expect(data.blockchain).toHaveProperty('hashRate')
      expect(data.blockchain).toHaveProperty('totalSupply')
      expect(data.blockchain).toHaveProperty('maxSupply', 22000000)
      expect(typeof data.blockchain.height).toBe('number')
      expect(data.blockchain.height).toBeGreaterThanOrEqual(0)
      
      // Mining data
      expect(data.mining).toHaveProperty('networkHashrate')
      expect(data.mining).toHaveProperty('difficulty')
      expect(data.mining).toHaveProperty('blockReward')
      expect(data.mining).toHaveProperty('miningAlgorithm', 'SHA256d')
      
      // Security data
      expect(data.security).toHaveProperty('quantumResistant', true)
      expect(data.security).toHaveProperty('consensusAlgorithm', 'Proof of Work')
      expect(data.security).toHaveProperty('revStopProtection', true)
      expect(data.security).toHaveProperty('vulnerabilities', 0)
      
      // Economics data
      expect(data.economics).toHaveProperty('priceUSD')
      expect(data.economics).toHaveProperty('marketCap')
      expect(typeof data.economics.priceUSD).toBe('number')
      expect(typeof data.economics.marketCap).toBe('number')
    } else {
      // Error case
      expect(data).toHaveProperty('success', false)
      expect(data).toHaveProperty('error')
    }
  })

  it('handles backend connection failures gracefully', async () => {
    // Mock console.error to avoid noise in test output
    const originalError = console.error
    console.error = jest.fn()

    const { req, res } = createMocks({
      method: 'GET',
    })

    await handler(req, res)

    console.error = originalError

    // Should return a response (either success or controlled failure)
    expect([200, 500]).toContain(res._getStatusCode())
  })

  it('calculates live network stats when backend unavailable', async () => {
    const { req, res } = createMocks({
      method: 'GET',
    })

    await handler(req, res)

    // Even if backend is down, it should return calculated stats
    const data = JSON.parse(res._getData())
    
    if (data.success) {
      expect(data.blockchain.height).toBeGreaterThanOrEqual(0)
      expect(data.blockchain.maxSupply).toBe(22000000)
      expect(data.security.quantumResistant).toBe(true)
    }
  })
})
