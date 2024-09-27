import { expect, test } from '@playwright/test';

test('Initial page load shows startup message', async ({ page }) => {
  await page.goto('/');

  await page.waitForSelector('p.text-lg.font-semibold.text-yellow-400');

  const startupMessage = page.locator('p.text-lg.font-semibold.text-yellow-400');

  await expect(startupMessage).toBeVisible();

  await expect(startupMessage).toHaveText('Starting up. Please wait...');
});


