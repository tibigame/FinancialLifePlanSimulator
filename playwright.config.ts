import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests/ui",
  fullyParallel: true,
  use: {
    browserName: "chromium",
    channel: "msedge",
    headless: true,
    baseURL: "http://127.0.0.1:1420",
    viewport: { width: 1280, height: 800 },
  },
  webServer: {
    command: "node node_modules/vite/bin/vite.js --host 127.0.0.1",
    url: "http://127.0.0.1:1420",
    reuseExistingServer: false,
  },
});
