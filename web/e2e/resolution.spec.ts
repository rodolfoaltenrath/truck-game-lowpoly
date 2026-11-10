import { expect, test } from "@playwright/test";

type Game = {
  ready: boolean;
  graphics(): { scale: number; width: number; height: number; economy: number; quality: string };
};

test("resolução permanece nativa durante quadros lentos e mudança de preset", async ({ page }) => {
  await page.addInitScript(() => localStorage.setItem("graficos", "baixa"));
  await page.goto("/?world=road&solo");
  await page.waitForFunction(() => (window as unknown as { __game?: Game }).__game?.ready);
  await page.evaluate(async () => {
    for (let i = 0; i < 20; i++) {
      await new Promise(requestAnimationFrame);
      const start = performance.now();
      while (performance.now() - start < 50) {
        // Simula CPU ocupada para exercitar a redução de efeitos sem reduzir os pixels.
      }
    }
  });
  expect(
    await page.evaluate(() => (window as unknown as { __game: Game }).__game.graphics().economy),
  ).toBeGreaterThan(0);
  for (const quality of ["média", "alta", "baixa"]) {
    const dimensions = await page.evaluate(() => {
      const canvas = document.querySelector("canvas") as HTMLCanvasElement;
      return {
        ...(window as unknown as { __game: Game }).__game.graphics(),
        cssWidth: canvas.clientWidth,
        cssHeight: canvas.clientHeight,
      };
    });
    expect(dimensions.scale).toBe(1);
    expect(dimensions.width).toBe(dimensions.cssWidth);
    expect(dimensions.height).toBe(dimensions.cssHeight);
    await page.keyboard.press("KeyG");
    await expect
      .poll(() =>
        page.evaluate(() => (window as unknown as { __game: Game }).__game.graphics().quality),
      )
      .toBe(quality);
  }
  await page.setViewportSize({ width: 1024, height: 640 });
  await expect
    .poll(() =>
      page.evaluate(() => (window as unknown as { __game: Game }).__game.graphics().width),
    )
    .toBe(1024);
});
