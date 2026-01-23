import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: '.',
  fullyParallel: true,
  forbidOnly: !!process.env['CI'],
  retries: 0,
  reporter: 'html',

  use: {
    baseURL: 'http://localhost:8080',
    trace: 'on-first-retry',
  },

  snapshotPathTemplate:
    '{snapshotDir}/{testFileDir}/{testFileName}-snapshots/{arg}{ext}',

  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],

  ...(process.env['SERVE_COMMAND'] && {
    webServer: {
      command: process.env['SERVE_COMMAND'],
      url: 'http://localhost:8080',
      reuseExistingServer: !process.env['CI'],
    },
  }),
});
