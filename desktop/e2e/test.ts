import { expect, test } from '@playwright/test';

test('Jooble element is visible and clickable', async ({ page }) => {
  await page.goto('/');

  await page.waitForSelector('.w-1/3');

  const joobleElement = page.locator('.w-1/3 div').filter({ hasText: 'Jooble' });

  await expect(joobleElement).toBeVisible();

  await joobleElement.click();

});
