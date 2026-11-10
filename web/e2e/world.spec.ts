import { expect, test } from "@playwright/test";

interface Game {
  ready: boolean;
  teleport(pose: { p: number[]; q: number[] }): void;
  setInput(input: Record<string, number | boolean> | null): void;
  telemetry(): { position: { z: number; y: number }; wheelsInContact: number };
  world(): { regions: string[]; colliders: number };
  meshCount(): number;
}

test("regiões: atravessa a borda, libera recursos e preserva reboques estacionados", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto("/?world=road&solo");
  await page.waitForFunction(() => (window as unknown as { __game?: Game }).__game?.ready);
  await page.evaluate(() => {
    const g = (window as unknown as { __game: Game }).__game;
    g.teleport({ p: [125, 1.6, -4002], q: [0, 0, 0, 1] });
    g.setInput({ throttle: 1, handbrake: false });
  });
  await expect
    .poll(
      () =>
        page.evaluate(() => (window as unknown as { __game: Game }).__game.telemetry().position.z),
      { timeout: 40_000 },
    )
    .toBeGreaterThan(-3980);
  const crossed = await page.evaluate(() => {
    const g = (window as unknown as { __game: Game }).__game;
    g.setInput({ throttle: 0, handbrake: true });
    return g.telemetry();
  });
  expect(crossed.position.y).toBeGreaterThan(0.5);
  expect(crossed.wheelsInContact).toBe(6);

  const residency = await page.evaluate(() => {
    const g = (window as unknown as { __game: Game }).__game;
    const move = (z: number) => g.teleport({ p: [125, 1.6, z], q: [0, 0, 0, 1] });
    move(5500);
    const before = { ...g.world(), meshes: g.meshCount() };
    for (let i = 0; i < 12; i++) move(i % 2 === 0 ? -4500 : 5500);
    const after = { ...g.world(), meshes: g.meshCount() };
    return { before, after };
  });
  expect(residency.after).toEqual(residency.before);
  expect(residency.after.regions.length).toBeLessThanOrEqual(18);
  expect(residency.after.regions).toContain("0,5");
  expect(residency.after.regions).toContain("0,-5");
  expect(residency.after.regions).not.toContain("0,0");
  expect(errors).toEqual([]);
});
