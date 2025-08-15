module.exports = {
  extends: [
    'next/core-web-vitals',
    'eslint:recommended',
    '@typescript-eslint/recommended',
  ],
  parser: '@typescript-eslint/parser',
  plugins: ['@typescript-eslint'],
  rules: {
    // STRICT: No warnings allowed
    '@typescript-eslint/no-unused-vars': 'error',
    '@typescript-eslint/no-explicit-any': 'error',
    '@typescript-eslint/no-non-null-assertion': 'error',
    'no-console': 'warn',
    'prefer-const': 'error',
    'no-var': 'error',
    
    // Next.js specific
    '@next/next/no-img-element': 'error',
    '@next/next/no-html-link-for-pages': 'error',
    
    // React specific
    'react-hooks/exhaustive-deps': 'error',
    'react/jsx-key': 'error',
    'react/no-unescaped-entities': 'error',
  },
  settings: {
    'import/resolver': {
      typescript: {},
    },
  },
  env: {
    browser: true,
    node: true,
    es2021: true,
    jest: true,
  },
  ignorePatterns: [
    'node_modules',
    '.next',
    'out',
    'build',
    '*.config.js',
  ],
}
