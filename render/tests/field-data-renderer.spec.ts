import { test, expect } from '@playwright/test';
import { readdirSync, readFileSync } from 'node:fs';
import { join, basename, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const fixturesDir = join(__dirname, 'fixtures');
const files = readdirSync(fixturesDir);
const binFiles = files.filter(file => file.endsWith('.bin'));

const fixtures = binFiles.map(file => {
  const buffer = readFileSync(join(fixturesDir, file));
  const name = basename(file, '.bin');
  return { name, buffer };
});

for (const fixture of fixtures) {
  test.describe.parallel(fixture.name, () => {
    test('renders correctly', async ({ page }) => {
      await page.goto('/');

      await page.evaluate(
        fieldData => window.testRender(fieldData),
        new Uint8Array(fixture.buffer),
      );

      await page.waitForTimeout(500);

      await expect(page.locator('canvas')).toHaveScreenshot(
        `${fixture.name}.png`,
        { maxDiffPixels: 100, threshold: 0.2 },
      );
    });
  });
}
