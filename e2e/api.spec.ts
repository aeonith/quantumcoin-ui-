import { test, expect } from '@playwright/test';

test.describe('API Endpoints', () => {
  test('wallet generation works', async ({ page }) => {
    const response = await page.request.post('/api/wallet/generate');
    expect(response.ok()).toBeTruthy();
    
    const wallet = await response.json();
    expect(wallet).toHaveProperty('address');
    expect(wallet).toHaveProperty('publicKey');
    expect(wallet.address).toMatch(/^qtc[a-zA-Z0-9]+$/);
  });

  test('mining status endpoint works', async ({ page }) => {
    const response = await page.request.get('/api/mining/status');
    expect(response.ok()).toBeTruthy();
    
    const status = await response.json();
    expect(status).toHaveProperty('isActive');
    expect(status).toHaveProperty('hashRate');
    expect(typeof status.isActive).toBe('boolean');
  });

  test('network stats endpoint works', async ({ page }) => {
    const response = await page.request.get('/api/network/stats');
    expect(response.ok()).toBeTruthy();
    
    const stats = await response.json();
    expect(stats).toHaveProperty('blockHeight');
    expect(stats).toHaveProperty('difficulty');
  });
});
