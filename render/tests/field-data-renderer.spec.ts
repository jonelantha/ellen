import { test, expect } from '@playwright/test';
import { readdirSync, readFileSync } from 'node:fs';
import { join, basename, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const fixturesDir = join(dirname(fileURLToPath(import.meta.url)), 'fixtures');

const fixtures = readdirSync(fixturesDir)
  .filter(file => file.endsWith('.bin'))
  .map(file => ({
    name: basename(file, '.bin'),
    fieldData: new Uint8Array(readFileSync(join(fixturesDir, file))),
  }));

for (const fixture of fixtures) {
  test.describe.parallel(fixture.name, () => {
    test('renders correctly', async ({ page }) => {
      await page.goto('/');

      await page.evaluate(
        fieldData => window.testRender(fieldData),
        fixture.fieldData,
      );

      await expect(page.locator('canvas')).toHaveScreenshot(
        `${fixture.name}.png`,
        { maxDiffPixels: 100, threshold: 0.2 },
      );
    });
  });
}
