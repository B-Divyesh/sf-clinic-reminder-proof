import { expect, test } from '@playwright/test';

test('planning scaffold has an accessible document shell and no console errors', async ({ page }) => {
  const consoleErrors: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') consoleErrors.push(message.text());
  });

  await page.goto('/');

  await expect(page).toHaveTitle('Reminder Proof — Product skeleton');
  await expect(page.locator('main')).toHaveCount(1);
  await expect(page.getByRole('heading', { level: 1 })).toHaveText(
    'The product skeleton is ready.'
  );
  await expect(page.getByRole('link', { name: 'Skip to main content' })).toBeAttached();
  expect(consoleErrors).toEqual([]);
});
