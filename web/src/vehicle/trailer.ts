import {
  Quaternion,
  type Scene,
  type ShadowGenerator,
  type StandardMaterial,
  TransformNode,
  Vector3,
} from "@babylonjs/core";
import type RAPIER from "@dimforge/rapier3d-compat";
import { TRAILER_GROUPS } from "../physics/groups";
import { PALETTE } from "../render/palette";
import { buildTrailerModel } from "./trailerModel";
import { TAIL_DIM } from "./truckModel";
import {
  applyResistance,
  buildWheelVisuals,
  createWheels,
  WheeledInterpolation,
  type WheelSetSpec,
} from "./wheels";

type Rapier = typeof RAPIER;

export type TrailerKind = "bau" | "graneleiro" | "toras";

/**
 * Semirreboque. Origem local: centro do comprimento, y = 0 na face de baixo
 * do prato do pino-rei (o ponto que apoia na quinta-roda).
 * +Y para cima, +Z para frente, +X para a esquerda.
 */
export interface TrailerSpec extends WheelSetSpec {
  kind: TrailerKind;
  label: string;
  length: number;
  /** Distância do pino-rei até a frente do reboque. */
  kingpinFromFront: number;
  emptyMass: number;
  cargoMass: number;
  /** Colisor da carga/carroceria: meia-extensão e centro (y). */
  cargoHalf: [number, number, number];
  cargoCenterY: number;
  /** Distância dos pés de apoio até a frente. */
  legsFromFront: number;
  brakeImpulse: number;
  parkingBrakeImpulse: number;
  dragCoefficient: number;
  rollingResistance: number;
}

const AXLES = (length: number) => {
  const rear = -length / 2;
  const zs = [rear + 1.5, rear + 2.8, rear + 4.1];
  return zs.flatMap((z) => [
    { x: 1.05, z },
    { x: -1.05, z },
  ]);
};

const BASE = {
  wheelRadius: 0.5,
  wheelWidth: 0.4,
  wheelMountY: -0.55,
  suspensionRest: 0.45,
  suspensionTravel: 0.3,
  suspensionStiffness: 30,
  suspensionCompression: 2.6,
  suspensionRelaxation: 3.4,
  frictionSlip: 2.2,
  sideFrictionStiffness: 1.0,
  kingpinFromFront: 1.2,
  legsFromFront: 2.8,
  brakeImpulse: 300,
  parkingBrakeImpulse: 500,
  rollingResistance: 0.007,
};

const DECK = 0.32;

export const TRAILERS: Record<TrailerKind, TrailerSpec> = {
  bau: {
    ...BASE,
    kind: "bau",
    label: "Baú",
    length: 12,
    wheels: AXLES(12),
    emptyMass: 6500,
    cargoMass: 9000,
    cargoHalf: [1.25, 1.35, 5.9],
    cargoCenterY: DECK + 1.35,
    dragCoefficient: 3.2,
  },
  graneleiro: {
    ...BASE,
    kind: "graneleiro",
    label: "Graneleiro",
    length: 11,
    wheels: AXLES(11),
    emptyMass: 6000,
    cargoMass: 16000,
    cargoHalf: [1.25, 0.65, 5.3],
    cargoCenterY: DECK + 0.65,
    dragCoefficient: 2.2,
  },
  toras: {
    ...BASE,
    kind: "toras",
    label: "Toras",
    length: 12,
    wheels: AXLES(12),
    emptyMass: 5500,
    cargoMass: 14000,
    cargoHalf: [1.2, 0.7, 5.6],
    cargoCenterY: DECK + 0.7,
    dragCoefficient: 2.4,
  },
};

export interface TrailerPose {
  p: [number, number, number];
  q: [number, number, number, number];
}

export class Trailer {
  readonly root: TransformNode;
  readonly body: RAPIER.RigidBody;
  readonly kingpinLocal: { x: number; y: number; z: number };
  private readonly vehicle: RAPIER.DynamicRayCastVehicleController;
  private readonly legs: RAPIER.Collider[] = [];
  private readonly interp: WheeledInterpolation;
  private readonly tailMaterial: StandardMaterial;
  private brakeLightOn = false;
  coupled = false;

  constructor(
    rapier: Rapier,
    private readonly world: RAPIER.World,
    scene: Scene,
    shadows: ShadowGenerator,
    pose: TrailerPose,
    readonly spec: TrailerSpec,
  ) {
    this.kingpinLocal = { x: 0, y: 0, z: spec.length / 2 - spec.kingpinFromFront };
    this.body = world.createRigidBody(
      rapier.RigidBodyDesc.dynamic()
        .setTranslation(...pose.p)
        .setRotation({ x: pose.q[0], y: pose.q[1], z: pose.q[2], w: pose.q[3] })
        .setLinearDamping(0.02)
        .setAngularDamping(0.4)
        .setCanSleep(false),
    );
    const half = spec.length / 2;
    world.createCollider(
      rapier.ColliderDesc.cuboid(1.0, DECK / 2, half)
        .setTranslation(0, DECK / 2, 0)
        .setMass(spec.emptyMass)
        .setFriction(0.6)
        .setCollisionGroups(TRAILER_GROUPS),
      this.body,
    );
    const [cx, cy, cz] = spec.cargoHalf;
    world.createCollider(
      rapier.ColliderDesc.cuboid(cx, cy, cz)
        .setTranslation(0, spec.cargoCenterY, 0)
        .setMass(spec.cargoMass)
        .setFriction(0.6)
        .setCollisionGroups(TRAILER_GROUPS),
      this.body,
    );
    // Pés de apoio: só tocam o chão quando desengatado (desligados engatado).
    const legZ = half - spec.legsFromFront;
    for (const x of [-0.9, 0.9]) {
      this.legs.push(
        world.createCollider(
          rapier.ColliderDesc.cuboid(0.15, 0.7, 0.15)
            .setTranslation(x, -0.7, legZ)
            .setMass(50)
            .setFriction(1.2)
            .setCollisionGroups(TRAILER_GROUPS),
          this.body,
        ),
      );
    }

    this.vehicle = createWheels(world, this.body, spec);
    this.root = new TransformNode(`trailer-${spec.kind}`, scene);
    this.root.rotationQuaternion = new Quaternion();
    const model = buildTrailerModel(this.root, spec, scene, shadows);
    this.tailMaterial = model.tailMaterial;
    const visuals = buildWheelVisuals(
      `trailer-${spec.kind}`,
      this.root,
      spec,
      scene,
      shadows,
      model.flatMaterial,
    );
    this.interp = new WheeledInterpolation(this.body, this.vehicle, spec, this.root, visuals);
    this.interp.snap();
  }

  /** Massa total (vazio + carga), em kg. */
  get totalMass(): number {
    return this.spec.emptyMass + this.spec.cargoMass;
  }

  setCoupled(coupled: boolean): void {
    this.coupled = coupled;
    for (const leg of this.legs) {
      leg.setEnabled(!coupled);
    }
  }

  /** @param brake 0..1 vindo do cavalo; desengatado, freio de estacionamento. */
  update(dt: number, brake: number): void {
    const impulse = this.coupled ? brake * this.spec.brakeImpulse : this.spec.parkingBrakeImpulse;
    for (let i = 0; i < this.spec.wheels.length; i++) {
      this.vehicle.setWheelBrake(i, impulse);
      this.vehicle.setWheelEngineForce(i, 0);
    }
    applyResistance(this.body, this.spec.dragCoefficient, this.spec.rollingResistance);
    this.vehicle.updateVehicle(dt);

    const lightOn = this.coupled && brake > 0.05;
    if (lightOn !== this.brakeLightOn) {
      this.brakeLightOn = lightOn;
      this.tailMaterial.emissiveColor = PALETTE.taillight.scale(
        lightOn ? 1.6 : this.coupled ? TAIL_DIM : 0.05,
      );
    }
  }

  storePrevious(): void {
    this.interp.storePrevious();
  }

  captureCurrent(): void {
    this.interp.capture();
  }

  syncVisuals(alpha: number): void {
    this.interp.apply(alpha);
  }

  /** Posição do pino-rei no mundo. */
  kingpinWorld(): Vector3 {
    const t = this.body.translation();
    const r = this.body.rotation();
    const k = this.kingpinLocal;
    return new Vector3(k.x, k.y, k.z)
      .applyRotationQuaternion(new Quaternion(r.x, r.y, r.z, r.w))
      .addInPlaceFromFloats(t.x, t.y, t.z);
  }

  headingDeg(): number {
    const r = this.body.rotation();
    const f = new Vector3(0, 0, 1).applyRotationQuaternion(new Quaternion(r.x, r.y, r.z, r.w));
    return (Math.atan2(f.x, f.z) * 180) / Math.PI;
  }

  teleport(pose: TrailerPose): void {
    this.body.setTranslation({ x: pose.p[0], y: pose.p[1], z: pose.p[2] }, true);
    this.body.setRotation({ x: pose.q[0], y: pose.q[1], z: pose.q[2], w: pose.q[3] }, true);
    this.body.setLinvel({ x: 0, y: 0, z: 0 }, true);
    this.body.setAngvel({ x: 0, y: 0, z: 0 }, true);
    this.interp.snap();
  }

  dispose(): void {
    this.world.removeVehicleController(this.vehicle);
    this.world.removeRigidBody(this.body);
    this.root.dispose();
  }
}
