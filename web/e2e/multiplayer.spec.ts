import { type Browser, expect, type Page, test } from "@playwright/test";

type Game = {
  ready: boolean;
  teleport(p?: unknown): void;
  setInput(o: unknown): void;
  net(): { connected: boolean; player: number; remotes: number } | null;
  remoteViews(): Array<{ player: number; position: number[]; trailer?: { kind: number } }>;
};

const g = <T>(page: Page, fn: string) =>
  page.evaluate((src) => {
    const game = (window as unknown as { __game: Game }).__game;
    return new Function("g", `return (${src})(g)`)(game);
  }, fn) as Promise<T>;

async function player(browser: Browser, x: number) {
  // Contextos separados = sessionStorage separado = identidades diferentes.
  const context = await browser.newContext({ viewport: { width: 480, height: 270 } });
  const page = await context.newPage();
  await page.addInitScript(() => localStorage.setItem("graficos", "baixa"));
  await page.goto("/");
  await page.waitForFunction(() => (window as unknown as { __game?: Game }).__game?.ready);
  await g(page, `(g) => g.teleport({ p: [${x}, 1.6, -150], q: [0, 0, 0, 1] })`);
  return { context, page };
}

test("dois navegadores na mesma sala veem o caminhão e o reboque um do outro", async ({
  browser,
}) => {
  const a = await player(browser, 120);
  const b = await player(browser, 100);
  for (const p of [a.page, b.page]) {
    await p.waitForFunction(
      () => {
        const n = (window as unknown as { __game: Game }).__game.net();
        return n?.connected && n.remotes >= 1;
      },
      undefined,
      { timeout: 30_000 },
    );
  }
  // A anda; B precisa ver A se mover (pose interpolada, com o baú engatado).
  const before = await g<Array<{ position: number[] }>>(b.page, "(g) => g.remoteViews()");
  expect(before.length).toBe(1);
  await g(a.page, "(g) => g.setInput({ throttle: 1, brake: 0, steer: 0, handbrake: false })");
  await b.page.waitForFunction(
    (z0) => {
      const v = (window as unknown as { __game: Game }).__game.remoteViews();
      return v.length === 1 && (v[0]?.position[2] ?? 0) > z0 + 5;
    },
    before[0]?.position[2] ?? 0,
    { timeout: 60_000 },
  );
  const seen = await g<Array<{ position: number[]; trailer?: { kind: number } }>>(
    b.page,
    "(g) => g.remoteViews()",
  );
  expect(seen[0]?.trailer?.kind).toBe(0);
  expect(Math.abs((seen[0]?.position[0] ?? 0) - 120)).toBeLessThan(2);
  await b.page.screenshot({ path: "test-results/multiplayer-b.png" });

  // A fecha a aba: some da visão de B.
  await a.context.close();
  await b.page.waitForFunction(
    () => (window as unknown as { __game: Game }).__game.remoteViews().length === 0,
    undefined,
    { timeout: 15_000 },
  );
  await b.context.close();
});
