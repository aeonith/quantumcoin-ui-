import '@testing-library/jest-dom'

// Mock Next.js router
jest.mock('next/router', () => ({
  useRouter: () => ({
    push: jest.fn(),
    replace: jest.fn(),
    prefetch: jest.fn(),
    back: jest.fn(),
    pathname: '/',
    query: {},
    asPath: '/',
  }),
}))

// Mock environment variables
process.env.NEXT_PUBLIC_AI_URL = 'http://localhost:8000'
process.env.NEXT_PUBLIC_BUILD_ID = 'test-build'

// Global test timeout
jest.setTimeout(10000)
