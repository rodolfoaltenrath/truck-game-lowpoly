import {
  Color3,
  Matrix,
  Mesh,
  Quaternion,
  type Scene,
  StandardMaterial,
  Vector3,
  VertexData,
} from "@babylonjs/core";
import { PALETTE } from "../render/palette";

/**
 * Tufos de capim só perto da câmera: uma malha pequena (três lâminas) com
 * thin instances numa grade fixa no mundo. Cada célula sorteia (por hash da
 * posição, sempre igual) se tem tufo, onde e de que tamanho; ao andar, só a
 * janela em volta da câmera é recalculada.
 */
export interface GrassOptions {
  radius: number;
  cell: number;
  /** 0..1: fração das células com tufo. */
  density: number;
  allowed(x: number, z: number): boolean;
  height(x: number, z: number): number;
}

export class GrassField {
  private readonly mesh: Mesh;
  private readonly center = new Vector3(Number.NaN, 0, Number.NaN);
  private readonly buffer: Float32Array;
  private readonly tmp = new Matrix();

  constructor(
    scene: Scene,
    private readonly o: GrassOptions,
  ) {
    this.mesh = tuftMesh(scene);
    const side = Math.ceil((o.radius * 2) / o.cell) + 1;
    this.buffer = new Float32Array(side * side * 16);
    this.mesh.thinInstanceSetBuffer("matrix", this.buffer, 16, false);
    this.mesh.thinInstanceCount = 0;
  }

  /** Recalcula a janela quando a câmera se afasta do último centro. */
  update(camera: Vector3): void {
    const step = this.o.cell * 4;
    if (Math.abs(camera.x - this.center.x) < step && Math.abs(camera.z - this.center.z) < step) {
      return;
    }
    this.center.set(camera.x, 0, camera.z);
    const { cell, radius, density } = this.o;
    const x0 = Math.floor((camera.x - radius) / cell);
    const z0 = Math.floor((camera.z - radius) / cell);
    const n = Math.ceil((radius * 2) / cell);
    let count = 0;
    const scale = new Vector3();
    const pos = new Vector3();
    const rot = new Quaternion();
    for (let i = 0; i <= n; i++) {
      for (let j = 0; j <= n; j++) {
        const cx = x0 + i;
        const cz = z0 + j;
        const h = hash(cx, cz);
        if (h > density) {
          continue;
        }
        const x = (cx + hash(cz, cx + 7)) * cell;
        const z = (cz + hash(cx + 3, cz - 5)) * cell;
        const dx = x - camera.x;
        const dz = z - camera.z;
        if (dx * dx + dz * dz > radius * radius || !this.o.allowed(x, z)) {
          continue;
        }
        const s = 0.9 + hash(cx - 11, cz + 13) * 0.9;
        scale.set(s, s * (0.8 + hash(cz, cx) * 0.6), s);
        Quaternion.FromEulerAnglesToRef(0, h * 40, 0, rot);
        pos.set(x, this.o.height(x, z), z);
        Matrix.ComposeToRef(scale, rot, pos, this.tmp);
        this.tmp.copyToArray(this.buffer, count * 16);
        count++;
      }
    }
    this.mesh.thinInstanceCount = count;
    this.mesh.thinInstanceBufferUpdated("matrix");
  }
}

function tuftMesh(scene: Scene): Mesh {
  const positions: number[] = [];
  const colors: number[] = [];
  const indices: number[] = [];
  const base = PALETTE.grassC.scale(0.82);
  const tip = Color3.Lerp(PALETTE.grassD, Color3.White(), 0.15);
  const blades = 5;
  for (let b = 0; b < blades; b++) {
    const a = (b / blades) * Math.PI + b * 0.4;
    const lean = 0.12 + (b % 2) * 0.08;
    const height = 0.35 + (b % 3) * 0.12;
    const w = 0.07;
    const cx = Math.cos(a);
    const cz = Math.sin(a);
    const i = positions.length / 3;
    positions.push(-cx * w, 0, -cz * w, cx * w, 0, cz * w, -cz * lean, height, cx * lean);
    colors.push(base.r, base.g, base.b, 1, base.r, base.g, base.b, 1, tip.r, tip.g, tip.b, 1);
    indices.push(i, i + 1, i + 2);
  }
  const normals: number[] = [];
  for (let k = 0; k < positions.length / 3; k++) {
    normals.push(0, 1, 0);
  }
  const mesh = new Mesh("grass-tufts", scene);
  const data = new VertexData();
  data.positions = positions;
  data.colors = colors;
  data.indices = indices;
  data.normals = normals;
  data.applyToMesh(mesh);
  const mat = new StandardMaterial("grass-tufts", scene);
  mat.diffuseColor = Color3.White();
  mat.specularColor = Color3.Black();
  mat.backFaceCulling = false;
  mesh.material = mat;
  mesh.isPickable = false;
  mesh.alwaysSelectAsActiveMesh = true;
  return mesh;
}

/** Hash determinístico 0..1 de duas coordenadas inteiras. */
function hash(a: number, b: number): number {
  let h = Math.imul(a | 0, 374761393) ^ Math.imul(b | 0, 668265263);
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}
