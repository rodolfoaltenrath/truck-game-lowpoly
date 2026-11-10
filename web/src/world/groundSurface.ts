import { type Color3, Mesh, type Scene, VertexData } from "@babylonjs/core";

interface Patch {
  x0: number;
  x1: number;
  z0: number;
  z1: number;
  color: Color3;
}

/** Mosaico horizontal: cada ponto pertence a uma única face, sem camadas de profundidade. */
export class GroundSurface {
  private patches: Patch[];

  constructor(x: number, z: number, size: number, color: Color3) {
    this.patches = [{ x0: x, x1: x + size, z0: z, z1: z + size, color }];
  }

  paint(x: number, z: number, width: number, depth: number, color: Color3): void {
    const next: Patch[] = [];
    for (const p of this.patches) {
      const x0 = Math.max(p.x0, x - width / 2);
      const x1 = Math.min(p.x1, x + width / 2);
      const z0 = Math.max(p.z0, z - depth / 2);
      const z1 = Math.min(p.z1, z + depth / 2);
      if (x0 >= x1 || z0 >= z1) {
        next.push(p);
        continue;
      }
      if (p.z0 < z0) next.push({ ...p, z1: z0 });
      if (z1 < p.z1) next.push({ ...p, z0: z1 });
      if (p.x0 < x0) next.push({ ...p, x1: x0, z0, z1 });
      if (x1 < p.x1) next.push({ ...p, x0: x1, z0, z1 });
      next.push({ x0, x1, z0, z1, color });
    }
    this.patches = next;
  }

  mesh(name: string, scene: Scene): Mesh {
    const positions: number[] = [];
    const normals: number[] = [];
    const colors: number[] = [];
    const indices: number[] = [];
    for (const p of this.patches) {
      const base = positions.length / 3;
      positions.push(p.x0, 0, p.z0, p.x0, 0, p.z1, p.x1, 0, p.z1, p.x1, 0, p.z0);
      // Mesma orientação de faces de CreateGround no Babylon.
      indices.push(base, base + 2, base + 1, base, base + 3, base + 2);
      for (let i = 0; i < 4; i++) {
        normals.push(0, 1, 0);
        colors.push(p.color.r, p.color.g, p.color.b, 1);
      }
    }
    const mesh = new Mesh(name, scene);
    const data = new VertexData();
    data.positions = positions;
    data.normals = normals;
    data.colors = colors;
    data.indices = indices;
    data.applyToMesh(mesh);
    return mesh;
  }
}
