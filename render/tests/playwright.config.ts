import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: '.',
  fullyParallel: true,
  forbidOnly: !!process.env['CI'],
  retries: 0,
  reporter: process.env['CI'] ? 'github' : 'html',

  use: {
    baseURL: 'http://localhost:8080',
    trace: 'on-first-retry',
    launchOptions: {
      args: ['--enable-unsafe-webgpu', '--use-angle=swiftshader'],
    },
  },

  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],

  ...(process.env['SERVE_COMMAND'] && {
    webServer: {
      command: process.env['SERVE_COMMAND'],
      url: 'http://localhost:8080',
      reuseExistingServer: !process.env['CI'],
    },
  }),
});
