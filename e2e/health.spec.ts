import { test, expect } from '@playwright/test';

test.describe('System Health', () => {
  test('health endpoint returns valid response', async ({ page }) => {
    const response = await page.request.get('/api/health');
    expect(response.ok()).toBeTruthy();
    
    const health = await response.json();
    expect(health).toHaveProperty('status');
    expect(health).toHaveProperty('services');
    expect(health).toHaveProperty('features');
  });

  test('homepage loads successfully', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/QuantumCoin/);
    
    // Check for critical UI elements
    await expect(page.locator('text=Build:')).toBeVisible({ timeout: 10000 });
  });
});
