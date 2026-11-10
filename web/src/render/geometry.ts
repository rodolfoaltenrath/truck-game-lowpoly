import {
  type Color3,
  Mesh,
  MeshBuilder,
  type Scene,
  Vector3,
  VertexBuffer,
  VertexData,
} from "@babylonjs/core";

/**
 * Peças básicas do estilo low poly: faces planas (flat shading), quinas
 * chanfradas e cor por vértice. Tudo é gerado por código.
 */

export interface ChamferBoxOptions {
  width: number;
  height: number;
  depth: number;
  /** Tamanho do chanfro nas 12 arestas (m). */
  chamfer: number;
}

/** Caixa com todas as arestas chanfradas: 6 faces, 12 chanfros e 8 cantos. */
export function chamferBox(name: string, o: ChamferBoxOptions, scene: Scene): Mesh {
  const a = o.width / 2;
  const b = o.height / 2;
  const c = o.depth / 2;
  const r = Math.max(0.001, Math.min(o.chamfer, a * 0.95, b * 0.95, c * 0.95));
  const S = [-1, 1] as const;

  const corner = (sx: number, sy: number, sz: number) => ({
    x: new Vector3(sx * a, sy * (b - r), sz * (c - r)),
    y: new Vector3(sx * (a - r), sy * b, sz * (c - r)),
    z: new Vector3(sx * (a - r), sy * (b - r), sz * c),
  });

  const faces: Vector3[][] = [];
  for (const s of S) {
    faces.push([corner(s, -1, -1).x, corner(s, 1, -1).x, corner(s, 1, 1).x, corner(s, -1, 1).x]);
    faces.push([corner(-1, s, -1).y, corner(1, s, -1).y, corner(1, s, 1).y, corner(-1, s, 1).y]);
    faces.push([corner(-1, -1, s).z, corner(1, -1, s).z, corner(1, 1, s).z, corner(-1, 1, s).z]);
  }
  for (const s1 of S) {
    for (const s2 of S) {
      faces.push([
        corner(s1, s2, -1).x,
        corner(s1, s2, 1).x,
        corner(s1, s2, 1).y,
        corner(s1, s2, -1).y,
      ]);
      faces.push([
        corner(s1, -1, s2).x,
        corner(s1, 1, s2).x,
        corner(s1, 1, s2).z,
        corner(s1, -1, s2).z,
      ]);
      faces.push([
        corner(-1, s1, s2).y,
        corner(1, s1, s2).y,
        corner(1, s1, s2).z,
        corner(-1, s1, s2).z,
      ]);
    }
  }
  for (const sx of S) {
    for (const sy of S) {
      for (const sz of S) {
        const k = corner(sx, sy, sz);
        faces.push([k.x, k.y, k.z]);
      }
    }
  }
  return convexFacesMesh(name, faces, scene);
}

/**
 * Monta uma malha de faces planas a partir de polígonos convexos de um sólido
 * convexo centrado na origem. A orientação de cada face é corrigida para fora.
 */
function convexFacesMesh(name: string, faces: Vector3[][], scene: Scene): Mesh {
  const positions: number[] = [];
  const normals: number[] = [];
  const indices: number[] = [];
  for (const face of faces) {
    const [p0, p1, p2] = face;
    if (!p0 || !p1 || !p2) {
      continue;
    }
    let n = Vector3.Cross(p1.subtract(p0), p2.subtract(p0)).normalize();
    const centroid = face
      .reduce((acc, p) => acc.addInPlace(p), new Vector3())
      .scaleInPlace(1 / face.length);
    let poly = face;
    if (Vector3.Dot(n, centroid) < 0) {
      poly = [...face].reverse();
      n = n.negate();
    }
    const base = positions.length / 3;
    for (const p of poly) {
      positions.push(p.x, p.y, p.z);
      normals.push(n.x, n.y, n.z);
    }
    for (let i = 1; i < poly.length - 1; i++) {
      // Babylon espera ordem horária para a face da frente.
      indices.push(base, base + i + 1, base + i);
    }
  }
  const mesh = new Mesh(name, scene);
  const data = new VertexData();
  data.positions = positions;
  data.normals = normals;
  data.indices = indices;
  data.applyToMesh(mesh);
  return mesh;
}

/** Cilindro de poucos lados com faces planas, eixo em Y. */
export function facetedCylinder(
  name: string,
  o: {
    diameter?: number;
    diameterTop?: number;
    diameterBottom?: number;
    height: number;
    sides: number;
  },
  scene: Scene,
): Mesh {
  const mesh = MeshBuilder.CreateCylinder(
    name,
    {
      diameter: o.diameter,
      diameterTop: o.diameterTop ?? o.diameter,
      diameterBottom: o.diameterBottom ?? o.diameter,
      height: o.height,
      tessellation: o.sides,
    },
    scene,
  );
  mesh.convertToFlatShadedMesh();
  return mesh;
}

/** Bolha irregular facetada (copa de árvore, pedra, nuvem). */
export function facetedBlob(
  name: string,
  o: { radius: number; jitter: number; seed: number; flatten?: number },
  scene: Scene,
): Mesh {
  const mesh = MeshBuilder.CreateIcoSphere(
    name,
    { radius: o.radius, subdivisions: 1, updatable: true },
    scene,
  );
  const pos = mesh.getVerticesData(VertexBuffer.PositionKind);
  if (pos) {
    const rand = mulberry32(o.seed);
    // Vértices repetidos na mesma posição precisam do mesmo deslocamento.
    const offsets = new Map<string, number>();
    for (let i = 0; i < pos.length; i += 3) {
      const key = `${(pos[i] ?? 0).toFixed(3)},${(pos[i + 1] ?? 0).toFixed(3)},${(pos[i + 2] ?? 0).toFixed(3)}`;
      let k = offsets.get(key);
      if (k === undefined) {
        k = 1 + (rand() * 2 - 1) * o.jitter;
        offsets.set(key, k);
      }
      pos[i] = (pos[i] ?? 0) * k;
      pos[i + 1] = (pos[i + 1] ?? 0) * k * (o.flatten ?? 1);
      pos[i + 2] = (pos[i + 2] ?? 0) * k;
    }
    mesh.updateVerticesData(VertexBuffer.PositionKind, pos);
  }
  mesh.convertToFlatShadedMesh();
  return mesh;
}

/** Pinta todos os vértices de uma malha com uma cor. */
export function paint(mesh: Mesh, color: Color3): Mesh {
  const count = mesh.getTotalVertices();
  const colors = new Float32Array(count * 4);
  for (let i = 0; i < count; i++) {
    colors[i * 4] = color.r;
    colors[i * 4 + 1] = color.g;
    colors[i * 4 + 2] = color.b;
    colors[i * 4 + 3] = 1;
  }
  mesh.setVerticesData(VertexBuffer.ColorKind, colors);
  return mesh;
}

/** Cor por face com leve variação aleatória (aspecto facetado vivo). */
export function paintFacesVaried(
  mesh: Mesh,
  colors: readonly Color3[],
  seed: number,
  pick?: (faceCenter: Vector3) => Color3,
): void {
  const pos = mesh.getVerticesData(VertexBuffer.PositionKind);
  if (!pos) {
    return;
  }
  const rand = mulberry32(seed);
  const out = new Float32Array((pos.length / 3) * 4);
  // Malha já em flat shading: cada 3 vértices consecutivos são um triângulo.
  for (let v = 0; v < pos.length / 3; v += 3) {
    const center = new Vector3(
      ((pos[v * 3] ?? 0) + (pos[v * 3 + 3] ?? 0) + (pos[v * 3 + 6] ?? 0)) / 3,
      ((pos[v * 3 + 1] ?? 0) + (pos[v * 3 + 4] ?? 0) + (pos[v * 3 + 7] ?? 0)) / 3,
      ((pos[v * 3 + 2] ?? 0) + (pos[v * 3 + 5] ?? 0) + (pos[v * 3 + 8] ?? 0)) / 3,
    );
    const base = pick ? pick(center) : (colors[Math.floor(rand() * colors.length)] ?? colors[0]);
    if (!base) {
      continue;
    }
    const shade = 0.975 + rand() * 0.05;
    for (let k = 0; k < 3; k++) {
      const o = (v + k) * 4;
      out[o] = base.r * shade;
      out[o + 1] = base.g * shade;
      out[o + 2] = base.b * shade;
      out[o + 3] = 1;
    }
  }
  mesh.setVerticesData(VertexBuffer.ColorKind, out);
}

/**
 * Funde peças numa malha só. Com `vertexColors`, todas precisam ter cor por
 * vértice (use `paint`) e o resultado usa um único material.
 */
export function mergeParts(name: string, parts: Mesh[]): Mesh {
  // Low poly não usa textura: sem UV, todas as peças ficam com os mesmos atributos.
  for (const p of parts) {
    for (const kind of [VertexBuffer.UVKind, VertexBuffer.UV2Kind, VertexBuffer.TangentKind]) {
      if (p.isVerticesDataPresent(kind)) {
        p.removeVerticesData(kind);
      }
    }
  }
  const merged = Mesh.MergeMeshes(parts, true, true, undefined, false, true);
  if (!merged) {
    throw new Error(`falha ao fundir malhas de ${name}`);
  }
  merged.name = name;
  return merged;
}

/** Gerador pseudoaleatório determinístico (mesma semente, mesmo cenário). */
export function mulberry32(seed: number): () => number {
  let s = seed >>> 0;
  return () => {
    s = (s + 0x6d2b79f5) >>> 0;
    let t = s;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
