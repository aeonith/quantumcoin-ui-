import { createMocks } from 'node-mocks-http'
import handler from '../../../pages/api/wallet/generate'

describe('/api/wallet/generate', () => {
  it('generates a new wallet', async () => {
    const { req, res } = createMocks({
      method: 'POST',
    })

    await handler(req, res)

    expect(res._getStatusCode()).toBe(200)
    const data = JSON.parse(res._getData())
    expect(data).toHaveProperty('address')
    expect(data).toHaveProperty('publicKey')
    expect(data).toHaveProperty('privateKey')
    expect(data.address).toMatch(/^qtc[a-zA-Z0-9]+$/)
  })

  it('rejects non-POST requests', async () => {
    const { req, res } = createMocks({
      method: 'GET',
    })

    await handler(req, res)

    expect(res._getStatusCode()).toBe(405)
  })
})
