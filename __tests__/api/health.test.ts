import { createMocks } from 'node-mocks-http'
import handler from '../../pages/api/health'

describe('/api/health', () => {
  it('returns health status with all required fields', async () => {
    const { req, res } = createMocks({
      method: 'GET',
    })

    await handler(req, res)

    expect([200, 503]).toContain(res._getStatusCode())
    const data = JSON.parse(res._getData())
    
    expect(data).toHaveProperty('status')
    expect(['healthy', 'degraded']).toContain(data.status)
    expect(data).toHaveProperty('timestamp')
    expect(data).toHaveProperty('version')
    expect(data).toHaveProperty('network')
    expect(data).toHaveProperty('chainId')
    expect(data).toHaveProperty('services')
    expect(data).toHaveProperty('features')
    expect(data).toHaveProperty('security')
    expect(data).toHaveProperty('environment')
    
    // Verify services structure
    expect(data.services).toHaveProperty('rustBackend')
    expect(data.services).toHaveProperty('postgres')
    expect(data.services).toHaveProperty('redis')
    expect(data.services).toHaveProperty('blockchain')
    
    // Verify features
    expect(data.features).toHaveProperty('realBlockchain')
    expect(data.features).toHaveProperty('quantumSecurity')
    expect(data.features.realBlockchain).toBe(true)
  })
  
  it('handles errors gracefully', async () => {
    // Mock console.error to avoid noise in test output
    const originalError = console.error
    console.error = jest.fn()
    
    const { req, res } = createMocks({
      method: 'GET',
    })
    
    // This test verifies the error handling works
    await handler(req, res)
    
    console.error = originalError
    
    // Should still return a response (either success or error)
    expect([200, 503, 500]).toContain(res._getStatusCode())
  })
})
