import {
  Color3,
  Mesh,
  MeshBuilder,
  type Scene,
  StandardMaterial,
  VertexBuffer,
} from "@babylonjs/core";
import type RAPIER from "@dimforge/rapier3d-compat";
import init, { world_regions } from "../../wasm/pkg/game_core_wasm.js";
import { chamferBox, paint } from "../render/geometry";
import { PALETTE } from "../render/palette";
import type { TestTrack } from "./testTrack";

interface Region {
  x: number;
  z: number;
  road: boolean;
  town: string | null;
}

interface LoadedRegion {
  meshes: Mesh[];
  colliders: RAPIER.Collider[];
}

/** Primeiro streaming de geometria local; o formato de regiões vem do Rust/WASM. */
export async function buildRegionWorld(
  rapier: typeof RAPIER,
  world: RAPIER.World,
  scene: Scene,
): Promise<
  TestTrack & { update(x: number, z: number, anchors?: Float64Array): void; status(): unknown }
> {
  await init();
  const loaded = new Map<string, LoadedRegion>();
  const material = new StandardMaterial("regions-vc", scene);
  material.diffuseColor = Color3.White();
  material.specularColor = Color3.Black();
  let previous = "";
  let currentTown: string | null = null;

  const load = (r: Region): LoadedRegion => {
    const parts: Mesh[] = [];
    const colliders: RAPIER.Collider[] = [];
    const ox = r.x * 1000;
    const oz = r.z * 1000;
    const ground = MeshBuilder.CreateGround(
      `region-${r.x}-${r.z}`,
      { width: 1000, height: 1000 },
      scene,
    );
    ground.position.set(ox + 500, 0, oz + 500);
    parts.push(paint(ground, PALETTE.grassA));
    colliders.push(
      world.createCollider(
        rapier.ColliderDesc.cuboid(500, 0.5, 500)
          .setTranslation(ox + 500, -0.5, oz + 500)
          .setFriction(0.9),
      ),
    );
    const surface = (
      x: number,
      z: number,
      width: number,
      depth: number,
      color: Color3,
      y = 0.02,
    ) => {
      const mesh = MeshBuilder.CreateGround("region-surface", { width, height: depth }, scene);
      mesh.position.set(x, y, z);
      parts.push(paint(mesh, color));
    };
    const building = (x: number, z: number, width: number, depth: number, height: number) => {
      const mesh = chamferBox("region-building", { width, depth, height, chamfer: 0.2 }, scene);
      mesh.position.set(x, height / 2, z);
      parts.push(paint(mesh, PALETTE.crate));
      colliders.push(
        world.createCollider(
          rapier.ColliderDesc.cuboid(width / 2, height / 2, depth / 2).setTranslation(
            x,
            height / 2,
            z,
          ),
        ),
      );
    };
    if (r.road) {
      surface(128, oz + 500, 16, 1000, PALETTE.asphalt);
      for (const side of [-1, 1]) {
        surface(128 + side * 7, oz + 500, 0.2, 1000, PALETTE.roadLine, 0.03);
      }
      for (let z = oz + 5; z < oz + 1000; z += 20) {
        surface(128, z, 0.2, 6, PALETTE.paintYellow, 0.03);
      }
    }
    if (r.town) {
      surface(194, oz + 500, 116, 160, PALETTE.asphalt);
      for (let i = 0; i < 4; i++) {
        building(230, oz + 445 + i * 36, 30, 26, 8 + (i % 2) * 3);
      }
    }
    // Posto intermediário: pátio e cobertura originais, ainda sem abastecimento.
    if (r.x === 0 && r.z === 0) {
      surface(186, 500, 100, 100, PALETTE.asphalt);
      building(218, 500, 20, 30, 5);
      const roof = chamferBox(
        "station-roof",
        { width: 30, depth: 24, height: 0.6, chamfer: 0.1 },
        scene,
      );
      roof.position.set(174, 6, 500);
      parts.push(paint(roof, PALETTE.paintRed));
      for (const z of [492, 508]) building(174, z, 1, 1, 5.7);
    }
    // Todas as peças compartilham material: evita criar MultiMaterial por carga.
    for (const part of parts) {
      for (const kind of [VertexBuffer.UVKind, VertexBuffer.UV2Kind, VertexBuffer.TangentKind]) {
        if (part.isVerticesDataPresent(kind)) part.removeVerticesData(kind);
      }
    }
    const merged = Mesh.MergeMeshes(parts, true, true);
    if (!merged) throw new Error("Não foi possível montar a região");
    merged.name = `region-${r.x}-${r.z}`;
    merged.material = material;
    merged.receiveShadows = true;
    merged.isPickable = false;
    merged.freezeWorldMatrix();
    return { meshes: [merged], colliders };
  };

  const update = (x: number, z: number, anchors = new Float64Array()) => {
    const serialized = world_regions(x, z, anchors);
    if (serialized === previous) return;
    const regions = JSON.parse(serialized) as Region[];
    const wanted = new Set(regions.map((r) => `${r.x},${r.z}`));
    // Primeiro cria o chão novo, antes do próximo passo da física.
    for (const r of regions) {
      const key = `${r.x},${r.z}`;
      if (!loaded.has(key)) loaded.set(key, load(r));
    }
    for (const [key, region] of loaded) {
      if (wanted.has(key)) continue;
      for (const mesh of region.meshes) mesh.dispose();
      for (const collider of region.colliders) world.removeCollider(collider, true);
      loaded.delete(key);
    }
    currentTown = regions.find((r) => r.town)?.town ?? null;
    previous = serialized;
  };
  update(125, -4500);
  return {
    spawn: { p: [125, 1.6, -4500], q: [0, 0, 0, 1] },
    update,
    status: () => ({
      regions: [...loaded.keys()],
      town: currentTown,
      colliders: world.colliders.len(),
    }),
    storePrevious() {},
    captureCurrent() {},
    syncVisuals() {},
    updateGrass() {},
  };
}
