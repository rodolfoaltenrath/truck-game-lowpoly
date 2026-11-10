import { ArcRotateCamera, Quaternion, type Scene, UniversalCamera, Vector3 } from "@babylonjs/core";
import type { Truck } from "./vehicle/truck";

export type CameraMode = "perseguição" | "cabine" | "livre";
const MODES: CameraMode[] = ["perseguição", "cabine", "livre"];

const MOUSE_SENSITIVITY = 0.005;
/** Segundos sem mexer o mouse até a câmera voltar para trás do caminhão. */
const RECENTER_DELAY = 1.5;
const RECENTER_RATE = 2.5;
const CHASE_PITCH = 0.3;
const CHASE_PITCH_MIN = 0.03;
const CHASE_PITCH_MAX = 1.35;
const CHASE_DIST_MIN = 7;
const CHASE_DIST_MAX = 45;
const CAB_YAW_MAX = 2.4;
const CAB_PITCH_MAX = 0.8;

/**
 * Câmeras de perseguição, cabine e órbita livre.
 * Nas duas primeiras, arrastar com o mouse olha em volta e a roda do mouse
 * aproxima/afasta; soltando, a câmera volta sozinha para trás do caminhão.
 */
export class CameraRig {
  mode: CameraMode = "perseguição";
  private readonly follow: UniversalCamera;
  private readonly orbit: ArcRotateCamera;
  private readonly smoothPivot = new Vector3();
  private smoothHeading = 0;
  private initialized = false;

  private yawOffset = 0;
  private pitchOffset = 0;
  private chaseDistance = 17;
  private trailerTarget = 0;
  private trailerExtra = 0;
  private dragging = false;
  private idleTime = RECENTER_DELAY;

  constructor(
    private readonly scene: Scene,
    private readonly canvas: HTMLCanvasElement,
    private readonly truck: Truck,
  ) {
    this.follow = new UniversalCamera("follow", new Vector3(0, 6, -20), scene);
    this.follow.minZ = 0.3;
    this.follow.maxZ = 3000;
    this.follow.fov = 0.9;
    this.follow.inputs.clear();

    this.orbit = new ArcRotateCamera("orbit", -Math.PI / 2, 1.2, 22, Vector3.Zero(), scene);
    this.orbit.minZ = 0.3;
    this.orbit.maxZ = 3000;
    this.orbit.lowerRadiusLimit = 6;
    this.orbit.upperRadiusLimit = 120;
    this.orbit.wheelDeltaPercentage = 0.02;

    scene.activeCamera = this.follow;
    this.bindMouse();
  }

  private bindMouse(): void {
    const c = this.canvas;
    c.addEventListener("contextmenu", (e) => e.preventDefault());
    c.addEventListener("pointerdown", (e) => {
      if (this.mode === "livre") {
        return;
      }
      this.dragging = true;
      c.setPointerCapture(e.pointerId);
      c.style.cursor = "grabbing";
    });
    const stop = (e: PointerEvent) => {
      if (!this.dragging) {
        return;
      }
      this.dragging = false;
      this.idleTime = 0;
      if (c.hasPointerCapture(e.pointerId)) {
        c.releasePointerCapture(e.pointerId);
      }
      c.style.cursor = "";
    };
    c.addEventListener("pointerup", stop);
    c.addEventListener("pointercancel", stop);
    c.addEventListener("pointermove", (e) => {
      if (!this.dragging || this.mode === "livre") {
        return;
      }
      this.yawOffset -= e.movementX * MOUSE_SENSITIVITY;
      this.pitchOffset += e.movementY * MOUSE_SENSITIVITY;
      this.clampOffsets();
    });
    c.addEventListener(
      "wheel",
      (e) => {
        if (this.mode !== "perseguição") {
          return;
        }
        e.preventDefault();
        const factor = Math.exp(Math.sign(e.deltaY) * 0.1);
        this.chaseDistance = Math.min(
          CHASE_DIST_MAX,
          Math.max(CHASE_DIST_MIN, this.chaseDistance * factor),
        );
      },
      { passive: false },
    );
  }

  private clampOffsets(): void {
    if (this.mode === "cabine") {
      this.yawOffset = Math.max(-CAB_YAW_MAX, Math.min(CAB_YAW_MAX, this.yawOffset));
      this.pitchOffset = Math.max(-CAB_PITCH_MAX, Math.min(CAB_PITCH_MAX, this.pitchOffset));
    } else {
      this.yawOffset = wrapAngle(this.yawOffset);
      this.pitchOffset = Math.max(
        CHASE_PITCH_MIN - CHASE_PITCH,
        Math.min(CHASE_PITCH_MAX - CHASE_PITCH, this.pitchOffset),
      );
    }
  }

  /** Afasta a câmera de perseguição quando há reboque engatado. */
  setTrailerLength(length: number): void {
    this.trailerTarget = length * 0.85;
  }

  next(): void {
    const i = MODES.indexOf(this.mode);
    this.mode = MODES[(i + 1) % MODES.length] ?? "perseguição";
    this.yawOffset = 0;
    this.pitchOffset = 0;
    if (this.mode === "livre") {
      this.orbit.target.copyFrom(this.truck.root.position);
      this.orbit.attachControl(this.canvas, true);
      this.scene.activeCamera = this.orbit;
    } else {
      this.orbit.detachControl();
      this.scene.activeCamera = this.follow;
      this.initialized = false;
    }
  }

  update(dt: number): void {
    const root = this.truck.root;
    const q = root.rotationQuaternion ?? Quaternion.Identity();

    if (this.mode === "livre") {
      this.orbit.target = Vector3.Lerp(this.orbit.target, root.position, Math.min(1, dt * 8));
      return;
    }

    // Volta ao centro depois de um tempo sem arrastar.
    if (!this.dragging) {
      this.idleTime += dt;
      if (this.idleTime > RECENTER_DELAY) {
        const k = Math.exp(-dt * RECENTER_RATE);
        this.yawOffset *= k;
        this.pitchOffset *= k;
      }
    }

    if (this.mode === "cabine") {
      const local = (x: number, y: number, z: number) =>
        new Vector3(x, y, z).applyRotationQuaternion(q).addInPlace(root.position);
      const eye = local(0.55, 2.35, 2.6);
      const look = new Vector3(
        Math.sin(this.yawOffset) * Math.cos(this.pitchOffset),
        -Math.sin(this.pitchOffset) - 0.005,
        Math.cos(this.yawOffset) * Math.cos(this.pitchOffset),
      );
      const target = local(0.55 + look.x * 30, 2.35 + look.y * 30, 2.6 + look.z * 30);
      this.follow.position.copyFrom(eye);
      this.follow.setTarget(target);
      this.follow.fov = 1.05;
      return;
    }

    // Perseguição: órbita em volta do caminhão, rumo suavizado.
    const fwd = new Vector3(0, 0, 1).applyRotationQuaternion(q);
    const heading = Math.atan2(fwd.x, fwd.z);
    const pivot = root.position.add(new Vector3(0, 2, 0));
    if (!this.initialized) {
      this.smoothHeading = heading;
      this.smoothPivot.copyFrom(pivot);
      this.initialized = true;
    }
    this.smoothHeading += wrapAngle(heading - this.smoothHeading) * (1 - Math.exp(-dt * 3.5));
    this.smoothPivot.copyFrom(Vector3.Lerp(this.smoothPivot, pivot, 1 - Math.exp(-dt * 12)));

    this.trailerExtra += (this.trailerTarget - this.trailerExtra) * (1 - Math.exp(-dt * 1.5));
    const yaw = this.smoothHeading + this.yawOffset;
    const pitch = CHASE_PITCH + this.pitchOffset + this.trailerExtra * 0.006;
    const d = this.chaseDistance + this.trailerExtra;
    const pos = this.smoothPivot.add(
      new Vector3(
        -Math.sin(yaw) * Math.cos(pitch) * d,
        Math.sin(pitch) * d,
        -Math.cos(yaw) * Math.cos(pitch) * d,
      ),
    );
    if (pos.y < 1) {
      pos.y = 1;
    }
    // Olhando para trás do caminhão, mira o centro; de frente para a estrada, um pouco adiante.
    const lookAhead = 5 * Math.max(0, Math.cos(this.yawOffset));
    const target = this.smoothPivot.add(
      new Vector3(Math.sin(this.smoothHeading), 0, Math.cos(this.smoothHeading)).scale(lookAhead),
    );
    this.follow.position.copyFrom(pos);
    this.follow.setTarget(target);
    this.follow.fov = 0.9;
  }
}

function wrapAngle(a: number): number {
  return Math.atan2(Math.sin(a), Math.cos(a));
}
