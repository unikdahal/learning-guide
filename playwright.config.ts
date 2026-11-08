import {defineConfig} from '@playwright/test';
export default defineConfig({
  testDir:'tests/browser',fullyParallel:true,workers:2,timeout:45000,reporter:'list',
  use:{baseURL:'http://127.0.0.1:4322/learning-guide/',headless:true,viewport:{width:1280,height:800},
    launchOptions:process.env.CHROMIUM_PATH ? {executablePath:process.env.CHROMIUM_PATH} : {}},
  webServer:{command:'node scripts/qa/serve.mjs',url:'http://127.0.0.1:4322/learning-guide/',reuseExistingServer:!process.env.CI},
});
