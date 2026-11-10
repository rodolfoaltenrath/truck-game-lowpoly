import {
  Quaternion,
  type Scene,
  type ShadowGenerator,
  type StandardMaterial,
  TransformNode,
  Vector3,
} from "@babylonjs/core";
import { PAINTS } from "../render/palette";
import { TRAILERS, type TrailerKind } from "../vehicle/trailer";
import { buildTrailerModel } from "../vehicle/trailerModel";
import { DEFAULT_TRUCK } from "../vehicle/truck";
import { buildTruckModel } from "../vehicle/truckModel";
import { buildWheelVisuals, type WheelSetSpec, type WheelVisual } from "../vehicle/wheels";
import type { RemoteView } from "./session";

/** Ordem dos tipos de reboque no protocolo (índice = `kind`). */
export const TRAILER_KINDS: readonly TrailerKind[] = ["bau", "graneleiro", "toras"];

interface Wheeled {
  root: TransformNode;
  wheels: WheelVisual[];
  spec: WheelSetSpec;
  spin: number;
  last: Vector3 | null;
}

interface RemoteVehicle {
  truck: Wheeled;
  paint: StandardMaterial;
  paintIndex: number;
  trailers: Map<number, Wheeled>;
  seen: boolean;
}

/**
 * Caminhões dos outros jogadores: mesmo modelo low poly do jogador, sem
 * física. Pose e reboque vêm da interpolação do `game_core`; as rodas giram
 * pela distância percorrida e esterçam pelo valor recebido.
 */
export class RemoteVehicles {
  private readonly vehicles = new Map<number, RemoteVehicle>();

  constructor(
    private readonly scene: Scene,
    private readonly shadows: ShadowGenerator,
  ) {}

  get count(): number {
    return this.vehicles.size;
  }

  private wheeled(
    name: string,
    spec: WheelSetSpec,
    build: (root: TransformNode) => StandardMaterial,
  ): Wheeled {
    const root = new TransformNode(name, this.scene);
    root.rotationQuaternion = new Quaternion();
    const material = build(root);
    const wheels = buildWheelVisuals(name, root, spec, this.scene, this.shadows, material);
    for (const [i, w] of wheels.entries()) {
      const s = spec.wheels[i];
      if (s) {
        // Suspensão em repouso (sem física nos remotos).
        w.pivot.position.set(s.x, spec.wheelMountY - spec.suspensionRest + 0.04, s.z);
      }
    }
    return { root, wheels, spec, spin: 0, last: null };
  }

  private create(player: number): RemoteVehicle {
    let paint: StandardMaterial | null = null;
    const truck = this.wheeled(`remote-${player}`, DEFAULT_TRUCK, (root) => {
      const model = buildTruckModel(root, DEFAULT_TRUCK, this.scene, this.shadows);
      paint = model.paintMaterial;
      return model.flatMaterial;
    });
    if (!paint) {
      throw new Error("modelo sem material de pintura");
    }
    return { truck, paint, paintIndex: -1, trailers: new Map(), seen: true };
  }

  private trailer(v: RemoteVehicle, player: number, kind: number): Wheeled | null {
    const existing = v.trailers.get(kind);
    if (existing) {
      return existing;
    }
    const name = TRAILER_KINDS[kind];
    if (!name) {
      return null;
    }
    const spec = TRAILERS[name];
    const w = this.wheeled(`remote-${player}-trailer-${name}`, spec, (root) => {
      const model = buildTrailerModel(root, spec, this.scene, this.shadows);
      model.tailMaterial.emissiveColor = model.tailMaterial.diffuseColor.scale(0.35);
      return model.flatMaterial;
    });
    v.trailers.set(kind, w);
    return w;
  }

  private place(w: Wheeled, position: number[], rotation: number[], steer: number): void {
    const p = new Vector3(position[0] ?? 0, position[1] ?? 0, position[2] ?? 0);
    w.root.position.copyFrom(p);
    w.root.rotationQuaternion?.set(
      rotation[0] ?? 0,
      rotation[1] ?? 0,
      rotation[2] ?? 0,
      rotation[3] ?? 1,
    );
    w.root.setEnabled(true);
    // Giro das rodas pela distância andada no sentido da frente.
    if (w.last) {
      const delta = p.subtract(w.last);
      const forward = new Vector3(0, 0, 1).applyRotationQuaternion(
        w.root.rotationQuaternion ?? Quaternion.Identity(),
      );
      const along = Vector3.Dot(delta, forward);
      if (Math.abs(along) < 20) {
        w.spin += along / w.spec.wheelRadius;
      }
    }
    w.last = p;
    for (const [i, wheel] of w.wheels.entries()) {
      wheel.spinner.rotation.x = w.spin;
      const isFront = i < 2 && "maxSteer" in w.spec;
      wheel.pivot.rotation.y = isFront ? steer : 0;
    }
  }

  update(views: RemoteView[]): void {
    for (const v of this.vehicles.values()) {
      v.seen = false;
    }
    for (const view of views) {
      let v = this.vehicles.get(view.player);
      if (!v) {
        v = this.create(view.player);
        this.vehicles.set(view.player, v);
      }
      v.seen = true;
      if (v.paintIndex !== view.paint) {
        v.paintIndex = view.paint;
        const p = PAINTS[view.paint % PAINTS.length];
        if (p) {
          v.paint.diffuseColor.copyFrom(p.color);
        }
      }
      this.place(v.truck, view.position, view.rotation, view.steer_mrad / 1000);
      for (const [kind, t] of v.trailers) {
        if (!view.trailer || view.trailer.kind !== kind) {
          t.root.setEnabled(false);
          t.last = null;
        }
      }
      if (view.trailer) {
        const t = this.trailer(v, view.player, view.trailer.kind);
        if (t) {
          this.place(t, view.trailer.position, view.trailer.rotation, 0);
        }
      }
    }
    // Quem sumiu (sem notícias ou saiu): esconde; quem saiu da sala de vez
    // é reaproveitado se voltar com o mesmo número.
    for (const v of this.vehicles.values()) {
      if (!v.seen) {
        v.truck.root.setEnabled(false);
        v.truck.last = null;
        for (const t of v.trailers.values()) {
          t.root.setEnabled(false);
          t.last = null;
        }
      }
    }
  }
}
