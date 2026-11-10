import { expect, type Page, test } from "@playwright/test";

interface Telemetry {
  speedKmh: number;
  gear: number;
  rpm: number;
  wheelsInContact: number;
  position: { x: number; y: number; z: number };
  headingDeg: number;
  wheelSpin: number;
  steerAngle: number;
}

const telemetry = (page: Page) =>
  page.evaluate(() =>
    (window as unknown as { __game: { telemetry(): Telemetry } }).__game.telemetry(),
  );

const setInput = (page: Page, o: Record<string, unknown> | null) =>
  page.evaluate((v) => {
    (window as unknown as { __game: { setInput(o: unknown): void } }).__game.setInput(v);
  }, o);

/** Espera até a condição valer ou estourar o prazo; devolve a última telemetria. */
async function until(page: Page, cond: (t: Telemetry) => boolean, ms: number): Promise<Telemetry> {
  const end = Date.now() + ms;
  let t = await telemetry(page);
  while (!cond(t) && Date.now() < end) {
    await page.waitForTimeout(100);
    t = await telemetry(page);
  }
  return t;
}

test("caminhão assenta, acelera, esterça, freia e dá ré", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  page.on("console", (m) => {
    if (m.type() === "error") errors.push(m.text());
  });

  await page.goto("/");
  await page.waitForFunction(
    () => (window as unknown as { __game?: { ready: boolean } }).__game?.ready,
  );
  await page.evaluate(() =>
    (window as unknown as { __game: { teleport(): void } }).__game.teleport(),
  );

  // Assenta nas seis rodas.
  const rest = await until(
    page,
    (t) => t.wheelsInContact === 6 && Math.abs(t.speedKmh) < 0.5,
    15_000,
  );
  expect(rest.wheelsInContact).toBe(6);
  expect(rest.position.y).toBeGreaterThan(0.5);
  expect(rest.position.y).toBeLessThan(2.5);
  const start = rest.position;

  // Acelera para frente (+Z).
  await setInput(page, { throttle: 1, brake: 0, steer: 0, handbrake: false });
  const fast = await until(page, (t) => t.speedKmh > 30, 40_000);
  expect(fast.speedKmh).toBeGreaterThan(30);
  expect(fast.position.z).toBeGreaterThan(start.z + 5);
  expect(fast.gear).toBeGreaterThan(1);
  // Roda gira para frente (rotação positiva em torno de +X, como no visual).
  expect(fast.wheelSpin).toBeGreaterThan(rest.wheelSpin + 5);

  // Esterça para a direita: com +Z à frente, direita é -X, logo o rumo diminui.
  await setInput(page, { throttle: 0.4, brake: 0, steer: 1, handbrake: false });
  const turned = await until(page, (t) => t.headingDeg < -20, 20_000);
  expect(turned.headingDeg).toBeLessThan(-20);
  expect(turned.wheelsInContact).toBeGreaterThanOrEqual(4);
  expect(turned.steerAngle).toBeLessThan(0);

  // Freia até parar.
  await setInput(page, { throttle: 0, brake: 1, steer: 0, handbrake: false });
  const stopped = await until(page, (t) => Math.abs(t.speedKmh) < 1, 30_000);
  expect(Math.abs(stopped.speedKmh)).toBeLessThan(1);

  // Continua segurando o freio parado: engata ré e anda para trás.
  const reversing = await until(page, (t) => t.speedKmh < -3, 20_000);
  expect(reversing.gear).toBe(-1);
  expect(reversing.speedKmh).toBeLessThan(-3);

  await setInput(page, null);
  await page.screenshot({ path: "test-results/drive.png" });
  expect(errors).toEqual([]);
});
