import {
  type Color3,
  Quaternion,
  type Scene,
  type ShadowGenerator,
  type StandardMaterial,
  TransformNode,
  Vector3,
} from "@babylonjs/core";
import type RAPIER from "@dimforge/rapier3d-compat";
import type { DriveInput } from "../input";
import { PALETTE } from "../render/palette";
import { Drivetrain, type DrivetrainSpec } from "./drivetrain";
import { buildTruckModel, TAIL_DIM } from "./truckModel";

type Rapier = typeof RAPIER;

/**
 * Definição de dados de um cavalo mecânico. Unidades SI (m, kg, N).
 * Eixos locais: +Y para cima, +Z para frente, +X para a esquerda.
 */
export interface TruckSpec {
  frameHalfExtents: [number, number, number];
  frameMass: number;
  cabHalfExtents: [number, number, number];
  cabOffset: [number, number, number];
  cabMass: number;
  wheels: ReadonlyArray<{ x: number; z: number; steer: boolean; drive: boolean }>;
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
  maxSteer: number;
  /** Velocidade (m/s) em que o esterço máximo cai pela metade. */
  steerSpeedFalloff: number;
  /** Rad/s de giro do volante. */
  steerRate: number;
  /** Impulso de freio por roda com pedal no fundo. */
  brakeImpulse: number;
  handbrakeImpulse: number;
  engineBrakeImpulse: number;
  dragCoefficient: number;
  rollingResistance: number;
  drivetrain: DrivetrainSpec;
}

export const DEFAULT_TRUCK: TruckSpec = {
  frameHalfExtents: [0.9, 0.25, 3.3],
  frameMass: 6500,
  cabHalfExtents: [1.25, 1.1, 1.0],
  cabOffset: [0, 1.05, 2.45],
  cabMass: 1500,
  wheels: [
    { x: 1.05, z: 2.55, steer: true, drive: false },
    { x: -1.05, z: 2.55, steer: true, drive: false },
    { x: 1.05, z: -1.3, steer: false, drive: true },
    { x: -1.05, z: -1.3, steer: false, drive: true },
    { x: 1.05, z: -2.65, steer: false, drive: true },
    { x: -1.05, z: -2.65, steer: false, drive: true },
  ],
  wheelRadius: 0.52,
  wheelWidth: 0.4,
  wheelMountY: -0.2,
  suspensionRest: 0.45,
  suspensionTravel: 0.3,
  suspensionStiffness: 30,
  suspensionCompression: 2.6,
  suspensionRelaxation: 3.4,
  frictionSlip: 2.2,
  sideFrictionStiffness: 1.0,
  maxSteer: 0.6,
  steerSpeedFalloff: 10,
  steerRate: 1.6,
  brakeImpulse: 260,
  handbrakeImpulse: 600,
  engineBrakeImpulse: 18,
  dragCoefficient: 4.5,
  rollingResistance: 0.008,
  drivetrain: {
    idleRpm: 650,
    maxRpm: 2200,
    torqueCurve: [
      [650, 1300],
      [1000, 1900],
      [1400, 1900],
      [1900, 1500],
      [2200, 1100],
    ],
    gearRatios: [5.2, 3.9, 2.9, 2.2, 1.7, 1.33, 1.0, 0.82],
    reverseRatio: 5.0,
    finalDrive: 3.4,
    efficiency: 0.85,
    shiftUpRpm: 1850,
    shiftDownRpm: 1050,
    shiftTime: 0.35,
  },
};

export interface TruckTelemetry {
  speedKmh: number;
  gear: number;
  rpm: number;
  rpmFraction: number;
  wheelsInContact: number;
  position: { x: number; y: number; z: number };
  headingDeg: number;
  /** Rotação acumulada da roda dianteira esquerda (rad), para testes. */
  wheelSpin: number;
  /** Esterço atual das rodas dianteiras (rad). */
  steerAngle: number;
}

export interface TruckPose {
  p: [number, number, number];
  q: [number, number, number, number];
}

interface WheelVisual {
  pivot: TransformNode;
  spinner: TransformNode;
}

/** Estado físico usado para interpolar o desenho entre dois passos fixos. */
interface Snapshot {
  pos: Vector3;
  rot: Quaternion;
  suspension: number[];
  spin: number[];
  steer: number[];
}

function emptySnapshot(wheels: number): Snapshot {
  return {
    pos: new Vector3(),
    rot: new Quaternion(),
    suspension: new Array<number>(wheels).fill(0),
    spin: new Array<number>(wheels).fill(0),
    steer: new Array<number>(wheels).fill(0),
  };
}

/** Pedais de teclado são 0/1; isso dá a eles uma subida/descida gradual. */
const PEDAL_RISE = 3.5;
const PEDAL_FALL = 6;

function approach(current: number, target: number, dt: number): number {
  const rate = target > current ? PEDAL_RISE : PEDAL_FALL;
  const step = rate * dt;
  return current + Math.max(-step, Math.min(step, target - current));
}

export class Truck {
  readonly root: TransformNode;
  readonly body: RAPIER.RigidBody;
  private readonly vehicle: RAPIER.DynamicRayCastVehicleController;
  private readonly drivetrain: Drivetrain;
  private readonly wheelVisuals: WheelVisual[] = [];
  private readonly paintMaterial: StandardMaterial;
  private readonly tailMaterial: StandardMaterial;
  private steer = 0;
  private throttle = 0;
  private brake = 0;
  private reverseMode = false;
  private brakeLightOn = false;
  private readonly spawn: TruckPose;
  private readonly prev: Snapshot;
  private readonly curr: Snapshot;

  constructor(
    rapier: Rapier,
    private readonly world: RAPIER.World,
    scene: Scene,
    shadows: ShadowGenerator,
    spawn: TruckPose,
    readonly spec: TruckSpec = DEFAULT_TRUCK,
  ) {
    this.spawn = spawn;
    this.drivetrain = new Drivetrain(spec.drivetrain);

    const bodyDesc = rapier.RigidBodyDesc.dynamic()
      .setTranslation(...spawn.p)
      .setRotation({ x: spawn.q[0], y: spawn.q[1], z: spawn.q[2], w: spawn.q[3] })
      .setLinearDamping(0.02)
      .setAngularDamping(0.4)
      .setCanSleep(false);
    this.body = world.createRigidBody(bodyDesc);

    const [fx, fy, fz] = spec.frameHalfExtents;
    world.createCollider(
      rapier.ColliderDesc.cuboid(fx, fy, fz).setMass(spec.frameMass).setFriction(0.6),
      this.body,
    );
    const [cx, cy, cz] = spec.cabHalfExtents;
    world.createCollider(
      rapier.ColliderDesc.cuboid(cx, cy, cz)
        .setTranslation(...spec.cabOffset)
        .setMass(spec.cabMass)
        .setFriction(0.6),
      this.body,
    );

    this.vehicle = world.createVehicleController(this.body);
    this.vehicle.indexUpAxis = 1;
    this.vehicle.setIndexForwardAxis = 2;
    for (const [i, w] of spec.wheels.entries()) {
      this.vehicle.addWheel(
        { x: w.x, y: spec.wheelMountY, z: w.z },
        { x: 0, y: -1, z: 0 },
        { x: -1, y: 0, z: 0 },
        spec.suspensionRest,
        spec.wheelRadius,
      );
      this.vehicle.setWheelMaxSuspensionTravel(i, spec.suspensionTravel);
      this.vehicle.setWheelSuspensionStiffness(i, spec.suspensionStiffness);
      this.vehicle.setWheelSuspensionCompression(i, spec.suspensionCompression);
      this.vehicle.setWheelSuspensionRelaxation(i, spec.suspensionRelaxation);
      this.vehicle.setWheelMaxSuspensionForce(i, 1e7);
      this.vehicle.setWheelFrictionSlip(i, spec.frictionSlip);
      this.vehicle.setWheelSideFrictionStiffness(i, spec.sideFrictionStiffness);
    }

    this.root = new TransformNode("truck", scene);
    this.root.rotationQuaternion = new Quaternion();
    const model = buildTruckModel(this.root, spec, scene, shadows);
    this.wheelVisuals.push(...model.wheels);
    this.paintMaterial = model.paintMaterial;
    this.tailMaterial = model.tailMaterial;
    this.prev = emptySnapshot(spec.wheels.length);
    this.curr = emptySnapshot(spec.wheels.length);
    this.snap();
  }

  /** Guarda o estado atual como "anterior". Chamar antes de cada passo de física. */
  storePrevious(): void {
    copySnapshot(this.curr, this.prev);
  }

  /** Lê o estado da física. Chamar depois de cada passo de física. */
  captureCurrent(): void {
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

  /** Sem interpolação: usado ao criar e ao teleportar. */
  private snap(): void {
    this.captureCurrent();
    copySnapshot(this.curr, this.prev);
    this.syncVisuals(1);
  }

  /** Troca a cor da cabine. */
  setPaint(color: Color3): void {
    this.paintMaterial.diffuseColor.copyFrom(color);
  }

  /** Aplica controles. Chamar antes de `world.step`. */
  update(dt: number, input: DriveInput): void {
    const spec = this.spec;
    const v = this.vehicle;
    const speed = v.currentVehicleSpeed();

    // Ré estilo arcade: segurar freio parado engata a ré; acelerar volta para frente.
    if (!this.reverseMode && input.brake > 0.1 && input.throttle < 0.05 && speed < 0.8) {
      this.reverseMode = true;
    } else if (this.reverseMode && input.throttle > 0.1 && speed > -0.8) {
      this.reverseMode = false;
    }
    this.throttle = approach(this.throttle, input.throttle, dt);
    this.brake = approach(this.brake, input.brake, dt);
    const drive = this.reverseMode ? this.brake : this.throttle;
    const pedalBrake = this.reverseMode ? this.throttle : this.brake;

    const force = this.drivetrain.update(dt, drive, speed, spec.wheelRadius, this.reverseMode);
    const driven = spec.wheels.filter((w) => w.drive).length;

    const steerLimit = spec.maxSteer / (1 + Math.abs(speed) / spec.steerSpeedFalloff);
    const steerTarget = -input.steer * steerLimit;
    const maxDelta = spec.steerRate * dt;
    this.steer += Math.max(-maxDelta, Math.min(maxDelta, steerTarget - this.steer));

    const coasting = drive < 0.05 ? spec.engineBrakeImpulse : 0;
    for (const [i, w] of spec.wheels.entries()) {
      v.setWheelEngineForce(i, w.drive ? force / driven : 0);
      v.setWheelSteering(i, w.steer ? this.steer : 0);
      let brake = pedalBrake * spec.brakeImpulse + coasting;
      if (input.handbrake && w.drive) {
        brake = Math.max(brake, spec.handbrakeImpulse);
      }
      v.setWheelBrake(i, brake);
    }

    // Arrasto aerodinâmico + resistência ao rolamento.
    const lin = this.body.linvel();
    const vel = Math.hypot(lin.x, lin.y, lin.z);
    this.body.resetForces(true);
    if (vel > 0.05) {
      const mass = this.body.mass();
      const magnitude = spec.dragCoefficient * vel * vel + spec.rollingResistance * mass * 9.81;
      const k = -magnitude / vel;
      this.body.addForce({ x: lin.x * k, y: lin.y * k, z: lin.z * k }, true);
    }

    v.updateVehicle(dt);
    const lightOn = pedalBrake > 0.05 || input.handbrake;
    if (lightOn !== this.brakeLightOn) {
      this.brakeLightOn = lightOn;
      this.tailMaterial.emissiveColor = PALETTE.taillight.scale(lightOn ? 1.6 : TAIL_DIM);
    }
  }

  /**
   * Copia física → malhas, interpolando entre o passo anterior e o atual.
   * @param alpha fração (0..1) do tempo decorrido dentro do passo seguinte.
   */
  syncVisuals(alpha: number): void {
    const a = this.prev;
    const b = this.curr;
    Vector3.LerpToRef(a.pos, b.pos, alpha, this.root.position);
    if (this.root.rotationQuaternion) {
      Quaternion.SlerpToRef(a.rot, b.rot, alpha, this.root.rotationQuaternion);
    }

    const spec = this.spec;
    const lerp = (x: number, y: number) => x + (y - x) * alpha;
    for (const [i, wv] of this.wheelVisuals.entries()) {
      const w = spec.wheels[i];
      if (!w) {
        continue;
      }
      const len = lerp(a.suspension[i] ?? 0, b.suspension[i] ?? 0);
      wv.pivot.position.set(w.x, spec.wheelMountY - len, w.z);
      wv.pivot.rotation.y = lerp(a.steer[i] ?? 0, b.steer[i] ?? 0);
      wv.spinner.rotation.x = lerp(a.spin[i] ?? 0, b.spin[i] ?? 0);
    }
  }

  telemetry(): TruckTelemetry {
    const speed = this.vehicle.currentVehicleSpeed();
    const t = this.body.translation();
    const r = this.body.rotation();
    const forward = new Vector3(0, 0, 1).applyRotationQuaternion(
      new Quaternion(r.x, r.y, r.z, r.w),
    );
    let contact = 0;
    for (let i = 0; i < this.spec.wheels.length; i++) {
      if (this.vehicle.wheelIsInContact(i)) {
        contact++;
      }
    }
    const dt = this.drivetrain;
    return {
      speedKmh: speed * 3.6,
      gear: dt.gear,
      rpm: dt.rpm,
      rpmFraction: (dt.rpm - dt.spec.idleRpm) / (dt.spec.maxRpm - dt.spec.idleRpm),
      wheelsInContact: contact,
      position: { x: t.x, y: t.y, z: t.z },
      headingDeg: (Math.atan2(forward.x, forward.z) * 180) / Math.PI,
      wheelSpin: this.vehicle.wheelRotation(0) ?? 0,
      steerAngle: this.vehicle.wheelSteering(0) ?? 0,
    };
  }

  pose(): TruckPose {
    const t = this.body.translation();
    const r = this.body.rotation();
    return { p: [t.x, t.y, t.z], q: [r.x, r.y, r.z, r.w] };
  }

  /** Endireita o caminhão no lugar, mantendo a direção atual. */
  rightInPlace(): void {
    const t = this.body.translation();
    const half = (this.telemetry().headingDeg * Math.PI) / 180 / 2;
    this.teleport({ p: [t.x, t.y + 1.5, t.z], q: [0, Math.sin(half), 0, Math.cos(half)] });
  }

  /** Move para uma pose, parado. Sem argumento, volta ao ponto inicial. */
  teleport(pose: TruckPose = this.spawn): void {
    const target = pose;
    this.body.setTranslation({ x: target.p[0], y: target.p[1], z: target.p[2] }, true);
    this.body.setRotation({ x: target.q[0], y: target.q[1], z: target.q[2], w: target.q[3] }, true);
    this.body.setLinvel({ x: 0, y: 0, z: 0 }, true);
    this.body.setAngvel({ x: 0, y: 0, z: 0 }, true);
    this.steer = 0;
    this.throttle = 0;
    this.brake = 0;
    this.reverseMode = false;
    this.snap();
  }

  dispose(): void {
    this.world.removeVehicleController(this.vehicle);
    this.world.removeRigidBody(this.body);
    this.root.dispose();
  }
}

function copySnapshot(from: Snapshot, to: Snapshot): void {
  to.pos.copyFrom(from.pos);
  to.rot.copyFrom(from.rot);
  for (let i = 0; i < from.spin.length; i++) {
    to.suspension[i] = from.suspension[i] ?? 0;
    to.spin[i] = from.spin[i] ?? 0;
    to.steer[i] = from.steer[i] ?? 0;
  }
}
