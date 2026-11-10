import {
  Quaternion,
  type Scene,
  type ShadowGenerator,
  type StandardMaterial,
  TransformNode,
  Vector3,
} from "@babylonjs/core";
import type RAPIER from "@dimforge/rapier3d-compat";
import { chamferBox, facetedCylinder, mergeParts, paint } from "../render/geometry";
import { PALETTE } from "../render/palette";

/**
 * Rodas por raycast (Rapier) compartilhadas por cavalo e reboque: montagem
 * física, modelo low poly e interpolação do desenho entre passos fixos.
 */
export interface WheelSetSpec {
  wheels: ReadonlyArray<{ x: number; z: number }>;
  wheelRadius: number;
  wheelWidth: number;
  wheelMountY: number;
  suspensionRest: number;
  suspensionTravel: number;
  suspensionStiffness: number;
  suspensionCompression: number;
  suspensionRelaxation: number;
  frictionSlip: number;
  sideFrictionStiffness: number;
}

export interface WheelVisual {
  pivot: TransformNode;
  spinner: TransformNode;
}

/** Cria o controlador de veículo com +Y para cima e +Z para frente. */
export function createWheels(
  world: RAPIER.World,
  body: RAPIER.RigidBody,
  spec: WheelSetSpec,
): RAPIER.DynamicRayCastVehicleController {
  const v = world.createVehicleController(body);
  v.indexUpAxis = 1;
  v.setIndexForwardAxis = 2;
  for (const [i, w] of spec.wheels.entries()) {
    v.addWheel(
      { x: w.x, y: spec.wheelMountY, z: w.z },
      { x: 0, y: -1, z: 0 },
      { x: -1, y: 0, z: 0 },
      spec.suspensionRest,
      spec.wheelRadius,
    );
    v.setWheelMaxSuspensionTravel(i, spec.suspensionTravel);
    v.setWheelSuspensionStiffness(i, spec.suspensionStiffness);
    v.setWheelSuspensionCompression(i, spec.suspensionCompression);
    v.setWheelSuspensionRelaxation(i, spec.suspensionRelaxation);
    v.setWheelMaxSuspensionForce(i, 1e7);
    v.setWheelFrictionSlip(i, spec.frictionSlip);
    v.setWheelSideFrictionStiffness(i, spec.sideFrictionStiffness);
  }
  return v;
}

/** Modelo de cada roda: pneu facetado, aro claro, cubo e porcas. */
export function buildWheelVisuals(
  name: string,
  root: TransformNode,
  spec: WheelSetSpec,
  scene: Scene,
  shadows: ShadowGenerator,
  material: StandardMaterial,
): WheelVisual[] {
  const out: WheelVisual[] = [];
  for (const [i, w] of spec.wheels.entries()) {
    const pivot = new TransformNode(`${name}-wheel-pivot-${i}`, scene);
    pivot.parent = root;
    pivot.position.set(w.x, spec.wheelMountY - spec.suspensionRest, w.z);
    const spinner = new TransformNode(`${name}-wheel-spin-${i}`, scene);
    spinner.parent = pivot;

    const parts = [];
    const outward = Math.sign(w.x) || 1;
    const r = spec.wheelRadius;
    for (const [d, h, sides, color] of [
      [r * 2, spec.wheelWidth, 14, PALETTE.tire],
      [r * 1.2, spec.wheelWidth + 0.02, 10, PALETTE.rim],
      [0.26, spec.wheelWidth + 0.08, 8, PALETTE.hub],
    ] as const) {
      const m = facetedCylinder("wheel-part", { diameter: d, height: h, sides }, scene);
      m.rotation.z = Math.PI / 2;
      parts.push(paint(m, color));
    }
    // Porcas no aro: deixam o giro visível.
    for (let s = 0; s < 6; s++) {
      const a = (s / 6) * Math.PI * 2;
      const nut = chamferBox(
        "nut",
        { width: 0.05, height: 0.06, depth: 0.06, chamfer: 0.01 },
        scene,
      );
      nut.position.set(
        outward * (spec.wheelWidth / 2 + 0.02),
        Math.cos(a) * r * 0.38,
        Math.sin(a) * r * 0.38,
      );
      parts.push(paint(nut, PALETTE.hub));
    }
    const wheel = mergeParts(`${name}-wheel-${i}`, parts);
    wheel.parent = spinner;
    wheel.material = material;
    wheel.receiveShadows = true;
    shadows.addShadowCaster(wheel);
    out.push({ pivot, spinner });
  }
  return out;
}

/** Estado físico de um corpo com rodas, para interpolar o desenho. */
class Snapshot {
  readonly pos = new Vector3();
  readonly rot = new Quaternion();
  readonly suspension: number[];
  readonly spin: number[];
  readonly steer: number[];

  constructor(wheels: number) {
    this.suspension = new Array<number>(wheels).fill(0);
    this.spin = new Array<number>(wheels).fill(0);
    this.steer = new Array<number>(wheels).fill(0);
  }

  copyFrom(o: Snapshot): void {
    this.pos.copyFrom(o.pos);
    this.rot.copyFrom(o.rot);
    for (let i = 0; i < this.spin.length; i++) {
      this.suspension[i] = o.suspension[i] ?? 0;
      this.spin[i] = o.spin[i] ?? 0;
      this.steer[i] = o.steer[i] ?? 0;
    }
  }
}

/**
 * Guarda o penúltimo e o último estado da física e desenha entre os dois.
 * Ordem por passo: `storePrevious()` → física → `capture()`; por quadro:
 * `apply(alpha)`.
 */
export class WheeledInterpolation {
  private readonly prev: Snapshot;
  private readonly curr: Snapshot;

  constructor(
    private readonly body: RAPIER.RigidBody,
    private readonly vehicle: RAPIER.DynamicRayCastVehicleController,
    private readonly spec: WheelSetSpec,
    private readonly root: TransformNode,
    private readonly visuals: WheelVisual[],
  ) {
    this.prev = new Snapshot(spec.wheels.length);
    this.curr = new Snapshot(spec.wheels.length);
  }

  storePrevious(): void {
    this.prev.copyFrom(this.curr);
  }

  capture(): void {
    const t = this.body.translation();
    const r = this.body.rotation();
    this.curr.pos.set(t.x, t.y, t.z);
    this.curr.rot.set(r.x, r.y, r.z, r.w);
    for (let i = 0; i < this.spec.wheels.length; i++) {
      this.curr.suspension[i] = this.vehicle.wheelSuspensionLength(i) ?? this.spec.suspensionRest;
      this.curr.spin[i] = this.vehicle.wheelRotation(i) ?? 0;
      this.curr.steer[i] = this.vehicle.wheelSteering(i) ?? 0;
    }
  }

  /** Sem interpolação: ao criar e ao teleportar. */
  snap(): void {
    this.capture();
    this.prev.copyFrom(this.curr);
    this.apply(1);
  }

  apply(alpha: number): void {
    const a = this.prev;
    const b = this.curr;
    Vector3.LerpToRef(a.pos, b.pos, alpha, this.root.position);
    if (this.root.rotationQuaternion) {
      Quaternion.SlerpToRef(a.rot, b.rot, alpha, this.root.rotationQuaternion);
    }
    const lerp = (x: number, y: number) => x + (y - x) * alpha;
    for (const [i, wv] of this.visuals.entries()) {
      const w = this.spec.wheels[i];
      if (!w) {
        continue;
      }
      wv.pivot.position.set(
        w.x,
        this.spec.wheelMountY - lerp(a.suspension[i] ?? 0, b.suspension[i] ?? 0),
        w.z,
      );
      wv.pivot.rotation.y = lerp(a.steer[i] ?? 0, b.steer[i] ?? 0);
      wv.spinner.rotation.x = lerp(a.spin[i] ?? 0, b.spin[i] ?? 0);
    }
  }
}

/** Arrasto aerodinâmico + resistência ao rolamento como força no corpo. */
export function applyResistance(body: RAPIER.RigidBody, drag: number, rolling: number): void {
  const lin = body.linvel();
  const vel = Math.hypot(lin.x, lin.y, lin.z);
  body.resetForces(true);
  if (vel > 0.05) {
    const magnitude = drag * vel * vel + rolling * body.mass() * 9.81;
    const k = -magnitude / vel;
    body.addForce({ x: lin.x * k, y: lin.y * k, z: lin.z * k }, true);
  }
}
