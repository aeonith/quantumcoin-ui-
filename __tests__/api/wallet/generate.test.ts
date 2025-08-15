import { createMocks } from 'node-mocks-http'
import handler from '../../../pages/api/wallet/generate'

describe('/api/wallet/generate', () => {
  it('generates a new wallet with correct structure', async () => {
    const { req, res } = createMocks({
      method: 'POST',
    })

    await handler(req, res)

    expect(res._getStatusCode()).toBe(200)
    const data = JSON.parse(res._getData())
    
    expect(data).toHaveProperty('success', true)
    expect(data).toHaveProperty('address')
    expect(data).toHaveProperty('publicKey')
    expect(data).toHaveProperty('createdAt')
    expect(data).toHaveProperty('quantumResistant', true)
    expect(data).toHaveProperty('algorithm')
    expect(data).toHaveProperty('network')
    expect(data).toHaveProperty('keyStrength')
    expect(data).toHaveProperty('revStopCapable', true)
    
    expect(data.address).toMatch(/^QTC[a-zA-Z0-9]+$/)
    expect(typeof data.publicKey).toBe('string')
    expect(data.publicKey.length).toBeGreaterThan(0)
  })

  it('rejects non-POST requests', async () => {
    const { req, res } = createMocks({
      method: 'GET',
    })

    await handler(req, res)

    expect(res._getStatusCode()).toBe(405)
    const data = JSON.parse(res._getData())
    expect(data).toHaveProperty('success', false)
    expect(data).toHaveProperty('error')
  })

  it('generates unique addresses', async () => {
    const { req: req1, res: res1 } = createMocks({ method: 'POST' })
    const { req: req2, res: res2 } = createMocks({ method: 'POST' })

    await handler(req1, res1)
    await handler(req2, res2)

    const data1 = JSON.parse(res1._getData())
    const data2 = JSON.parse(res2._getData())

    expect(data1.address).not.toEqual(data2.address)
    expect(data1.publicKey).not.toEqual(data2.publicKey)
  })
})
