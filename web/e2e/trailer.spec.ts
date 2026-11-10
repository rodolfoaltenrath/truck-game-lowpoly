import { expect, type Page, test } from "@playwright/test";

interface Telemetry {
  speedKmh: number;
  headingDeg: number;
  position: { x: number; y: number; z: number };
}
interface Hitch {
  coupled: boolean;
  label: string | null;
  canCouple: boolean;
  nearestDistance: number | null;
  jointGap: number | null;
  trailerHeadingDeg: number | null;
}
type Game = {
  ready: boolean;
  teleport(p?: unknown): void;
  setInput(o: unknown): void;
  telemetry(): Telemetry;
  hitch(): Hitch;
  toggleCouple(): string;
};

const game = <T>(page: Page, fn: (g: Game) => T) =>
  page.evaluate((src) => {
    const g = (window as unknown as { __game: Game }).__game;
    return new Function("g", `return (${src})(g)`)(g);
  }, fn.toString()) as Promise<T>;

async function until(page: Page, cond: (t: Telemetry, h: Hitch) => boolean, ms: number) {
  const end = Date.now() + ms;
  for (;;) {
    const [t, h] = await game(page, (g) => [g.telemetry(), g.hitch()] as const);
    if (cond(t, h) || Date.now() > end) {
      return { t, h };
    }
    await page.waitForTimeout(100);
  }
}

const drive = (page: Page, o: Record<string, unknown>) =>
  page.evaluate((v) => (window as unknown as { __game: Game }).__game.setInput(v), {
    throttle: 0,
    brake: 0,
    steer: 0,
    handbrake: false,
    ...o,
  });

test("reboque segue o cavalo, desengata e engata de novo de ré", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.addInitScript(() => localStorage.setItem("graficos", "baixa"));
  await page.goto("/");
  await page.waitForFunction(() => (window as unknown as { __game?: Game }).__game?.ready);
  // Reta do oval, longe de obstáculos.
  await game(page, (g) => g.teleport({ p: [120, 1.6, -150], q: [0, 0, 0, 1] }));

  let s = await until(page, (_t, h) => h.coupled && (h.jointGap ?? 9) < 0.15, 10_000);
  expect(s.h.coupled).toBe(true);
  expect(s.h.label).toBe("Baú");

  // Acelera e faz uma curva: o reboque acompanha com atraso e a junta não abre.
  await drive(page, { throttle: 1 });
  s = await until(page, (t) => t.speedKmh > 20, 40_000);
  expect(s.t.speedKmh).toBeGreaterThan(20);
  await drive(page, { throttle: 0.5, steer: -1 });
  s = await until(page, (t) => t.headingDeg > 30, 30_000);
  expect(s.t.headingDeg).toBeGreaterThan(30);
  expect(s.h.trailerHeadingDeg ?? 0).toBeLessThan(s.t.headingDeg);
  expect(s.h.trailerHeadingDeg ?? 0).toBeGreaterThan(0);
  expect(s.h.jointGap ?? 9).toBeLessThan(0.2);

  // Para. Solta o freio logo ao parar: segurar o freio parado engata a ré.
  await drive(page, { brake: 1 });
  s = await until(page, (t) => t.speedKmh < 1, 30_000);
  await drive(page, {});
  await page.waitForTimeout(500);

  // Desengata e anda para frente: o reboque fica.
  expect(await game(page, (g) => g.toggleCouple())).toContain("desengatado");
  await page.waitForTimeout(300); // status é atualizado a cada quadro
  const parkedAt = await game(page, (g) => g.hitch());
  expect(parkedAt.coupled).toBe(false);
  const before = (await game(page, (g) => g.telemetry())).position;
  await drive(page, { throttle: 0.6 });
  s = await until(
    page,
    (t) => Math.hypot(t.position.x - before.x, t.position.z - before.z) > 10,
    30_000,
  );
  await drive(page, { brake: 1 });
  s = await until(page, (t) => t.speedKmh < 0.5, 20_000);
  await drive(page, {});
  await page.waitForTimeout(500);
  expect((await game(page, (g) => g.hitch())).nearestDistance ?? 0).toBeGreaterThan(8);

  // Ré em linha reta: ao encostar, a quinta-roda trava sozinha no pino-rei.
  // (Em ré, o freio é o acelerador: segurar freio parado engata a ré.)
  await drive(page, { brake: 0.6 });
  s = await until(page, (_t, h) => h.coupled, 60_000);
  await drive(page, {});
  expect(s.t.speedKmh).toBeGreaterThan(-9.5);
  expect(s.h.coupled).toBe(true);
  s = await until(page, (_t, h) => (h.jointGap ?? 9) < 0.15, 10_000);
  expect(s.h.coupled).toBe(true);
  expect(s.h.jointGap ?? 9).toBeLessThan(0.15);

  await page.screenshot({ path: "test-results/trailer.png" });
  expect(errors).toEqual([]);
});
