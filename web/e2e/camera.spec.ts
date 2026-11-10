import { expect, test } from "@playwright/test";

test("arrastar o mouse gira a câmera em volta do caminhão", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto("/");
  await page.waitForFunction(
    () => (window as unknown as { __game?: { ready: boolean } }).__game?.ready,
  );
  await page.evaluate(() =>
    (window as unknown as { __game: { teleport(): void } }).__game.teleport(),
  );
  await page.waitForTimeout(1500);

  await page.mouse.move(640, 360);
  await page.mouse.down();
  for (let i = 1; i <= 20; i++) {
    await page.mouse.move(640 + i * 20, 360 + i * 3);
  }
  await page.waitForTimeout(300);
  await page.screenshot({ path: "test-results/camera-drag.png" });
  await page.mouse.up();
  expect(errors).toEqual([]);
});
