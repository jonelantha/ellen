import { test, expect } from '@playwright/test';
import { glob, readFile } from 'node:fs/promises';
import { basename } from 'node:path';

for await (const fieldDataFixture of glob('field-data-fixtures/*.bin')) {
  const fixtureName = basename(fieldDataFixture, '.bin');

  test.describe.parallel(fixtureName, () => {
    test('renders correctly', async ({ page }) => {
      const fieldData = new Uint8Array(await readFile(fieldDataFixture));

      await page.goto('/');

      await page.evaluate(fieldData => window.testRender(fieldData), fieldData);

      await expect(page.locator('canvas')).toHaveScreenshot(
        `${fixtureName}.png`,
        { maxDiffPixels: 100, threshold: 0.2 },
      );
    });
  });
}
