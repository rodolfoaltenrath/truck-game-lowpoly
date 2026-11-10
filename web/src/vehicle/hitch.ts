import { Quaternion, Vector3 } from "@babylonjs/core";
import type RAPIER from "@dimforge/rapier3d-compat";
import type { Trailer, TrailerPose } from "./trailer";
import { FIFTH_WHEEL_LOCAL, type Truck } from "./truck";

type Rapier = typeof RAPIER;

/** Limites para engatar: o pino-rei precisa estar sobre a quinta-roda. */
const MAX_HORIZONTAL = 1.0;
const MIN_VERTICAL = -0.45;
const MAX_VERTICAL = 0.35;
const MAX_HEADING_DEG = 35;
const MAX_SPEED_KMH = 6;
/** Engate automático de ré: até a velocidade máxima de ré (8 km/h) e um pouco. */
const AUTO_COUPLE_MAX_KMH = 9;
/** Distância para liberar o engate automático de um reboque recém-solto. */
const AUTO_RELEASE_DISTANCE = 3;

export interface HitchStatus {
  coupled: boolean;
  /** Nome do reboque engatado ou do candidato ao alcance. */
  label: string | null;
  massKg: number | null;
  /** Há um reboque ao alcance para engatar agora. */
  canCouple: boolean;
  /** Distância horizontal até o pino-rei mais próximo (m). */
  nearestDistance: number | null;
  /** Folga entre quinta-roda e pino-rei quando engatado (m). */
  jointGap: number | null;
  trailerHeadingDeg: number | null;
}

/**
 * Quinta-roda: liga o cavalo a um reboque com uma junta esférica do Rapier
 * entre o prato da quinta-roda e o pino-rei.
 */
export class Hitch {
  private joint: RAPIER.ImpulseJoint | null = null;
  private attached: Trailer | null = null;
  /** Reboque recém-desengatado: não engata sozinho até o cavalo se afastar. */
  private lockout: Trailer | null = null;
  private messages: string[] = [];

  constructor(
    private readonly rapier: Rapier,
    private readonly world: RAPIER.World,
    private readonly truck: Truck,
    readonly trailers: Trailer[],
  ) {}

  get trailer(): Trailer | null {
    return this.attached;
  }

  fifthWheelWorld(): Vector3 {
    const t = this.truck.body.translation();
    const r = this.truck.body.rotation();
    const f = FIFTH_WHEEL_LOCAL;
    return new Vector3(f.x, f.y, f.z)
      .applyRotationQuaternion(new Quaternion(r.x, r.y, r.z, r.w))
      .addInPlaceFromFloats(t.x, t.y, t.z);
  }

  /** Pose para um reboque nascer já engatado ao cavalo na pose atual. */
  poseBehindTruck(trailer: { kingpinLocal: { x: number; y: number; z: number } }): TrailerPose {
    const r = this.truck.body.rotation();
    const q = new Quaternion(r.x, r.y, r.z, r.w);
    const k = trailer.kingpinLocal;
    const p = this.fifthWheelWorld().subtract(
      new Vector3(k.x, k.y, k.z).applyRotationQuaternion(q),
    );
    return { p: [p.x, p.y, p.z], q: [q.x, q.y, q.z, q.w] };
  }

  private nearest(): { trailer: Trailer; horizontal: number; ok: boolean } | null {
    const fw = this.fifthWheelWorld();
    const truckHeading = this.truck.telemetry().headingDeg;
    let best: { trailer: Trailer; horizontal: number; ok: boolean } | null = null;
    for (const trailer of this.trailers) {
      if (trailer === this.attached) {
        continue;
      }
      const kp = trailer.kingpinWorld();
      const horizontal = Math.hypot(kp.x - fw.x, kp.z - fw.z);
      if (best && horizontal >= best.horizontal) {
        continue;
      }
      const vertical = kp.y - fw.y;
      let dh = Math.abs(trailer.headingDeg() - truckHeading) % 360;
      dh = dh > 180 ? 360 - dh : dh;
      const ok =
        horizontal <= MAX_HORIZONTAL &&
        vertical >= MIN_VERTICAL &&
        vertical <= MAX_VERTICAL &&
        dh <= MAX_HEADING_DEG;
      best = { trailer, horizontal, ok };
    }
    return best;
  }

  couple(trailer: Trailer): void {
    if (this.attached) {
      this.decouple();
    }
    const f = FIFTH_WHEEL_LOCAL;
    const data = this.rapier.JointData.spherical({ x: f.x, y: f.y, z: f.z }, trailer.kingpinLocal);
    this.joint = this.world.createImpulseJoint(data, this.truck.body, trailer.body, true);
    this.attached = trailer;
    trailer.setCoupled(true);
  }

  decouple(): void {
    if (this.joint) {
      this.world.removeImpulseJoint(this.joint, true);
    }
    this.attached?.setCoupled(false);
    this.joint = null;
    this.attached = null;
  }

  /** Tecla de engate: desengata, ou engata se houver reboque ao alcance. */
  toggle(): string {
    if (this.attached) {
      if (Math.abs(this.truck.telemetry().speedKmh) > MAX_SPEED_KMH) {
        return "Pare para desengatar";
      }
      const label = this.attached.spec.label;
      this.lockout = this.attached;
      this.decouple();
      return `${label} desengatado`;
    }
    const n = this.nearest();
    if (!n?.ok) {
      return "Nenhum reboque ao alcance: dê ré com a quinta-roda sob o pino-rei";
    }
    if (Math.abs(this.truck.telemetry().speedKmh) > MAX_SPEED_KMH) {
      return "Devagar para engatar";
    }
    this.couple(n.trailer);
    return `${n.trailer.spec.label} engatado`;
  }

  /** Mensagens geradas sozinhas (engate automático), para o HUD. */
  takeMessages(): string[] {
    const out = this.messages;
    this.messages = [];
    return out;
  }

  /** Chamar a cada passo de física, depois de `truck.update`. */
  update(dt: number): void {
    if (!this.attached) {
      const n = this.nearest();
      if (
        this.lockout &&
        (!n || n.trailer !== this.lockout || n.horizontal > AUTO_RELEASE_DISTANCE)
      ) {
        if (
          this.lockout.kingpinWorld().subtract(this.fifthWheelWorld()).length() >
          AUTO_RELEASE_DISTANCE
        ) {
          this.lockout = null;
        }
      }
      // Como a mandíbula da quinta-roda: engata sozinho ao encostar de ré.
      const speed = this.truck.telemetry().speedKmh;
      if (n?.ok && n.trailer !== this.lockout && speed < -0.2 && speed > -AUTO_COUPLE_MAX_KMH) {
        this.couple(n.trailer);
        this.messages.push(`${n.trailer.spec.label} engatado`);
      }
    }
    for (const trailer of this.trailers) {
      trailer.update(dt, trailer === this.attached ? this.truck.trailerBrake : 0);
    }
  }

  status(): HitchStatus {
    if (this.attached) {
      const gap = this.fifthWheelWorld().subtract(this.attached.kingpinWorld()).length();
      return {
        coupled: true,
        label: this.attached.spec.label,
        massKg: this.attached.totalMass,
        canCouple: false,
        nearestDistance: 0,
        jointGap: gap,
        trailerHeadingDeg: this.attached.headingDeg(),
      };
    }
    const n = this.nearest();
    return {
      coupled: false,
      label: n?.ok ? n.trailer.spec.label : null,
      massKg: n?.ok ? n.trailer.totalMass : null,
      canCouple: n?.ok ?? false,
      nearestDistance: n?.horizontal ?? null,
      jointGap: null,
      trailerHeadingDeg: null,
    };
  }
}
