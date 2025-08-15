import { createMocks } from 'node-mocks-http'
import handler from '../../pages/api/btc-price'

// Mock fetch globally
global.fetch = jest.fn()

describe('/api/btc-price', () => {
  beforeEach(() => {
    jest.clearAllMocks()
  })

  it('returns BTC price and QTC price when API succeeds', async () => {
    const mockResponse = {
      bitcoin: { usd: 48000 }
    }
    
    ;(global.fetch as jest.Mock).mockResolvedValueOnce({
      json: () => Promise.resolve(mockResponse)
    })

    const { req, res } = createMocks({
      method: 'GET',
    })

    await handler(req, res)

    expect(res._getStatusCode()).toBe(200)
    const data = JSON.parse(res._getData())
    
    expect(data).toHaveProperty('usd', 48000)
    expect(data).toHaveProperty('qtcUsd', 20) // 48000 / 2400
    expect(data).toHaveProperty('timestamp')
    expect(typeof data.timestamp).toBe('string')
  })

  it('handles API failure gracefully', async () => {
    ;(global.fetch as jest.Mock).mockRejectedValueOnce(new Error('API Error'))

    const { req, res } = createMocks({
      method: 'GET',
    })

    await handler(req, res)

    expect(res._getStatusCode()).toBe(500)
    const data = JSON.parse(res._getData())
    
    expect(data).toHaveProperty('usd', null)
    expect(data).toHaveProperty('qtcUsd', 0.025)
    expect(data).toHaveProperty('error', 'Price fetch failed')
    expect(data).toHaveProperty('timestamp')
  })

  it('handles malformed API response', async () => {
    ;(global.fetch as jest.Mock).mockResolvedValueOnce({
      json: () => Promise.resolve({})
    })

    const { req, res } = createMocks({
      method: 'GET',
    })

    await handler(req, res)

    expect(res._getStatusCode()).toBe(200)
    const data = JSON.parse(res._getData())
    
    expect(data).toHaveProperty('usd', null)
    expect(data).toHaveProperty('qtcUsd', 0.025)
    expect(data).toHaveProperty('timestamp')
  })
})
