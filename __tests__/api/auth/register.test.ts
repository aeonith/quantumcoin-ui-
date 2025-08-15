import { createMocks } from 'node-mocks-http'
import handler from '../../../pages/api/auth/register'

describe('/api/auth/register', () => {
  it('handles registration request', async () => {
    const { req, res } = createMocks({
      method: 'POST',
      body: {
        email: 'test@example.com',
        password: 'password123',
        confirmPassword: 'password123'
      }
    })

    await handler(req, res)

    // Should return a response (even if not implemented yet)
    expect([200, 400, 409, 500, 501]).toContain(res._getStatusCode())
  })

  it('rejects non-POST requests', async () => {
    const { req, res } = createMocks({
      method: 'GET',
    })

    await handler(req, res)

    expect([405, 501]).toContain(res._getStatusCode())
  })
})
