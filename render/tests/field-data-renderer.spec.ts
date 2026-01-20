import { test, expect } from '@playwright/test';
import fixtures from './fixtures/field-data-cases.js';

for (const fixture of fixtures) {
  test.describe.parallel(fixture.name, () => {
    test('renders correctly', async ({ page }) => {
      await page.goto('/');

      await page.evaluate(
        fieldData => window.testRender(fieldData),
        new Uint8Array(fixture.field_data),
      );

      await expect(page.locator('canvas')).toHaveScreenshot(
        `${fixture.name}.png`,
        { maxDiffPixels: 100, threshold: 0.2 },
      );
    });
  });
}
