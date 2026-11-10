import { MeshBuilder, NullEngine, Scene, VertexBuffer } from "@babylonjs/core";
import { expect, test } from "@playwright/test";
import { PALETTE } from "../src/render/palette";
import { GroundSurface } from "../src/world/groundSurface";

test("pintura do terreno cobre a região uma vez, com faces visíveis pelo alto", () => {
  const engine = new NullEngine();
  try {
    const scene = new Scene(engine);
    scene.useRightHandedSystem = true;
    const surface = new GroundSurface(-50, -50, 100, PALETTE.grassA);
    surface.paint(0, 0, 16, 200, PALETTE.asphalt);
    surface.paint(20, -10, 40, 30, PALETTE.asphalt);
    surface.paint(7, 0, 0.25, 100, PALETTE.roadLine);
    surface.paint(0, 0, 0.25, 6, PALETTE.paintYellow);
    const mesh = surface.mesh("test", scene);
    const positions = mesh.getVerticesData(VertexBuffer.PositionKind) ?? [];
    const indices = mesh.getIndices() ?? [];
    const bounds: Array<{ x0: number; x1: number; z0: number; z1: number }> = [];
    for (let i = 0; i < positions.length; i += 12) {
      const points = positions.slice(i, i + 12);
      expect([points[1], points[4], points[7], points[10]]).toEqual([0, 0, 0, 0]);
      const xs = [points[0] ?? 0, points[3] ?? 0, points[6] ?? 0, points[9] ?? 0];
      const zs = [points[2] ?? 0, points[5] ?? 0, points[8] ?? 0, points[11] ?? 0];
      bounds.push({
        x0: Math.min(...xs),
        x1: Math.max(...xs),
        z0: Math.min(...zs),
        z1: Math.max(...zs),
      });
    }
    expect(bounds.reduce((area, p) => area + (p.x1 - p.x0) * (p.z1 - p.z0), 0)).toBe(10000);
    for (const [i, a] of bounds.entries()) {
      for (const b of bounds.slice(i + 1)) {
        const overlapX = Math.min(a.x1, b.x1) - Math.max(a.x0, b.x0);
        const overlapZ = Math.min(a.z1, b.z1) - Math.max(a.z0, b.z0);
        expect(overlapX > 0 && overlapZ > 0).toBe(false);
      }
    }
    const reference = MeshBuilder.CreateGround("reference", { width: 1, height: 1 }, scene);
    const winding = (p: number[], ids: ArrayLike<number>, offset: number) => {
      const a = (ids[offset] ?? 0) * 3;
      const b = (ids[offset + 1] ?? 0) * 3;
      const c = (ids[offset + 2] ?? 0) * 3;
      return Math.sign(
        ((p[b] ?? 0) - (p[a] ?? 0)) * ((p[c + 2] ?? 0) - (p[a + 2] ?? 0)) -
          ((p[b + 2] ?? 0) - (p[a + 2] ?? 0)) * ((p[c] ?? 0) - (p[a] ?? 0)),
      );
    };
    const expected = winding(
      reference.getVerticesData(VertexBuffer.PositionKind) ?? [],
      reference.getIndices() ?? [],
      0,
    );
    for (let i = 0; i < indices.length; i += 3)
      expect(winding(positions, indices, i)).toBe(expected);
  } finally {
    engine.dispose();
  }
});
