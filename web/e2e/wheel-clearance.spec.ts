import {
  DirectionalLight,
  NullEngine,
  Ray,
  Scene,
  ShadowGenerator,
  TransformNode,
  Vector3,
} from "@babylonjs/core";
import { expect, test } from "@playwright/test";
import { TRAILERS } from "../src/vehicle/trailer";
import { buildTrailerModel } from "../src/vehicle/trailerModel";
import { DEFAULT_TRUCK } from "../src/vehicle/truck";
import { buildTruckModel } from "../src/vehicle/truckModel";
import type { WheelSetSpec } from "../src/vehicle/wheels";

test("cabine, degraus e para-lamas deixam espaço para os pneus com suspensão comprimida", () => {
  const engine = new NullEngine();
  try {
    const scene = new Scene(engine);
    scene.useRightHandedSystem = true;
    const sun = new DirectionalLight("sun", new Vector3(0, -1, 0), scene);
    const shadows = new ShadowGenerator(512, sun);
    const check = (root: TransformNode, spec: WheelSetSpec) => {
      const meshes = root.getChildMeshes();
      for (const mesh of meshes) mesh.computeWorldMatrix(true);
      for (const [i, wheel] of spec.wheels.entries()) {
        const side = Math.sign(wheel.x);
        for (const length of [
          spec.suspensionRest,
          Math.max(0, spec.suspensionRest - spec.suspensionTravel),
        ]) {
          for (const dy of [-0.9, -0.75, -0.35, 0, 0.35, 0.75, 0.9]) {
            for (const dz of [-0.9, -0.75, -0.35, 0, 0.35, 0.75, 0.9]) {
              if (Math.hypot(dy, dz) > 0.9) continue;
              const ray = new Ray(
                new Vector3(
                  wheel.x + side * (spec.wheelWidth / 2 + 0.01),
                  spec.wheelMountY - length + dy * spec.wheelRadius,
                  wheel.z + dz * spec.wheelRadius,
                ),
                new Vector3(-side, 0, 0),
                spec.wheelWidth + 0.02,
              );
              const intruders = meshes
                .filter((mesh) => ray.intersectsMesh(mesh).hit)
                .map((mesh) => mesh.name);
              expect(
                intruders,
                `${root.name}: roda ${i}, suspensão ${length}, amostra ${dy}/${dz}`,
              ).toEqual([]);
            }
          }
        }
      }
      root.dispose();
    };
    const truck = new TransformNode("truck", scene);
    buildTruckModel(truck, DEFAULT_TRUCK, scene, shadows);
    check(truck, DEFAULT_TRUCK);
    for (const spec of Object.values(TRAILERS)) {
      const root = new TransformNode(spec.kind, scene);
      buildTrailerModel(root, spec, scene, shadows);
      check(root, spec);
    }
  } finally {
    engine.dispose();
  }
});
