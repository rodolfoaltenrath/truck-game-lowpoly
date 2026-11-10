import {
  Color3,
  Matrix,
  type Mesh,
  Quaternion,
  type Scene,
  type ShadowGenerator,
  StandardMaterial,
  Vector3,
} from "@babylonjs/core";
import type RAPIER from "@dimforge/rapier3d-compat";
import { facetedBlob, facetedCylinder, mergeParts, mulberry32, paint } from "../render/geometry";
import { PALETTE } from "../render/palette";

type Rapier = typeof RAPIER;

/**
 * Vegetação e pedras espalhadas. Cada variante é uma malha com "thin
 * instances" (uma chamada de desenho por variante) e colisor estático simples.
 */
export interface SceneryOptions {
  count: number;
  radius: number;
  seed: number;
  /** Devolve false onde não pode haver objeto (estrada, área de testes…). */
  allowed(x: number, z: number): boolean;
  /** Altura do terreno no ponto. */
  height(x: number, z: number): number;
}

export function buildScenery(
  rapier: Rapier,
  world: RAPIER.World,
  scene: Scene,
  shadows: ShadowGenerator,
  o: SceneryOptions,
): void {
  const material = new StandardMaterial("scenery", scene);
  material.diffuseColor = Color3.White();
  material.specularColor = new Color3(0.04, 0.04, 0.04);

  const trees = [
    treeVariant(scene, PALETTE.leafA, 1, 2.3),
    treeVariant(scene, PALETTE.leafB, 2, 2.0),
    treeVariant(scene, PALETTE.leafC, 3, 2.5),
    pineVariant(scene, PALETTE.leafB, 4),
  ];
  const rocks = [rockVariant(scene, 5), rockVariant(scene, 6)];
  for (const m of [...trees, ...rocks]) {
    m.material = material;
    m.receiveShadows = true;
    m.isPickable = false;
  }
  for (const m of trees) {
    shadows.addShadowCaster(m);
  }

  const rand = mulberry32(o.seed);
  const treeMatrices: number[][] = trees.map(() => []);
  const rockMatrices: number[][] = rocks.map(() => []);
  let placed = 0;
  let attempts = 0;
  while (placed < o.count && attempts < o.count * 20) {
    attempts++;
    const angle = rand() * Math.PI * 2;
    const dist = Math.sqrt(rand()) * o.radius;
    const x = Math.cos(angle) * dist;
    const z = Math.sin(angle) * dist;
    if (!o.allowed(x, z)) {
      continue;
    }
    const y = o.height(x, z);
    const isRock = rand() < 0.15;
    const scale = isRock ? 0.5 + rand() * 1.1 : 0.75 + rand() * 0.7;
    const m = Matrix.Compose(
      new Vector3(scale, scale, scale),
      Quaternion.FromEulerAngles(0, rand() * Math.PI * 2, 0),
      new Vector3(x, y - 0.05, z),
    );
    const list = isRock ? rockMatrices : treeMatrices;
    const variant = Math.floor(rand() * list.length);
    const target = list[variant];
    if (!target) {
      continue;
    }
    target.push(...m.asArray());

    const collider = isRock
      ? rapier.ColliderDesc.ball(0.9 * scale).setTranslation(x, y + 0.2 * scale, z)
      : rapier.ColliderDesc.cylinder(1.6 * scale, 0.22 * scale).setTranslation(
          x,
          y + 1.6 * scale,
          z,
        );
    world.createCollider(collider.setFriction(0.8));
    placed++;
  }

  const apply = (meshes: Mesh[], matrices: number[][]) => {
    for (const [i, mesh] of meshes.entries()) {
      const data = matrices[i] ?? [];
      if (data.length === 0) {
        mesh.setEnabled(false);
        continue;
      }
      mesh.thinInstanceSetBuffer("matrix", new Float32Array(data), 16, true);
      mesh.freezeWorldMatrix();
    }
  };
  apply(trees, treeMatrices);
  apply(rocks, rockMatrices);
}

function treeVariant(scene: Scene, leaf: Color3, seed: number, canopy: number): Mesh {
  const trunk = facetedCylinder(
    "trunk",
    { diameterTop: 0.32, diameterBottom: 0.5, height: 2.4, sides: 6 },
    scene,
  );
  trunk.position.y = 1.2;
  const crown = facetedBlob("crown", { radius: canopy, jitter: 0.16, seed, flatten: 0.9 }, scene);
  crown.position.y = 2.2 + canopy * 0.75;
  const side = facetedBlob(
    "crown2",
    { radius: canopy * 0.6, jitter: 0.2, seed: seed + 100 },
    scene,
  );
  side.position.set(canopy * 0.8, 1.9 + canopy * 0.6, canopy * 0.25);
  return mergeParts(`tree-${seed}`, [
    paint(trunk, PALETTE.trunk),
    paint(crown, leaf),
    paint(side, Color3.Lerp(leaf, Color3.White(), 0.08)),
  ]);
}

function pineVariant(scene: Scene, leaf: Color3, seed: number): Mesh {
  const parts: Mesh[] = [];
  const trunk = facetedCylinder(
    "pine-trunk",
    { diameterTop: 0.2, diameterBottom: 0.36, height: 2.4, sides: 6 },
    scene,
  );
  trunk.position.y = 1.2;
  parts.push(paint(trunk, PALETTE.trunk));
  const dark = Color3.Lerp(leaf, Color3.Black(), 0.18);
  for (let i = 0; i < 3; i++) {
    const cone = facetedCylinder(
      "pine-cone",
      { diameterTop: 0, diameterBottom: 3.4 - i * 0.9, height: 2.6, sides: 7 },
      scene,
    );
    cone.position.y = 2.6 + i * 1.5;
    parts.push(paint(cone, i % 2 === 0 ? dark : leaf));
  }
  return mergeParts(`pine-${seed}`, parts);
}

function rockVariant(scene: Scene, seed: number): Mesh {
  const rock = facetedBlob("rock", { radius: 0.9, jitter: 0.3, seed, flatten: 0.6 }, scene);
  return mergeParts(`rock-${seed}`, [paint(rock, PALETTE.rock)]);
}
