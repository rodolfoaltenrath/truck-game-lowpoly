import { expect, test } from "@playwright/test";

type Game = {
  ready: boolean;
  teleport(p?: unknown): void;
  setInput(o: unknown): void;
  renderPosition(): number[];
  setInterpolation(on: boolean): void;
  toggleCouple(): string;
  telemetry(): { speedKmh: number };
};

/**
 * Mede a suavidade do desenho: em velocidade quase constante, a velocidade
 * aparente (deslocamento desenhado / tempo do quadro) não pode oscilar muito
 * de um quadro para o outro. Sem interpolação, quadros sem passo de física
 * aparecem como velocidade zero e o desvio fica enorme.
 */
// Janela pequena: o render por software fica rápido o bastante para a física
// não precisar pular passos (o que mascararia a medida).
test.use({ viewport: { width: 480, height: 270 } });

test("movimento desenhado é contínuo entre quadros", async ({ page }) => {
  // Mede a interpolação, não a placa de vídeo: o render por software do
  // Chromium headless fica lento demais com o pós-processamento.
  await page.addInitScript(() => localStorage.setItem("graficos", "baixa"));
  await page.goto("/");
  await page.waitForFunction(() => (window as unknown as { __game?: Game }).__game?.ready);
  await page.evaluate(() => {
    const g = (window as unknown as { __game: Game }).__game;
    // Reta do oval: 400 m livres à frente.
    g.teleport({ p: [120, 1.6, -190], q: [0, 0, 0, 1] });
    // Mede só o desenho: cavalo solto (a junta e as trocas de marcha com
    // carga variam a velocidade de verdade e já têm teste próprio).
    g.toggleCouple();
    g.setInput({ throttle: 1, brake: 0, steer: 0, handbrake: false });
  });
  await page.waitForFunction(
    () => (window as unknown as { __game: Game }).__game.telemetry().speedKmh > 60,
    undefined,
    { timeout: 90_000 },
  );
  await page.evaluate(() => {
    (window as unknown as { __game: Game }).__game.setInput({
      throttle: 0,
      brake: 0,
      steer: 0,
      handbrake: false,
    });
  });

  const measure = () =>
    page.evaluate(
      () =>
        new Promise<{ frames: number; cv: number; meanSpeed: number }>((resolve) => {
          const g = (window as unknown as { __game: Game }).__game;
          const speeds: number[] = [];
          let last = g.renderPosition();
          let lastT = performance.now();
          const tick = (t: number) => {
            const p = g.renderPosition();
            const dt = (t - lastT) / 1000;
            if (dt > 0) {
              speeds.push(
                Math.hypot((p[0] ?? 0) - (last[0] ?? 0), (p[2] ?? 0) - (last[2] ?? 0)) / dt,
              );
            }
            last = p;
            lastT = t;
            if (speeds.length < 60) {
              requestAnimationFrame(tick);
            } else {
              const mean = speeds.reduce((a, b) => a + b, 0) / speeds.length;
              const sd = Math.sqrt(speeds.reduce((a, b) => a + (b - mean) ** 2, 0) / speeds.length);
              resolve({ frames: speeds.length, cv: sd / mean, meanSpeed: mean });
            }
          };
          requestAnimationFrame((t) => {
            lastT = t;
            last = g.renderPosition();
            requestAnimationFrame(tick);
          });
        }),
    );

  // Mesma medida com e sem interpolação, no mesmo trecho: o resultado relativo
  // independe da velocidade do render por software.
  const setInterp = (on: boolean) =>
    page.evaluate((v) => (window as unknown as { __game: Game }).__game.setInterpolation(v), on);
  await setInterp(false);
  const off = await measure();
  await setInterp(true);
  const stats = await measure();
  console.log(JSON.stringify({ off, on: stats }));
  expect(stats.meanSpeed).toBeGreaterThan(2);
  // Coeficiente de variação da velocidade aparente: bem menor com interpolação.
  expect(stats.cv).toBeLessThan(off.cv * 0.8);
});
