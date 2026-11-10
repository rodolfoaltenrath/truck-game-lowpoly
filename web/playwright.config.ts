import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "e2e",
  timeout: 120_000,
  // Render por software divide a CPU; em paralelo as medidas de suavidade ficam ruidosas.
  workers: 1,
  use: {
    baseURL: "http://localhost:5173",
    viewport: { width: 1280, height: 720 },
    launchOptions: { args: ["--use-gl=angle", "--use-angle=swiftshader", "--enable-unsafe-swiftshader"] },
  },
  webServer: [
    {
      command: "pnpm dev",
      url: "http://localhost:5173",
      reuseExistingServer: true,
    },
    {
      // Sala do multiplayer (servidor Rust).
      command: "cargo run --release -p truck_server -- --bind 127.0.0.1:39100",
      port: 39100,
      reuseExistingServer: true,
      timeout: 300_000,
    },
  ],
});
