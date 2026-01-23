import { test, expect } from '@playwright/test';
import { globSync, readFileSync } from 'node:fs';
import { basename } from 'node:path';

const fieldDataFixtures = globSync('field-data-fixtures/*.bin').map(file => ({
  name: basename(file, '.bin'),
  fieldData: new Uint8Array(readFileSync(file)),
}));

for (const { name, fieldData } of fieldDataFixtures) {
  test.describe.parallel(name, () => {
    test('renders correctly', async ({ page }) => {
      await page.goto('/');

      await page.evaluate(fieldData => window.testRender(fieldData), fieldData);

      await expect(page.locator('canvas')).toHaveScreenshot(`${name}.png`, {
        maxDiffPixels: 100,
        threshold: 0.2,
      });
    });
  });
}
