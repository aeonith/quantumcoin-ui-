import { createMocks } from 'node-mocks-http'
import handler from '../../../pages/api/mining/status'

describe('/api/mining/status', () => {
  it('returns mining status', async () => {
    const { req, res } = createMocks({
      method: 'GET',
    })

    await handler(req, res)

    expect(res._getStatusCode()).toBe(200)
    const data = JSON.parse(res._getData())
    expect(data).toHaveProperty('isActive')
    expect(data).toHaveProperty('hashRate')
    expect(data).toHaveProperty('difficulty')
    expect(typeof data.isActive).toBe('boolean')
    expect(typeof data.hashRate).toBe('number')
  })
})
