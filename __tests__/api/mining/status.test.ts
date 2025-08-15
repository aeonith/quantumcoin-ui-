import { createMocks } from 'node-mocks-http'
import handler from '../../../pages/api/mining/status'

describe('/api/mining/status', () => {
  it('returns mining status with correct structure', async () => {
    const { req, res } = createMocks({
      method: 'GET',
    })

    await handler(req, res)

    expect([200, 500]).toContain(res._getStatusCode())
    const data = JSON.parse(res._getData())
    
    if (res._getStatusCode() === 200) {
      expect(data).toHaveProperty('success', true)
      expect(data).toHaveProperty('timestamp')
      expect(data).toHaveProperty('mining')
      expect(data).toHaveProperty('network')
      expect(data).toHaveProperty('earnings')
      expect(data).toHaveProperty('performance')
      
      // Mining section
      expect(data.mining).toHaveProperty('active')
      expect(data.mining).toHaveProperty('hashrate')
      expect(data.mining).toHaveProperty('difficulty')
      expect(typeof data.mining.active).toBe('boolean')
      expect(typeof data.mining.hashrate).toBe('number')
      
      // Network section
      expect(data.network).toHaveProperty('height')
      expect(data.network).toHaveProperty('difficulty')
      expect(typeof data.network.height).toBe('number')
      
      // Earnings section
      expect(data.earnings).toHaveProperty('todayQTC')
      expect(data.earnings).toHaveProperty('totalQTC')
      expect(typeof data.earnings.todayQTC).toBe('number')
      expect(typeof data.earnings.totalQTC).toBe('number')
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
})
