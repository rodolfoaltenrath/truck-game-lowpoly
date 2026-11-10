import {
  Color3,
  type Mesh,
  type Scene,
  type ShadowGenerator,
  StandardMaterial,
  type TransformNode,
} from "@babylonjs/core";
import { chamferBox, facetedCylinder, mergeParts, paint } from "../render/geometry";
import { PALETTE } from "../render/palette";
import type { TruckSpec } from "./truck";

/**
 * Modelo low poly do cavalo mecânico de cabine avançada, todo gerado por
 * código. Peças de cor fixa usam cor por vértice e são fundidas numa malha;
 * pintura, vidro e luzes têm material próprio (trocar cor, brilho, acender).
 *
 * Eixos locais: +Y para cima, +Z para frente, +X para a esquerda.
 * y = 0 é o centro do corpo físico (~1,2 m acima do chão em repouso).
 */
export interface TruckModel {
  /** Material de cor por vértice (reaproveitado pelas rodas). */
  flatMaterial: StandardMaterial;
  paintMaterial: StandardMaterial;
  tailMaterial: StandardMaterial;
}

const CAB = { width: 2.5, height: 2.2, depth: 2.0, x: 0, y: 1.05, z: 2.45, chamfer: 0.14 };
const CAB_FRONT = CAB.z + CAB.depth / 2;
const CAB_BOTTOM = CAB.y - CAB.height / 2;
const REAR = -3.35;

export const TAIL_DIM = 0.35;

export function buildTruckModel(
  root: TransformNode,
  spec: TruckSpec,
  scene: Scene,
  shadows: ShadowGenerator,
): TruckModel {
  const flat = new StandardMaterial("truck-vc", scene);
  flat.diffuseColor = Color3.White();
  flat.specularColor = new Color3(0.12, 0.12, 0.12);

  const paintMaterial = new StandardMaterial("truck-paint", scene);
  paintMaterial.diffuseColor = PALETTE.paintRed.clone();
  paintMaterial.specularColor = new Color3(0.35, 0.35, 0.35);
  paintMaterial.specularPower = 48;

  const glass = new StandardMaterial("truck-glass", scene);
  glass.diffuseColor = PALETTE.glass;
  glass.emissiveColor = PALETTE.glass.scale(0.25);
  glass.specularColor = new Color3(0.9, 0.9, 0.9);
  glass.specularPower = 96;

  const head = new StandardMaterial("truck-headlight", scene);
  head.diffuseColor = PALETTE.headlight;
  head.emissiveColor = PALETTE.headlight;
  head.disableLighting = true;

  const tailMaterial = new StandardMaterial("truck-taillight", scene);
  tailMaterial.diffuseColor = PALETTE.taillight;
  tailMaterial.emissiveColor = PALETTE.taillight.scale(TAIL_DIM);

  const fixed: Mesh[] = [];
  const painted: Mesh[] = [];
  const glazed: Mesh[] = [];
  const heads: Mesh[] = [];
  const tails: Mesh[] = [];

  const box = (
    list: Mesh[],
    color: Color3 | null,
    w: number,
    h: number,
    d: number,
    x: number,
    y: number,
    z: number,
    chamfer = 0.03,
  ): Mesh => {
    const m = chamferBox("part", { width: w, height: h, depth: d, chamfer }, scene);
    m.position.set(x, y, z);
    if (color) {
      paint(m, color);
    }
    list.push(m);
    return m;
  };

  // --- Cabine ---------------------------------------------------------------
  const frontAxleZ = spec.wheels.find((w) => w.steer)?.z ?? 2.55;
  const archRear = frontAxleZ - spec.wheelRadius - 0.12;
  const archFront = frontAxleZ + spec.wheelRadius + 0.12;
  const wheelTop =
    spec.wheelMountY - Math.max(0, spec.suspensionRest - spec.suspensionTravel) + spec.wheelRadius;
  const archTop = wheelTop + 0.1;
  const cabTop = CAB.y + CAB.height / 2;
  // Parte superior inteira; saia inferior só antes/depois do vão da roda.
  box(
    painted,
    null,
    CAB.width,
    cabTop - archTop,
    CAB.depth,
    CAB.x,
    (cabTop + archTop) / 2,
    CAB.z,
    CAB.chamfer,
  );
  for (const [z0, z1] of [
    [CAB.z - CAB.depth / 2, archRear],
    [archFront, CAB_FRONT],
  ]) {
    if (z0 === undefined || z1 === undefined || z1 <= z0) continue;
    box(
      painted,
      null,
      CAB.width,
      archTop - CAB_BOTTOM,
      z1 - z0,
      0,
      (archTop + CAB_BOTTOM) / 2,
      (z0 + z1) / 2,
      0.04,
    );
    box(
      fixed,
      PALETTE.chassisDark,
      CAB.width - 0.04,
      0.16,
      z1 - z0,
      0,
      CAB_BOTTOM + 0.04,
      (z0 + z1) / 2,
      0.04,
    );
  }
  // Pala de sol sobre o para-brisa.
  box(
    painted,
    null,
    CAB.width - 0.2,
    0.1,
    0.28,
    0,
    CAB.y + CAB.height / 2 - 0.12,
    CAB_FRONT + 0.08,
    0.04,
  );
  // Para-brisa e janelas laterais.
  box(glazed, null, CAB.width - 0.34, 0.92, 0.06, 0, CAB.y + 0.4, CAB_FRONT + 0.005, 0.04);
  for (const s of [-1, 1]) {
    box(
      glazed,
      null,
      0.06,
      0.72,
      0.95,
      s * (CAB.width / 2 + 0.005),
      CAB.y + 0.42,
      CAB.z + 0.38,
      0.04,
    );
    // Puxador e linha da porta.
    box(
      fixed,
      PALETTE.chassisDark,
      0.04,
      0.05,
      0.22,
      s * (CAB.width / 2 + 0.01),
      CAB.y - 0.1,
      CAB.z + 0.2,
      0.01,
    );
    // Retrovisor: braço + espelho.
    box(
      fixed,
      PALETTE.chassisDark,
      0.28,
      0.04,
      0.04,
      s * (CAB.width / 2 + 0.12),
      CAB.y + 0.75,
      CAB_FRONT - 0.15,
      0.01,
    );
    box(
      fixed,
      PALETTE.chassisDark,
      0.1,
      0.42,
      0.18,
      s * (CAB.width / 2 + 0.28),
      CAB.y + 0.55,
      CAB_FRONT - 0.15,
      0.03,
    );
    // Degrau de acesso.
    box(
      fixed,
      PALETTE.chassis,
      0.32,
      0.06,
      0.3,
      s * (CAB.width / 2 - 0.12),
      CAB_BOTTOM - 0.28,
      archRear - 0.22,
      0.02,
    );
  }

  // Grade com moldura clara.
  box(fixed, PALETTE.trim, 1.84, 0.66, 0.04, 0, CAB.y - 0.42, CAB_FRONT + 0.01, 0.02);
  box(fixed, PALETTE.grille, 1.7, 0.54, 0.05, 0, CAB.y - 0.42, CAB_FRONT + 0.025, 0.02);
  for (let i = 0; i < 4; i++) {
    box(
      fixed,
      PALETTE.chassisDark,
      1.6,
      0.03,
      0.02,
      0,
      CAB.y - 0.6 + i * 0.12,
      CAB_FRONT + 0.055,
      0.005,
    );
  }

  // Para-choque com faróis e piscas.
  const bumperY = CAB_BOTTOM - 0.24;
  box(fixed, PALETTE.chassis, CAB.width - 0.02, 0.4, 0.36, 0, bumperY, CAB_FRONT - 0.1, 0.07);
  for (const s of [-1, 1]) {
    box(heads, null, 0.42, 0.17, 0.04, s * 0.78, bumperY + 0.03, CAB_FRONT + 0.085, 0.02);
    box(
      fixed,
      PALETTE.indicator,
      0.14,
      0.12,
      0.04,
      s * 1.13,
      bumperY + 0.03,
      CAB_FRONT + 0.085,
      0.02,
    );
  }
  // Placa.
  box(fixed, PALETTE.stripeWhite, 0.52, 0.12, 0.02, 0, bumperY - 0.08, CAB_FRONT + 0.08, 0.01);

  // --- Chassi -----------------------------------------------------------------
  const railY = -0.08;
  for (const s of [-1, 1]) {
    box(
      fixed,
      PALETTE.chassis,
      0.2,
      0.32,
      CAB_FRONT - REAR - 0.3,
      s * 0.5,
      railY,
      (CAB_FRONT - 0.3 + REAR) / 2,
      0.02,
    );
  }
  // Plataforma atrás da cabine.
  box(fixed, PALETTE.chassisDark, 1.6, 0.08, 3.9, 0, railY + 0.2, -1.3, 0.02);
  // Quinta-roda.
  const fifth = facetedCylinder("fifth", { diameter: 1.05, height: 0.14, sides: 12 }, scene);
  fifth.position.set(0, railY + 0.3, -1.95);
  fixed.push(paint(fifth, PALETTE.chassisDark));
  box(fixed, PALETTE.chassis, 0.22, 0.04, 0.5, 0, railY + 0.38, -2.15, 0.01);

  // Tanque de combustível (esquerda) e caixa de ferramentas/bateria (direita).
  box(fixed, PALETTE.trim, 0.6, 0.6, 1.25, 1.02, railY - 0.25, 0.55, 0.12);
  box(fixed, PALETTE.chassisDark, 0.56, 0.08, 1.3, 1.02, railY - 0.25, 0.55, 0.02).scaling.set(
    1.02,
    1,
    1,
  );
  box(fixed, PALETTE.chassisDark, 0.55, 0.55, 0.9, -1.02, railY - 0.25, 0.6, 0.06);

  // Para-lamas sobre o eixo duplo traseiro e lameiros.
  const rearZ = ((spec.wheels[2]?.z ?? -1.3) + (spec.wheels[4]?.z ?? -2.65)) / 2;
  for (const s of [-1, 1]) {
    box(fixed, PALETTE.chassisDark, 0.52, 0.07, 2.85, s * 1.05, wheelTop + 0.09, rearZ, 0.03);
    box(fixed, PALETTE.chassisDark, 0.5, 0.5, 0.03, s * 1.05, -0.3, REAR + 0.15, 0.01);
  }

  // Traseira: barra com listras, lanternas.
  const barY = -0.52;
  box(fixed, PALETTE.chassis, 2.1, 0.14, 0.12, 0, barY, REAR, 0.02);
  const stripes = 8;
  for (let i = 0; i < stripes; i++) {
    const w = 2.0 / stripes;
    box(
      fixed,
      i % 2 === 0 ? PALETTE.stripeRed : PALETTE.stripeWhite,
      w,
      0.1,
      0.02,
      -1.0 + w / 2 + i * w,
      barY,
      REAR - 0.065,
      0.003,
    );
  }
  for (const s of [-1, 1]) {
    box(tails, null, 0.36, 0.14, 0.05, s * 0.78, railY - 0.08, REAR + 0.02, 0.02);
    box(fixed, PALETTE.indicator, 0.12, 0.14, 0.05, s * 1.03, railY - 0.08, REAR + 0.02, 0.02);
  }

  const parent = (mesh: Mesh, material: StandardMaterial, castsShadow = true) => {
    mesh.material = material;
    mesh.parent = root;
    mesh.receiveShadows = true;
    if (castsShadow) {
      shadows.addShadowCaster(mesh);
    }
  };
  parent(mergeParts("truck-fixed", fixed), flat);
  parent(mergeParts("truck-paint", painted), paintMaterial);
  parent(mergeParts("truck-glass", glazed), glass, false);
  parent(mergeParts("truck-headlights", heads), head, false);
  parent(mergeParts("truck-taillights", tails), tailMaterial, false);

  return { flatMaterial: flat, paintMaterial, tailMaterial };
}
