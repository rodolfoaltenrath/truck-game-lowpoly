import { expect, test } from "@playwright/test";

type Game = { ready: boolean; teleport(p?: unknown): void };

/** Capturas de referência do visual (conferência manual em test-results/). */
test("capturas do visual low poly", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  page.on("console", (m) => {
    if (m.type() === "error") errors.push(m.text());
  });
  await page.goto("/");
  await page.waitForFunction(() => (window as unknown as { __game?: Game }).__game?.ready);
  // Perto da estrada, olhando para o oval com árvores ao fundo.
  await page.evaluate(() => {
    const half = -Math.PI / 4;
    (window as unknown as { __game: Game }).__game.teleport({
      p: [100, 1.6, -150],
      q: [0, Math.sin(half), 0, Math.cos(half)],
    });
  });
  await page.keyboard.press("KeyH");
  await page.waitForTimeout(2500);
  await page.screenshot({ path: "test-results/visual-chase.png" });

  // Órbita para ver a frente do caminhão.
  await page.mouse.move(640, 360);
  await page.mouse.down();
  for (let i = 1; i <= 30; i++) {
    await page.mouse.move(640 + i * 17, 360 + i * 1);
  }
  await page.waitForTimeout(400);
  await page.screenshot({ path: "test-results/visual-front.png" });
  await page.mouse.up();
  expect(errors).toEqual([]);
});
