import { createMocks } from 'node-mocks-http'
import handler from '../../../pages/api/network/stats'

describe('/api/network/stats', () => {
  it('returns network statistics', async () => {
    const { req, res } = createMocks({
      method: 'GET',
    })

    await handler(req, res)

    expect(res._getStatusCode()).toBe(200)
    const data = JSON.parse(res._getData())
    expect(data).toHaveProperty('blockHeight')
    expect(data).toHaveProperty('totalTransactions')
    expect(data).toHaveProperty('networkHashRate')
    expect(data).toHaveProperty('connectedPeers')
    expect(typeof data.blockHeight).toBe('number')
    expect(typeof data.totalTransactions).toBe('number')
    expect(typeof data.networkHashRate).toBe('number')
    expect(typeof data.connectedPeers).toBe('number')
  })
})
