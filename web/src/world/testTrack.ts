import {
  Color3,
  Mesh,
  MeshBuilder,
  Quaternion,
  type Scene,
  type ShadowGenerator,
  StandardMaterial,
  Vector3,
  VertexBuffer,
} from "@babylonjs/core";
import type RAPIER from "@dimforge/rapier3d-compat";
import { chamferBox, facetedCylinder, paint, paintFacesVaried } from "../render/geometry";
import { PALETTE } from "../render/palette";
import type { TruckPose } from "../vehicle/truck";
import { GrassField } from "./grass";
import { buildScenery } from "./scenery";

type Rapier = typeof RAPIER;

/**
 * Pista de testes procedural, sem nenhum asset externo:
 * chão plano, oval de asfalto, rampas, slalom de cones, pilha de caixas e
 * uma área de morros para testar suspensão.
 */
export interface TestTrack {
  spawn: TruckPose;
  /** Guarda o estado atual como anterior (antes de cada passo de física). */
  storePrevious(): void;
  /** Lê a física (depois de cada passo). */
  captureCurrent(): void;
  /** Copia física → malhas dos objetos móveis, interpolando. */
  syncVisuals(alpha: number): void;
  /** Reposiciona os tufos de capim em volta da câmera. */
  updateGrass(camera: Vector3): void;
}

interface DynamicProp {
  mesh: Mesh;
  body: RAPIER.RigidBody;
  prevPos: Vector3;
  prevRot: Quaternion;
  currPos: Vector3;
  currRot: Quaternion;
}

const GROUND_SIZE = 2400;
const OVAL_HALF_STRAIGHT = 200;
const OVAL_RADIUS = 120;
const ROAD_WIDTH = 12;
const HILLS = { x: 300, z: 0, size: 220, edge: 35 };

/** Altura do terreno (0 fora da área de morros). */
export function terrainHeight(wx: number, wz: number): number {
  const x = wx - HILLS.x;
  const z = wz - HILLS.z;
  const distToEdge = HILLS.size / 2 - Math.max(Math.abs(x), Math.abs(z));
  if (distToEdge <= 0) {
    return 0;
  }
  const t = Math.min(1, distToEdge / HILLS.edge);
  const falloff = t * t * (3 - 2 * t);
  const h = 3 * (1 + Math.sin(x / 20) * Math.cos(z / 27)) + 1.5 * (1 + Math.sin((x - z) / 11));
  return h * falloff;
}

/** Distância horizontal até a linha central do oval. */
function distanceToOval(x: number, z: number): number {
  if (Math.abs(z) <= OVAL_HALF_STRAIGHT) {
    return Math.abs(Math.abs(x) - OVAL_RADIUS);
  }
  const zc = Math.sign(z) * OVAL_HALF_STRAIGHT;
  return Math.abs(Math.hypot(x, z - zc) - OVAL_RADIUS);
}

export function buildTestTrack(
  rapier: Rapier,
  world: RAPIER.World,
  scene: Scene,
  shadows: ShadowGenerator,
): TestTrack {
  const props: DynamicProp[] = [];
  const firstMesh = scene.meshes.length;

  const solid = (name: string, color: Color3): StandardMaterial => {
    const m = new StandardMaterial(name, scene);
    m.diffuseColor = color;
    m.specularColor = new Color3(0.05, 0.05, 0.05);
    return m;
  };
  // Material único para peças com cor por vértice.
  const vc = solid("track-vc", Color3.White());

  // --- Chão: capim facetado com manchas de cor --------------------------------
  const ground = MeshBuilder.CreateGround(
    "ground",
    { width: GROUND_SIZE, height: GROUND_SIZE, subdivisions: 120 },
    scene,
  );
  ground.convertToFlatShadedMesh();
  const grass = [PALETTE.grassA, PALETTE.grassB, PALETTE.grassC, PALETTE.grassD];
  // Manchas suaves: cor contínua por ruído de baixa frequência (sem sorteio
  // por triângulo, que fica ruidoso de longe).
  paintFacesVaried(ground, grass, 11, (c) => {
    const n = (Math.sin(c.x / 140) + Math.cos(c.z / 110) + 0.5 * Math.sin((c.x + c.z) / 60)) / 2.5;
    const t = (n + 1) / 2;
    const a = grass[0] ?? PALETTE.grassA;
    const b = grass[2] ?? PALETTE.grassC;
    return Color3.Lerp(a, b, t);
  });
  ground.material = vc;
  ground.receiveShadows = true;
  world.createCollider(
    rapier.ColliderDesc.cuboid(GROUND_SIZE / 2, 0.5, GROUND_SIZE / 2)
      .setTranslation(0, -0.5, 0)
      .setFriction(0.9),
  );

  // --- Oval de asfalto (visual; o atrito é o mesmo do chão) ---------------
  const asphalt = solid("asphalt", PALETTE.asphalt);
  const line = solid("road-line", PALETTE.roadLine);
  const center = ovalCenterline(4);
  const offset = (d: number) =>
    center.map((p, i) => {
      const next = center[(i + 1) % center.length] ?? p;
      const prev = center[(i - 1 + center.length) % center.length] ?? p;
      const dir = next.subtract(prev).normalize();
      const normal = new Vector3(dir.z, 0, -dir.x);
      return p.add(normal.scale(d));
    });
  const road = MeshBuilder.CreateRibbon(
    "road",
    { pathArray: [offset(-ROAD_WIDTH / 2), offset(ROAD_WIDTH / 2)], closePath: true },
    scene,
  );
  road.position.y = 0.02;
  road.material = asphalt;
  road.receiveShadows = true;
  for (const side of [-1, 1]) {
    const edge = MeshBuilder.CreateRibbon(
      `road-edge-${side}`,
      {
        pathArray: [offset(side * (ROAD_WIDTH / 2 - 0.5)), offset(side * (ROAD_WIDTH / 2 - 0.25))],
        closePath: true,
      },
      scene,
    );
    edge.position.y = 0.03;
    edge.material = line;
  }
  const dashes: Mesh[] = [];
  for (let i = 0; i < center.length; i += 3) {
    const p = center[i];
    const n = center[(i + 1) % center.length];
    if (!p || !n) {
      continue;
    }
    const dash = MeshBuilder.CreateGround(`dash-${i}`, { width: 0.25, height: 3 }, scene);
    dash.position.set(p.x, 0.03, p.z);
    dash.rotation.y = Math.atan2(n.x - p.x, n.z - p.z);
    dashes.push(dash);
  }
  const dashMesh = Mesh.MergeMeshes(dashes, true, true);
  if (dashMesh) {
    dashMesh.name = "road-dashes";
    dashMesh.material = line;
  }

  // --- Rampas ---------------------------------------------------------------

  const ramp = (
    name: string,
    x: number,
    z: number,
    length: number,
    width: number,
    angle: number,
  ) => {
    const thickness = 1;
    const mesh = paint(
      chamferBox(name, { width, height: thickness, depth: length, chamfer: 0.12 }, scene),
      PALETTE.ramp,
    );
    const rise = Math.abs(Math.sin(angle)) * (length / 2);
    const y = rise - Math.cos(angle) * (thickness / 2);
    mesh.position.set(x, y, z);
    mesh.rotation.x = -angle;
    mesh.material = vc;
    mesh.receiveShadows = true;
    shadows.addShadowCaster(mesh);
    const q = Quaternion.FromEulerAngles(-angle, 0, 0);
    world.createCollider(
      rapier.ColliderDesc.cuboid(width / 2, thickness / 2, length / 2)
        .setTranslation(x, y, z)
        .setRotation({ x: q.x, y: q.y, z: q.z, w: q.w })
        .setFriction(0.9),
    );
  };
  ramp("ramp-small", 30, -20, 16, 8, 0.12);
  ramp("ramp-big", 45, 30, 24, 10, 0.2);
  ramp("ramp-big-down", 45, 54, 24, 10, -0.2);

  // --- Slalom de cones -------------------------------------------------------
  for (let i = 0; i < 8; i++) {
    const z = -60 + i * 20;
    const mesh = paint(
      facetedCylinder(
        `cone-${i}`,
        { diameterTop: 0.08, diameterBottom: 0.5, height: 0.8, sides: 8 },
        scene,
      ),
      PALETTE.cone,
    );
    mesh.material = vc;
    shadows.addShadowCaster(mesh);
    const body = world.createRigidBody(rapier.RigidBodyDesc.dynamic().setTranslation(-30, 0.4, z));
    world.createCollider(rapier.ColliderDesc.cone(0.4, 0.25).setMass(4).setFriction(0.7), body);
    props.push(newProp(mesh, body));
  }

  // --- Pilha de caixas ------------------------------------------------------------
  const size = 1.2;
  for (let layer = 0; layer < 4; layer++) {
    for (let col = 0; col < 4 - layer; col++) {
      const x = -((4 - layer - 1) * size) / 2 + col * size;
      const y = size / 2 + layer * size;
      const s = size * 0.98;
      const mesh = paint(
        chamferBox(
          `crate-${layer}-${col}`,
          { width: s, height: s, depth: s, chamfer: 0.08 },
          scene,
        ),
        PALETTE.crate,
      );
      mesh.material = vc;
      mesh.receiveShadows = true;
      shadows.addShadowCaster(mesh);
      const body = world.createRigidBody(rapier.RigidBodyDesc.dynamic().setTranslation(x, y, 70));
      world.createCollider(
        rapier.ColliderDesc.cuboid(size * 0.49, size * 0.49, size * 0.49)
          .setMass(60)
          .setFriction(0.6),
        body,
      );
      props.push(newProp(mesh, body));
    }
  }

  // --- Morros (trimesh) -------------------------------------------------------
  buildHills(rapier, world, scene, vc);

  // --- Árvores e pedras ----------------------------------------------------------
  buildScenery(rapier, world, scene, shadows, {
    count: 700,
    radius: 1100,
    seed: 2026,
    height: terrainHeight,
    allowed: (x, z) => {
      if (distanceToOval(x, z) < 20) {
        return false;
      }
      // Área de testes (rampas, cones, caixas, ponto inicial).
      if (Math.abs(x) < 85 && z > -125 && z < 115) {
        return false;
      }
      return true;
    },
  });

  // --- Tufos de capim perto da câmera --------------------------------------------
  const grassField = new GrassField(scene, {
    radius: 55,
    cell: 1.4,
    density: 0.7,
    height: terrainHeight,
    allowed: (x, z) => distanceToOval(x, z) > ROAD_WIDTH / 2 + 1,
  });

  // Tudo que esta pista criou e não se move tem a matriz congelada.
  const moving = new Set<Mesh>(props.map((p) => p.mesh));
  for (const mesh of scene.meshes.slice(firstMesh)) {
    if (!moving.has(mesh as Mesh) && !mesh.hasThinInstances) {
      mesh.freezeWorldMatrix();
      mesh.doNotSyncBoundingInfo = true;
    }
  }
  for (const p of props) {
    p.mesh.rotationQuaternion = new Quaternion();
    captureProp(p);
    p.prevPos.copyFrom(p.currPos);
    p.prevRot.copyFrom(p.currRot);
  }

  return {
    spawn: { p: [0, 1.6, -80], q: [0, 0, 0, 1] },
    updateGrass(camera: Vector3) {
      grassField.update(camera);
    },
    storePrevious() {
      for (const p of props) {
        p.prevPos.copyFrom(p.currPos);
        p.prevRot.copyFrom(p.currRot);
      }
    },
    captureCurrent() {
      for (const p of props) {
        captureProp(p);
      }
    },
    syncVisuals(alpha: number) {
      for (const p of props) {
        Vector3.LerpToRef(p.prevPos, p.currPos, alpha, p.mesh.position);
        if (p.mesh.rotationQuaternion) {
          Quaternion.SlerpToRef(p.prevRot, p.currRot, alpha, p.mesh.rotationQuaternion);
        }
      }
    },
  };
}

function newProp(mesh: Mesh, body: RAPIER.RigidBody): DynamicProp {
  return {
    mesh,
    body,
    prevPos: new Vector3(),
    prevRot: new Quaternion(),
    currPos: new Vector3(),
    currRot: new Quaternion(),
  };
}

function captureProp(p: DynamicProp): void {
  const t = p.body.translation();
  const r = p.body.rotation();
  p.currPos.set(t.x, t.y, t.z);
  p.currRot.set(r.x, r.y, r.z, r.w);
}

function ovalCenterline(step: number): Vector3[] {
  const points: Vector3[] = [];
  const x = OVAL_RADIUS;
  for (let z = -OVAL_HALF_STRAIGHT; z < OVAL_HALF_STRAIGHT; z += step) {
    points.push(new Vector3(x, 0, z));
  }
  const arc = Math.ceil((Math.PI * OVAL_RADIUS) / step);
  for (let i = 0; i < arc; i++) {
    const a = (i / arc) * Math.PI;
    points.push(
      new Vector3(OVAL_RADIUS * Math.cos(a), 0, OVAL_HALF_STRAIGHT + OVAL_RADIUS * Math.sin(a)),
    );
  }
  for (let z = OVAL_HALF_STRAIGHT; z > -OVAL_HALF_STRAIGHT; z -= step) {
    points.push(new Vector3(-x, 0, z));
  }
  for (let i = 0; i < arc; i++) {
    const a = Math.PI + (i / arc) * Math.PI;
    points.push(
      new Vector3(OVAL_RADIUS * Math.cos(a), 0, -OVAL_HALF_STRAIGHT + OVAL_RADIUS * Math.sin(a)),
    );
  }
  return points;
}

function buildHills(
  rapier: Rapier,
  world: RAPIER.World,
  scene: Scene,
  material: StandardMaterial,
): void {
  const size = HILLS.size;
  const cx = HILLS.x;
  const cz = HILLS.z;
  const mesh = MeshBuilder.CreateGround(
    "hills",
    { width: size, height: size, subdivisions: 88, updatable: true },
    scene,
  );
  // Levemente acima do chão base para não disputar profundidade nas bordas.
  mesh.position.set(cx, 0.02, cz);
  mesh.material = material;
  mesh.receiveShadows = true;

  const positions = mesh.getVerticesData(VertexBuffer.PositionKind);
  const indices = mesh.getIndices();
  if (!positions || !indices) {
    return;
  }
  for (let i = 0; i < positions.length; i += 3) {
    positions[i + 1] = terrainHeight((positions[i] ?? 0) + cx, (positions[i + 2] ?? 0) + cz);
  }
  mesh.updateVerticesData(VertexBuffer.PositionKind, positions);
  const colliderPositions = new Float32Array(positions);
  mesh.convertToFlatShadedMesh();
  paintFacesVaried(mesh, [PALETTE.hillLow], 5, (c) =>
    Color3.Lerp(PALETTE.hillLow, PALETTE.hillHigh, Math.min(1, c.y / 9)),
  );

  world.createCollider(
    rapier.ColliderDesc.trimesh(colliderPositions, new Uint32Array(indices))
      .setTranslation(cx, 0, cz)
      .setFriction(0.9),
  );
}
