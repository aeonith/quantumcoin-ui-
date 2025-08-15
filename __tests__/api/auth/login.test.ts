import { createMocks } from 'node-mocks-http'
import handler from '../../../pages/api/auth/login'

describe('/api/auth/login', () => {
  it('handles valid demo login', async () => {
    const { req, res } = createMocks({
      method: 'POST',
      body: {
        email: 'demo@quantumcoin.com',
        password: 'demo12345'
      }
    })

    await handler(req, res)

    expect(res._getStatusCode()).toBe(200)
    const data = JSON.parse(res._getData())
    expect(data).toHaveProperty('success', true)
    expect(data).toHaveProperty('user')
    expect(data).toHaveProperty('session')
    expect(data.user.email).toBe('demo@quantumcoin.com')
  })

  it('rejects invalid credentials', async () => {
    const { req, res } = createMocks({
      method: 'POST',
      body: {
        email: 'test@example.com',
        password: 'wrongpassword'
      }
    })

    await handler(req, res)

    expect(res._getStatusCode()).toBe(401)
    const data = JSON.parse(res._getData())
    expect(data).toHaveProperty('success', false)
    expect(data).toHaveProperty('error')
  })

  it('validates required fields', async () => {
    const { req, res } = createMocks({
      method: 'POST',
      body: {
        email: 'test@example.com'
        // missing password
      }
    })

    await handler(req, res)

    expect(res._getStatusCode()).toBe(400)
    const data = JSON.parse(res._getData())
    expect(data).toHaveProperty('success', false)
    expect(data).toHaveProperty('error')
  })

  it('validates email format', async () => {
    const { req, res } = createMocks({
      method: 'POST',
      body: {
        email: 'invalid-email',
        password: 'password123'
      }
    })

    await handler(req, res)

    expect(res._getStatusCode()).toBe(400)
  })

  it('rejects non-POST requests', async () => {
    const { req, res } = createMocks({
      method: 'GET',
    })

    await handler(req, res)

    expect(res._getStatusCode()).toBe(405)
    const data = JSON.parse(res._getData())
    expect(data).toHaveProperty('success', false)
  })
})
